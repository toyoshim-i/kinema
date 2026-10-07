use crate::model::{DocPage, ResolvedLink, TopicSummary};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct DocIndex {
    pub pages: Vec<DocPage>,
    pub id_map: HashMap<String, usize>,
    pub path_map: HashMap<String, usize>,
}

impl DocIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_page(&mut self, page: DocPage) {
        let idx = self.pages.len();
        let id_lower = page.id.to_lowercase();
        self.id_map.insert(id_lower.clone(), idx);

        for alias in &page.aliases {
            self.id_map.insert(alias.to_lowercase(), idx);
        }

        let normalized_path = normalize_path_str(&page.path);
        self.path_map.insert(normalized_path.clone(), idx);

        let file_stem = Path::new(&page.path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();
        if !file_stem.is_empty() {
            self.id_map.entry(file_stem).or_insert(idx);
        }

        self.pages.push(page);
    }

    pub fn lookup(&self, query: &str) -> Option<&DocPage> {
        let q = query.trim().to_lowercase();

        // 1. Exact ID or alias or file stem match
        if let Some(&idx) = self.id_map.get(&q) {
            return Some(&self.pages[idx]);
        }

        // 2. Normalized path match
        let q_path = normalize_path_str(&q);
        if let Some(&idx) = self.path_map.get(&q_path) {
            return Some(&self.pages[idx]);
        }

        // 3. Prefix or contains match
        for page in &self.pages {
            if page.id.to_lowercase().contains(&q)
                || page.title.to_lowercase().contains(&q)
                || page.aliases.iter().any(|a| a.to_lowercase().contains(&q))
            {
                return Some(page);
            }
        }

        None
    }

    pub fn resolve_link(&self, from_path: &str, target: &str) -> Option<ResolvedLink> {
        if target.starts_with("http://") || target.starts_with("https://") {
            return None;
        }

        let (path_part, anchor) = if let Some((p, a)) = target.split_once('#') {
            (p, Some(a.to_string()))
        } else {
            (target, None)
        };

        if path_part.is_empty() {
            // Anchor link in same page
            if let Some(&idx) = self.path_map.get(&normalize_path_str(from_path)) {
                let page = &self.pages[idx];
                return Some(ResolvedLink {
                    target_id: page.id.clone(),
                    category: page.category.clone(),
                    anchor,
                    label: String::new(),
                });
            }
            return None;
        }

        // Relative path resolution
        let from_dir = Path::new(from_path).parent().unwrap_or_else(|| Path::new(""));
        let resolved_path = normalize_path(&from_dir.join(path_part));
        let resolved_str = resolved_path.to_string_lossy().to_string();

        if let Some(&idx) = self.path_map.get(&resolved_str) {
            let page = &self.pages[idx];
            return Some(ResolvedLink {
                target_id: page.id.clone(),
                category: page.category.clone(),
                anchor,
                label: String::new(),
            });
        }

        // Fallback: check file stem
        if let Some(stem) = Path::new(path_part).file_stem().and_then(|s| s.to_str()) {
            if let Some(&idx) = self.id_map.get(&stem.to_lowercase()) {
                let page = &self.pages[idx];
                return Some(ResolvedLink {
                    target_id: page.id.clone(),
                    category: page.category.clone(),
                    anchor,
                    label: String::new(),
                });
            }
        }

        None
    }

    pub fn list_topics(&self, category_filter: Option<&str>) -> Vec<TopicSummary> {
        self.pages
            .iter()
            .filter(|p| {
                if let Some(cat) = category_filter {
                    p.category.eq_ignore_ascii_case(cat)
                } else {
                    true
                }
            })
            .map(|p| TopicSummary {
                id: p.id.clone(),
                title: p.title.clone(),
                summary: p.summary.clone(),
                category: p.category.clone(),
                aliases: p.aliases.clone(),
            })
            .collect()
    }
}

fn normalize_path_str(p: &str) -> String {
    normalize_path(Path::new(p)).to_string_lossy().to_string()
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    for comp in path.components() {
        match comp {
            std::path::Component::ParentDir => {
                components.pop();
            }
            std::path::Component::Normal(c) => {
                components.push(c);
            }
            _ => {}
        }
    }
    let mut out = PathBuf::new();
    for c in components {
        out.push(c);
    }
    out
}
