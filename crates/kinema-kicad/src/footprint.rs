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

#[derive(Debug, Clone, Default)]
pub struct FootprintResolverOptions {
    pub fp_lib_table: Option<PathBuf>,
    pub kicad_config_dir: Option<PathBuf>,
    pub kicad_footprint_dir: Option<PathBuf>,
    pub search_dirs: Vec<PathBuf>,
}

impl FpLibTable {
    pub fn parse(content: &str, base_dir: Option<&Path>) -> Result<Self, String> {
        Self::parse_with_env(content, base_dir, &HashMap::new())
    }

    pub fn parse_with_env(
        content: &str,
        base_dir: Option<&Path>,
        env_vars: &HashMap<String, String>,
    ) -> Result<Self, String> {
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
                    let expanded_uri = expand_env_vars(&lib_uri, base_dir, env_vars);
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

    pub fn discover_project(start_dir: &Path) -> Option<Self> {
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
        None
    }

    pub fn discover_global_with_dir(
        explicit_config_dir: Option<&Path>,
        explicit_footprint_dir: Option<&Path>,
    ) -> Option<Self> {
        let config_dir = explicit_config_dir
            .map(|p| p.to_path_buf())
            .or_else(|| std::env::var("KICAD_CONFIG_DIR").ok().map(PathBuf::from))
            .or_else(kicad_config_dir)?;

        // If config_dir itself contains fp-lib-table, use it directly
        let (version_dir, table_path) = if config_dir.join("fp-lib-table").exists() {
            let table = config_dir.join("fp-lib-table");
            (config_dir.clone(), table)
        } else if let Some(v_dir) = find_latest_kicad_version_dir(&config_dir) {
            let table = v_dir.join("fp-lib-table");
            (v_dir, table)
        } else {
            return None;
        };

        if !table_path.exists() {
            return None;
        }

        let content = fs::read_to_string(&table_path).ok()?;
        let mut env_vars = read_kicad_common_env(&version_dir);

        // Inject explicit or environment footprint directory
        let fp_dir = explicit_footprint_dir
            .map(|p| p.to_string_lossy().to_string())
            .or_else(|| std::env::var("KICAD_FOOTPRINT_DIR").ok());
        if let Some(dir) = fp_dir {
            env_vars.insert("KICAD_FOOTPRINT_DIR".to_string(), dir.clone());
            env_vars.insert("KICAD10_FOOTPRINT_DIR".to_string(), dir.clone());
            env_vars.insert("KICAD9_FOOTPRINT_DIR".to_string(), dir.clone());
            env_vars.insert("KICAD8_FOOTPRINT_DIR".to_string(), dir.clone());
            env_vars.insert("KICAD7_FOOTPRINT_DIR".to_string(), dir);
        }

        Self::parse_with_env(&content, version_dir.parent(), &env_vars).ok()
    }

    pub fn discover_global() -> Option<Self> {
        Self::discover_global_with_dir(None, None)
    }

    pub fn discover_with_options(start_dir: &Path, options: &FootprintResolverOptions) -> Option<Self> {
        if let Some(table_path) = &options.fp_lib_table {
            if let Ok(content) = fs::read_to_string(table_path) {
                let mut env_vars = HashMap::new();
                if let Some(dir) = &options.kicad_footprint_dir {
                    let d = dir.to_string_lossy().to_string();
                    env_vars.insert("KICAD_FOOTPRINT_DIR".to_string(), d.clone());
                    env_vars.insert("KICAD10_FOOTPRINT_DIR".to_string(), d.clone());
                    env_vars.insert("KICAD9_FOOTPRINT_DIR".to_string(), d.clone());
                    env_vars.insert("KICAD8_FOOTPRINT_DIR".to_string(), d.clone());
                    env_vars.insert("KICAD7_FOOTPRINT_DIR".to_string(), d);
                }
                return Self::parse_with_env(&content, table_path.parent(), &env_vars).ok();
            }
        }

        let global_table = Self::discover_global_with_dir(
            options.kicad_config_dir.as_deref(),
            options.kicad_footprint_dir.as_deref(),
        );
        let project_table = Self::discover_project(start_dir);

        match (global_table, project_table) {
            (Some(mut global), Some(project)) => {
                // Project libraries override or augment global libraries
                global.libraries.extend(project.libraries);
                Some(global)
            }
            (Some(global), None) => Some(global),
            (None, Some(project)) => Some(project),
            (None, None) => None,
        }
    }

    pub fn discover(start_dir: &Path) -> Option<Self> {
        Self::discover_with_options(start_dir, &FootprintResolverOptions::default())
    }
}

pub fn kicad_config_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("KICAD_CONFIG_HOME") {
        let p = PathBuf::from(dir);
        if p.exists() {
            return Some(p);
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            let p = PathBuf::from(appdata).join("kicad");
            if p.exists() {
                return Some(p);
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            let p = PathBuf::from(home).join("Library/Preferences/kicad");
            if p.exists() {
                return Some(p);
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            let p = PathBuf::from(xdg).join("kicad");
            if p.exists() {
                return Some(p);
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            let p = PathBuf::from(home).join(".config/kicad");
            if p.exists() {
                return Some(p);
            }
        }
    }

    // Fallbacks
    if let Ok(home) = std::env::var("HOME") {
        let mac_p = PathBuf::from(&home).join("Library/Preferences/kicad");
        if mac_p.exists() {
            return Some(mac_p);
        }
        let linux_p = PathBuf::from(&home).join(".config/kicad");
        if linux_p.exists() {
            return Some(linux_p);
        }
    }

    None
}

fn find_latest_kicad_version_dir(base_dir: &Path) -> Option<PathBuf> {
    let entries = fs::read_dir(base_dir).ok()?;
    let mut version_dirs: Vec<PathBuf> = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let table = path.join("fp-lib-table");
            if table.exists() {
                version_dirs.push(path);
            }
        }
    }

    version_dirs.sort_by(|a, b| {
        let a_name = a.file_name().and_then(|s| s.to_str()).unwrap_or("");
        let b_name = b.file_name().and_then(|s| s.to_str()).unwrap_or("");
        let a_ver = a_name.parse::<f64>().ok();
        let b_ver = b_name.parse::<f64>().ok();
        match (a_ver, b_ver) {
            (Some(av), Some(bv)) => bv.partial_cmp(&av).unwrap_or(std::cmp::Ordering::Equal),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => b_name.cmp(a_name),
        }
    });

    version_dirs.into_iter().next()
}

fn read_kicad_common_env(version_dir: &Path) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let common_json = version_dir.join("kicad_common.json");
    if let Ok(content) = fs::read_to_string(&common_json) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(vars) = val.get("environment").and_then(|e| e.get("vars")).and_then(|v| v.as_object()) {
                for (k, v) in vars {
                    if let Some(s) = v.as_str() {
                        map.insert(k.clone(), s.to_string());
                    }
                }
            }
        }
    }
    map
}

fn find_default_footprint_dir() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let standard_macos = [
            "/Applications/KiCad/KiCad.app/Contents/SharedSupport/footprints",
            "/Applications/KiCad.app/Contents/SharedSupport/footprints",
            "/Library/Application Support/kicad/footprints",
        ];
        for p in standard_macos {
            let path = PathBuf::from(p);
            if path.exists() {
                return Some(path);
            }
        }
        if let Ok(entries) = fs::read_dir("/Applications") {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.starts_with("KiCad") {
                    let candidate = entry.path().join("Contents/SharedSupport/footprints");
                    if candidate.exists() {
                        return Some(candidate);
                    }
                    let candidate2 = entry.path().join("KiCad.app/Contents/SharedSupport/footprints");
                    if candidate2.exists() {
                        return Some(candidate2);
                    }
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let standard_linux = [
            "/usr/share/kicad/footprints",
            "/usr/local/share/kicad/footprints",
            "/app/share/kicad/footprints",
        ];
        for p in standard_linux {
            let path = PathBuf::from(p);
            if path.exists() {
                return Some(path);
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(pf) = std::env::var("ProgramFiles") {
            let p = PathBuf::from(&pf).join("KiCad").join("share").join("kicad").join("footprints");
            if p.exists() {
                return Some(p);
            }
            let kicad_dir = PathBuf::from(&pf).join("KiCad");
            if kicad_dir.exists() {
                if let Ok(entries) = fs::read_dir(&kicad_dir) {
                    for entry in entries.flatten() {
                        let candidate = entry.path().join("share/kicad/footprints");
                        if candidate.exists() {
                            return Some(candidate);
                        }
                    }
                }
            }
        }
        let fallback = PathBuf::from("C:\\Program Files\\KiCad\\share\\kicad\\footprints");
        if fallback.exists() {
            return Some(fallback);
        }
    }

    None
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

    pub fn auto_discover_with_options(start_dir: &Path, options: &FootprintResolverOptions) -> Self {
        let table = FpLibTable::discover_with_options(start_dir, options);
        let mut search_dirs = vec![start_dir.to_path_buf()];
        // Also look for local .pretty or footprints/ directory
        if start_dir.join("footprints").exists() {
            search_dirs.push(start_dir.join("footprints"));
        }
        for dir in &options.search_dirs {
            if !search_dirs.contains(dir) {
                search_dirs.push(dir.clone());
            }
        }
        if let Some(fp_dir) = &options.kicad_footprint_dir {
            if !search_dirs.contains(fp_dir) {
                search_dirs.push(fp_dir.clone());
            }
        }
        Self { table, search_dirs }
    }

    pub fn auto_discover(start_dir: &Path) -> Self {
        Self::auto_discover_with_options(start_dir, &FootprintResolverOptions::default())
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
        } else if self.table.is_some() {
            Err(FootprintResolveError::LibraryNotFound(lib_name.to_string()))
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

fn expand_env_vars(s: &str, base_dir: Option<&Path>, extra_env: &HashMap<String, String>) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
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
            if var_name == "KIPRJMOD" {
                if let Some(base) = base_dir {
                    out.push_str(&base.to_string_lossy());
                } else {
                    out.push('.');
                }
            } else if let Some(val) = extra_env.get(&var_name) {
                out.push_str(val);
            } else if let Ok(val) = std::env::var(&var_name) {
                out.push_str(&val);
            } else if var_name.starts_with("KICAD") && (var_name.ends_with("_FOOTPRINT_DIR") || var_name == "KICAD_FOOTPRINT_DIR") {
                if let Some(fp_dir) = find_default_footprint_dir() {
                    out.push_str(&fp_dir.to_string_lossy());
                } else {
                    #[cfg(target_os = "windows")]
                    out.push_str("C:/Program Files/KiCad/share/kicad/footprints");
                    #[cfg(target_os = "macos")]
                    out.push_str("/Applications/KiCad/KiCad.app/Contents/SharedSupport/footprints");
                    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
                    out.push_str("/usr/share/kicad/footprints");
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}
