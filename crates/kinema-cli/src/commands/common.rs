use kinema_lint::codes;
use kinema_lint::diagnostic::{Diagnostic, LintReport, Subject};
use kinema_syntax::ast::{SourceFile, SourceLocation};
use kinema_syntax::parser::parse;
use kinema_syntax::project::ProjectConfig;
use std::fs;
use std::path::{Path, PathBuf};

pub fn resolve_project_sources(specified_files: &[String]) -> (Vec<PathBuf>, Option<PathBuf>) {
    let mut paths = Vec::new();
    let mut board_path = None;

    let config = if Path::new("kinema.toml").exists() {
        ProjectConfig::load_or_default("kinema.toml")
    } else {
        ProjectConfig::default()
    };

    if !specified_files.is_empty() && specified_files.iter().any(|f| !f.is_empty()) {
        for f in specified_files {
            if !f.is_empty() {
                let p = PathBuf::from(f);
                if p.extension().is_some_and(|ext| ext == "kicad_pcb") {
                    board_path = Some(p);
                } else {
                    paths.push(p);
                }
            }
        }
        // Always include lib/std.v if it exists and not already included
        let std_lib = PathBuf::from("lib/std.v");
        if std_lib.exists() && !paths.contains(&std_lib) {
            paths.insert(0, std_lib);
        }
    } else {
        // Collect from kinema.toml
        for pattern in &config.project.sources {
            if let Ok(entries) = glob::glob(pattern) {
                for entry in entries.flatten() {
                    paths.push(entry);
                }
            }
        }
        for pattern in &config.project.libraries {
            if let Ok(entries) = glob::glob(pattern) {
                for entry in entries.flatten() {
                    paths.push(entry);
                }
            }
        }
        if let Some(b) = &config.project.board {
            board_path = Some(PathBuf::from(b));
        }
    }

    (paths, board_path)
}

pub fn load_source_files(paths: &[PathBuf]) -> Result<Vec<SourceFile>, LintReport> {
    let mut files = Vec::new();
    let mut diags = Vec::new();

    for path in paths {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                diags.push(Diagnostic {
                    stage: "parse".into(),
                    code: codes::IO_ERROR.into(),
                    severity: "error".into(),
                    location: Some(SourceLocation {
                        file: path.display().to_string(),
                        line: 1,
                        col: 1,
                    }),
                    subject: Subject {
                        kind: "file".into(),
                        name: Some(path.display().to_string()),
                        path: None,
                        id: None,
                        ref_des: None,
                        pad: None,
                    },
                    related: vec![],
                    expected: None,
                    actual: None,
                    fix: None,
                    message: format!("Failed to read file: {}", e),
                });
                continue;
            }
        };

        match parse(&path.display().to_string(), &content) {
            Ok(ast) => files.push(ast),
            Err(e) => {
                diags.push(Diagnostic {
                    stage: "parse".into(),
                    code: codes::SYNTAX_ERROR.into(),
                    severity: "error".into(),
                    location: Some(SourceLocation {
                        file: e.file,
                        line: e.line,
                        col: e.col,
                    }),
                    subject: Subject {
                        kind: "syntax".into(),
                        name: None,
                        path: None,
                        id: None,
                        ref_des: None,
                        pad: None,
                    },
                    related: vec![],
                    expected: None,
                    actual: None,
                    fix: None,
                    message: e.message,
                });
            }
        }
    }

    if !diags.is_empty() {
        Err(LintReport::from_diagnostics(diags))
    } else {
        Ok(files)
    }
}

pub fn format_report(report: &LintReport, json: bool) -> String {
    if json {
        format!("{}\n", serde_json::to_string_pretty(report).unwrap_or_default())
    } else if report.diagnostics.is_empty() {
        "All checks passed (0 errors, 0 warnings).\n".to_string()
    } else {
        let mut out = String::new();
        let doc_engine = crate::doc::build_doc_engine();
        for d in &report.diagnostics {
            let sev = if d.severity == "error" {
                "ERROR"
            } else {
                "WARNING"
            };
            let loc_str = if let Some(loc) = &d.location {
                format!(" [{}:{}]", loc.file, loc.line)
            } else {
                String::new()
            };
            out.push_str(&format!("[{}] {}{}: {}\n", sev, d.code, loc_str, d.message));
            if let Some(f) = &d.fix {
                out.push_str(&format!("  Suggestion: {}\n", f));
            }
            if let Some(hint) = doc_engine.hint(&d.code) {
                out.push_str(&format!("  {}\n", hint));
            }
        }
        out
    }
}
