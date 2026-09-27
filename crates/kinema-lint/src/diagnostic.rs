use kinema_syntax::ast::SourceLocation;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subject {
    pub kind: String, // "hub", "pin-wire", "join", "pin", "net", "component", "pad"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
    pub ref_des: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pad: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Related {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<SourceLocation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub stage: String, // "lint", "parse", "fmt", "equiv", "drc"
    pub code: String,
    pub severity: String, // "error" or "warning"
    pub location: SourceLocation,
    pub subject: Subject,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related: Vec<Related>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LintReport {
    pub ok: bool,
    pub diagnostics: Vec<Diagnostic>,
}

impl LintReport {
    pub fn success() -> Self {
        Self {
            ok: true,
            diagnostics: Vec::new(),
        }
    }

    pub fn from_diagnostics(diagnostics: Vec<Diagnostic>) -> Self {
        let has_errors = diagnostics.iter().any(|d| d.severity == "error");
        Self {
            ok: !has_errors,
            diagnostics,
        }
    }
}
