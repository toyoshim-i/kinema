use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FootprintInfo {
    pub name: String,
    pub pad_count: usize,
    pub pads: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FootprintResolveError {
    InvalidFormat(String),
    LibraryNotFound(String),
    FootprintNotFound(String),
    TableNotFound,
    ParseError(String),
}

impl std::fmt::Display for FootprintResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFormat(s) => write!(f, "Invalid footprint identifier format: '{}' (expected 'Library:Footprint')", s),
            Self::LibraryNotFound(l) => write!(f, "Footprint library '{}' not found in fp-lib-table", l),
            Self::FootprintNotFound(fp) => write!(f, "Footprint '{}' does not exist in library", fp),
            Self::TableNotFound => write!(f, "fp-lib-table not found"),
            Self::ParseError(e) => write!(f, "Failed to parse .kicad_mod: {}", e),
        }
    }
}

impl std::error::Error for FootprintResolveError {}

#[derive(Debug, Clone, Default)]
pub struct FpLibTable {
    pub libraries: HashMap<String, PathBuf>, // lib_name -> .pretty directory path
}

impl FpLibTable {
    pub fn parse(content: &str, base_dir: Option<&Path>) -> Result<Self, String> {
        let mut libraries = HashMap::new();
        let bytes = content.as_bytes();
        let mut cursor = 0;

        // Simple S-expression parser for (lib (name "...")(type "...")(uri "..."))
        while cursor < bytes.len() {
            if cursor + 4 <= bytes.len() && &bytes[cursor..cursor + 4] == b"(lib" {
                cursor += 4;
                let mut lib_name = String::new();
                let mut lib_uri = String::new();

                let mut depth = 1;
                while cursor < bytes.len() && depth > 0 {
                    let b = bytes[cursor];
                    cursor += 1;
                    if b == b'(' {
                        depth += 1;
                        if cursor + 4 <= bytes.len() && &bytes[cursor..cursor + 4] == b"name" {
                            cursor += 4;
                            lib_name = extract_next_string(bytes, &mut cursor);
                        } else if cursor + 3 <= bytes.len() && &bytes[cursor..cursor + 3] == b"uri" {
                            cursor += 3;
                            lib_uri = extract_next_string(bytes, &mut cursor);
                        }
                    } else if b == b')' {
                        depth -= 1;
                    }
                }

                if !lib_name.is_empty() && !lib_uri.is_empty() {
                    let expanded_uri = expand_env_vars(&lib_uri);
                    let mut path = PathBuf::from(&expanded_uri);
                    if path.is_relative() {
                        if let Some(base) = base_dir {
                            path = base.join(path);
                        }
                    }
                    libraries.insert(lib_name, path);
                }
            } else {
                cursor += 1;
            }
        }

        Ok(Self { libraries })
    }

    pub fn discover(start_dir: &Path) -> Option<Self> {
        // 1. Search start_dir and parents
        let mut curr = Some(start_dir);
        while let Some(dir) = curr {
            let fp_table = dir.join("fp-lib-table");
            if fp_table.exists() {
                if let Ok(content) = fs::read_to_string(&fp_table) {
                    if let Ok(table) = Self::parse(&content, Some(dir)) {
                        return Some(table);
                    }
                }
            }
            curr = dir.parent();
        }

        // 2. Global KiCad configuration paths
        #[cfg(target_os = "windows")]
        {
            if let Ok(appdata) = std::env::var("APPDATA") {
                let kicad_base = PathBuf::from(appdata).join("kicad");
                if kicad_base.exists() {
                    if let Ok(entries) = fs::read_dir(&kicad_base) {
                        for entry in entries.flatten() {
                            let table_path = entry.path().join("fp-lib-table");
                            if table_path.exists() {
                                if let Ok(content) = fs::read_to_string(&table_path) {
                                    if let Ok(table) = Self::parse(&content, entry.path().parent()) {
                                        return Some(table);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            if let Ok(home) = std::env::var("HOME") {
                let kicad_base = PathBuf::from(home).join(".config/kicad");
                if kicad_base.exists() {
                    if let Ok(entries) = fs::read_dir(&kicad_base) {
                        for entry in entries.flatten() {
                            let table_path = entry.path().join("fp-lib-table");
                            if table_path.exists() {
                                if let Ok(content) = fs::read_to_string(&table_path) {
                                    if let Ok(table) = Self::parse(&content, entry.path().parent()) {
                                        return Some(table);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }
}

pub fn parse_kicad_mod(content: &str) -> Result<FootprintInfo, String> {
    let bytes = content.as_bytes();
    let mut cursor = 0;
    let mut fp_name = String::new();
    let mut pads = Vec::new();
    let mut seen_pads = HashSet::new();

    while cursor < bytes.len() {
        if cursor + 10 <= bytes.len() && &bytes[cursor..cursor + 10] == b"(footprint" {
            cursor += 10;
            fp_name = extract_next_string(bytes, &mut cursor);
            break;
        } else if cursor + 7 <= bytes.len() && &bytes[cursor..cursor + 7] == b"(module" {
            cursor += 7;
            fp_name = extract_next_string(bytes, &mut cursor);
            break;
        }
        cursor += 1;
    }

    while cursor < bytes.len() {
        if cursor + 4 <= bytes.len() && &bytes[cursor..cursor + 4] == b"(pad" {
            cursor += 4;
            let pad_num = extract_next_string(bytes, &mut cursor);
            if !pad_num.is_empty() && seen_pads.insert(pad_num.clone()) {
                pads.push(pad_num);
            }
        } else {
            cursor += 1;
        }
    }

    Ok(FootprintInfo {
        name: fp_name,
        pad_count: pads.len(),
        pads,
    })
}

#[derive(Debug, Clone, Default)]
pub struct FootprintResolver {
    pub table: Option<FpLibTable>,
    pub search_dirs: Vec<PathBuf>,
}

impl FootprintResolver {
    pub fn new(table: Option<FpLibTable>, search_dirs: Vec<PathBuf>) -> Self {
        Self { table, search_dirs }
    }

    pub fn auto_discover(start_dir: &Path) -> Self {
        let table = FpLibTable::discover(start_dir);
        let mut search_dirs = vec![start_dir.to_path_buf()];
        // Also look for local .pretty or footprints/ directory
        if start_dir.join("footprints").exists() {
            search_dirs.push(start_dir.join("footprints"));
        }
        Self { table, search_dirs }
    }

    pub fn resolve(&self, fp_id: &str) -> Result<FootprintInfo, FootprintResolveError> {
        let parts: Vec<&str> = fp_id.split(':').collect();
        if parts.len() != 2 {
            return Err(FootprintResolveError::InvalidFormat(fp_id.to_string()));
        }
        let lib_name = parts[0];
        let mod_name = parts[1];

        // 1. Check fp-lib-table if available
        let mut pretty_dir: Option<PathBuf> = None;
        if let Some(table) = &self.table {
            if let Some(path) = table.libraries.get(lib_name) {
                if path.exists() {
                    pretty_dir = Some(path.clone());
                } else {
                    return Err(FootprintResolveError::FootprintNotFound(fp_id.to_string()));
                }
            } else {
                return Err(FootprintResolveError::LibraryNotFound(lib_name.to_string()));
            }
        }

        // 2. Check search_dirs for <lib_name>.pretty
        if pretty_dir.is_none() {
            for dir in &self.search_dirs {
                let candidate = dir.join(format!("{}.pretty", lib_name));
                if candidate.exists() {
                    pretty_dir = Some(candidate);
                    break;
                }
                let candidate2 = dir.join("footprints").join(format!("{}.pretty", lib_name));
                if candidate2.exists() {
                    pretty_dir = Some(candidate2);
                    break;
                }
            }
        }

        if let Some(dir) = pretty_dir {
            let mod_path = dir.join(format!("{}.kicad_mod", mod_name));
            if mod_path.exists() {
                let content = fs::read_to_string(&mod_path)
                    .map_err(|e| FootprintResolveError::ParseError(e.to_string()))?;
                parse_kicad_mod(&content).map_err(FootprintResolveError::ParseError)
            } else {
                Err(FootprintResolveError::FootprintNotFound(fp_id.to_string()))
            }
        } else {
            Err(FootprintResolveError::TableNotFound)
        }
    }
}

fn extract_next_string(bytes: &[u8], cursor: &mut usize) -> String {
    while *cursor < bytes.len() && bytes[*cursor].is_ascii_whitespace() {
        *cursor += 1;
    }
    if *cursor >= bytes.len() {
        return String::new();
    }
    if bytes[*cursor] == b'"' {
        *cursor += 1;
        let start = *cursor;
        while *cursor < bytes.len() && bytes[*cursor] != b'"' {
            *cursor += 1;
        }
        let end = *cursor;
        if *cursor < bytes.len() {
            *cursor += 1; // consume closing quote
        }
        String::from_utf8_lossy(&bytes[start..end]).to_string()
    } else {
        let start = *cursor;
        while *cursor < bytes.len() && !bytes[*cursor].is_ascii_whitespace() && bytes[*cursor] != b')' {
            *cursor += 1;
        }
        String::from_utf8_lossy(&bytes[start..*cursor]).to_string()
    }
}

fn expand_env_vars(s: &str) -> String {
    let mut result = s.to_string();
    let re_dollar = regex_lite_find_and_replace(&result);
    result = re_dollar;
    result
}

fn regex_lite_find_and_replace(input: &str) -> String {
    let mut out = String::new();
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '$' && chars.peek() == Some(&'{') {
            chars.next(); // consume '{'
            let mut var_name = String::new();
            for vc in chars.by_ref() {
                if vc == '}' {
                    break;
                }
                var_name.push(vc);
            }
            if let Ok(val) = std::env::var(&var_name) {
                out.push_str(&val);
            } else {
                // If not found in env, check known default footprints path
                #[cfg(target_os = "windows")]
                if var_name.starts_with("KICAD") && var_name.ends_with("_FOOTPRINT_DIR") {
                    out.push_str("C:/Program Files/KiCad/share/kicad/footprints");
                }
                #[cfg(not(target_os = "windows"))]
                if var_name.starts_with("KICAD") && var_name.ends_with("_FOOTPRINT_DIR") {
                    out.push_str("/usr/share/kicad/footprints");
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}
