use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RenderMode {
    #[default]
    Agent,     // Plain text with actionable CLI link hints, no ANSI escapes
    Terminal,  // Formatted with ANSI styling and OSC 8 terminal hyperlinks
    Json,      // Structured JSON output
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocSection {
    pub heading: String,
    pub level: usize,
    pub anchor: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocPage {
    pub path: String,
    pub id: String,
    pub title: String,
    pub summary: String,
    pub category: String,
    pub aliases: Vec<String>,
    pub sections: Vec<DocSection>,
    pub raw_content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedLink {
    pub target_id: String,
    pub category: String,
    pub anchor: Option<String>,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicSummary {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub category: String,
    pub aliases: Vec<String>,
}
