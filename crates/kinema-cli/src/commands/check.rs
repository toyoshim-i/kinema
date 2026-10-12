use super::common::{format_report, load_source_files, resolve_project_sources};
use super::CommandResult;
use clap::ValueEnum;
use kinema_elab::elaborate_sources;
use kinema_equiv::check_equivalence;
use kinema_fmt::format_str;
use kinema_kicad::footprint::{FootprintResolver, FootprintResolverOptions};
use kinema_kicad::parse_kicad_pcb;
use kinema_lint::codes;
use kinema_lint::diagnostic::{Diagnostic, LintReport, Subject};
use kinema_lint::lint_source_files_with_resolver;
use kinema_syntax::ast::SourceLocation;
use kinema_syntax::project::ProjectConfig;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, ValueEnum, Debug, PartialEq, Eq)]
pub enum Stage {
    Parse,
    Fmt,
    Lint,
    Equiv,
    Drc,
}

#[derive(Debug, Clone, Default)]
pub struct CheckArgs {
    pub stage: Option<Stage>,
    pub json: bool,
    pub mode: Option<agent_man::RenderMode>,
    pub deny_warnings: bool,
    pub strict: bool,
    pub kicad_config_dir: Option<PathBuf>,
    pub kicad_footprint_dir: Option<PathBuf>,
    pub fp_lib_table: Option<PathBuf>,
    pub search_paths: Vec<PathBuf>,
    pub files: Vec<String>,
}

pub fn run_check(args: &CheckArgs) -> CommandResult {
    let mode = args.mode.unwrap_or_else(|| {
        if args.json {
            agent_man::RenderMode::Json
        } else if !std::io::IsTerminal::is_terminal(&std::io::stdout()) {
            agent_man::RenderMode::Agent
        } else {
            agent_man::RenderMode::Terminal
        }
    });

    let (paths, board_path) = resolve_project_sources(&args.files);
    let source_files = match load_source_files(&paths) {
        Ok(f) => f,
        Err(report) => {
            let output = format_report(&report, mode);
            let data = serde_json::to_value(&report).ok();
            return CommandResult {
                stdout: output,
                stderr: String::new(),
                success: false,
                data,
            };
        }
    };

    let mut all_diags = Vec::new();

    // 1. Parse stage only: already verified by load_source_files
    if args.stage == Some(Stage::Parse) {
        let report = LintReport::from_diagnostics(all_diags);
        let output = format_report(&report, mode);
        let data = serde_json::to_value(&report).ok();
        let mut res = CommandResult::success(output);
        res.data = data;
        return res;
    }

    // 2. Fmt stage
    if args.stage.is_none() || args.stage == Some(Stage::Fmt) {
        for path in &paths {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(formatted) = format_str(&path.display().to_string(), &content) {
                    if content != formatted {
                        all_diags.push(Diagnostic {
                            stage: "fmt".into(),
                            code: codes::NOT_FORMATTED.into(),
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
                            fix: Some(format!(
                                "Run 'kinema fmt {}' to format canonically",
                                path.display()
                            )),
                            message: format!("File '{}' is not formatted canonically", path.display()),
                        });
                    }
                }
            }
        }
    }

    // 3. Lint stage (AST rules + Elaboration verification)
    if args.stage.is_none() || args.stage == Some(Stage::Lint) {
        let config = if Path::new("kinema.toml").exists() {
            ProjectConfig::load_or_default("kinema.toml")
        } else {
            ProjectConfig::default()
        };

        let effective_fp_table = args
            .fp_lib_table
            .clone()
            .or_else(|| config.kicad.as_ref().and_then(|k| k.fp_lib_table.clone()));

        let effective_config_dir = args
            .kicad_config_dir
            .clone()
            .or_else(|| std::env::var("KICAD_CONFIG_DIR").ok().map(PathBuf::from))
            .or_else(|| config.kicad.as_ref().and_then(|k| k.config_dir.clone()));

        let effective_footprint_dir = args
            .kicad_footprint_dir
            .clone()
            .or_else(|| std::env::var("KICAD_FOOTPRINT_DIR").ok().map(PathBuf::from))
            .or_else(|| config.kicad.as_ref().and_then(|k| k.footprint_dir.clone()));

        let mut effective_search_paths = args.search_paths.clone();
        if let Some(kicad_sec) = &config.kicad {
            for sp in &kicad_sec.search_paths {
                if !effective_search_paths.contains(sp) {
                    effective_search_paths.push(sp.clone());
                }
            }
        }

        let project_dir = paths
            .first()
            .and_then(|p| p.parent())
            .unwrap_or(Path::new("."));

        let resolver_opts = FootprintResolverOptions {
            fp_lib_table: effective_fp_table,
            kicad_config_dir: effective_config_dir,
            kicad_config_home: None,
            kicad_footprint_dir: effective_footprint_dir,
            search_dirs: effective_search_paths,
        };
        let fp_resolver = FootprintResolver::auto_discover_with_options(project_dir, &resolver_opts);

        let lint_report = lint_source_files_with_resolver(&source_files, &fp_resolver);
        all_diags.extend(lint_report.diagnostics);

        if let Err(e) = elaborate_sources(&source_files) {
            all_diags.push(Diagnostic {
                stage: "lint".into(),
                code: codes::ELAB_ERROR.into(),
                severity: "error".into(),
                location: None,
                subject: Subject {
                    kind: "circuit".into(),
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
                message: format!("Elaboration error: {}", e),
            });
        }
    }

    // 4. Equiv stage
    let should_run_equiv = args.stage == Some(Stage::Equiv)
        || (args.stage.is_none() && (board_path.is_some() || args.strict));
    if should_run_equiv {
        if let Some(bp) = &board_path {
            if bp.exists() {
                match fs::read_to_string(bp) {
                    Ok(content) => match parse_kicad_pcb(&content) {
                        Ok(board) => match elaborate_sources(&source_files) {
                            Ok(ir) => {
                                let equiv_report = check_equivalence(&ir, &board);
                                all_diags.extend(equiv_report.diagnostics);
                            }
                            Err(e) => {
                                if args.stage == Some(Stage::Equiv) {
                                    all_diags.push(Diagnostic {
                                        stage: "equiv".into(),
                                        code: codes::ELAB_ERROR.into(),
                                        severity: "error".into(),
                                        location: Some(SourceLocation {
                                            file: bp.display().to_string(),
                                            line: 1,
                                            col: 1,
                                        }),
                                        subject: Subject {
                                            kind: "circuit".into(),
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
                                        message: format!("Elaboration error: {}", e),
                                    });
                                }
                            }
                        },
                        Err(e) => {
                            all_diags.push(Diagnostic {
                                stage: "equiv".into(),
                                code: codes::PCB_PARSE_ERROR.into(),
                                severity: "error".into(),
                                location: Some(SourceLocation {
                                    file: bp.display().to_string(),
                                    line: 1,
                                    col: 1,
                                }),
                                subject: Subject {
                                    kind: "board".into(),
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
                                message: format!("PCB S-expression parse error: {}", e),
                            });
                        }
                    },
                    Err(e) => {
                        all_diags.push(Diagnostic {
                            stage: "equiv".into(),
                            code: codes::IO_ERROR.into(),
                            severity: "error".into(),
                            location: Some(SourceLocation {
                                file: bp.display().to_string(),
                                line: 1,
                                col: 1,
                            }),
                            subject: Subject {
                                kind: "board".into(),
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
                            message: format!("Failed to read board file: {}", e),
                        });
                    }
                }
            } else {
                all_diags.push(Diagnostic {
                    stage: "equiv".into(),
                    code: codes::BOARD_MISSING.into(),
                    severity: "error".into(),
                    location: Some(SourceLocation {
                        file: bp.display().to_string(),
                        line: 1,
                        col: 1,
                    }),
                    subject: Subject {
                        kind: "board".into(),
                        name: Some(bp.display().to_string()),
                        path: None,
                        id: None,
                        ref_des: None,
                        pad: None,
                    },
                    related: vec![],
                    expected: None,
                    actual: None,
                    fix: Some("Ensure specified board file exists".into()),
                    message: format!("Board file '{}' not found", bp.display()),
                });
            }
        } else {
            all_diags.push(Diagnostic {
                stage: "equiv".into(),
                code: codes::BOARD_MISSING.into(),
                severity: "error".into(),
                location: None,
                subject: Subject {
                    kind: "board".into(),
                    name: None,
                    path: None,
                    id: None,
                    ref_des: None,
                    pad: None,
                },
                related: vec![],
                expected: None,
                actual: None,
                fix: Some("Specify board file via kinema.toml or CLI argument".into()),
                message: "Board file is required for equivalence verification".into(),
            });
        }
    }

    // 5. DRC stage
    let should_run_drc = args.stage == Some(Stage::Drc)
        || (args.stage.is_none() && (board_path.is_some() || args.strict));
    if should_run_drc {
        if let Some(bp) = &board_path {
            if bp.exists() {
                let drc_diags = run_kicad_drc(bp, args.stage == Some(Stage::Drc) || args.strict);
                all_diags.extend(drc_diags);
            } else {
                all_diags.push(Diagnostic {
                    stage: "drc".into(),
                    code: codes::BOARD_MISSING.into(),
                    severity: "error".into(),
                    location: Some(SourceLocation {
                        file: bp.display().to_string(),
                        line: 1,
                        col: 1,
                    }),
                    subject: Subject {
                        kind: "board".into(),
                        name: Some(bp.display().to_string()),
                        path: None,
                        id: None,
                        ref_des: None,
                        pad: None,
                    },
                    related: vec![],
                    expected: None,
                    actual: None,
                    fix: Some("Ensure specified board file exists".into()),
                    message: format!("Board file '{}' not found for DRC check", bp.display()),
                });
            }
        } else {
            all_diags.push(Diagnostic {
                stage: "drc".into(),
                code: codes::BOARD_MISSING.into(),
                severity: "error".into(),
                location: None,
                subject: Subject {
                    kind: "board".into(),
                    name: None,
                    path: None,
                    id: None,
                    ref_des: None,
                    pad: None,
                },
                related: vec![],
                expected: None,
                actual: None,
                fix: Some("Specify board file for DRC check".into()),
                message: "Board file is required for DRC verification".into(),
            });
        }
    }

    let report = LintReport::from_diagnostics(all_diags);
    let mut output = format_report(&report, mode);

    let has_errors = report.diagnostics.iter().any(|d| d.severity == "error");
    let has_warnings = report.diagnostics.iter().any(|d| d.severity == "warning");

    if mode != agent_man::RenderMode::Json && !has_errors && args.stage.is_none() && !args.strict && board_path.is_none() {
        output.push_str("Note: Board equivalence and DRC checks were skipped because no board file was specified. Use --strict for complete sign-off verification.\n");
    }

    let success = !(has_errors || (args.deny_warnings && has_warnings));
    let data = serde_json::to_value(&report).ok();

    CommandResult {
        stdout: output,
        stderr: String::new(),
        success,
        data,
    }
}

fn run_kicad_drc(board_path: &Path, is_strict_or_explicit: bool) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    if !board_path.exists() {
        diags.push(Diagnostic {
            stage: "drc".into(),
            code: codes::BOARD_MISSING.into(),
            severity: "error".into(),
            location: Some(SourceLocation {
                file: board_path.display().to_string(),
                line: 1,
                col: 1,
            }),
            subject: Subject {
                kind: "board".into(),
                name: Some(board_path.display().to_string()),
                path: None,
                id: None,
                ref_des: None,
                pad: None,
            },
            related: vec![],
            expected: None,
            actual: None,
            fix: Some("Ensure board file exists".into()),
            message: format!("Board file '{}' not found for DRC check", board_path.display()),
        });
        return diags;
    }

    let temp_drc_json =
        std::env::temp_dir().join(format!("kinema_drc_{}.json", std::process::id()));
    let status_res = std::process::Command::new("kicad-cli")
        .args([
            "pcb",
            "drc",
            "--format",
            "json",
            "-o",
            &temp_drc_json.display().to_string(),
            &board_path.display().to_string(),
        ])
        .output();

    match status_res {
        Ok(output) => {
            if temp_drc_json.exists() {
                match fs::read_to_string(&temp_drc_json) {
                    Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
                        Ok(json_val) => {
                            let mut found_violations = false;
                            if let Some(violations) =
                                json_val.get("violations").and_then(|v| v.as_array())
                            {
                                for v in violations {
                                    found_violations = true;
                                    let desc = v
                                        .get("description")
                                        .and_then(|d| d.as_str())
                                        .unwrap_or("DRC violation");
                                    let v_type = v
                                        .get("type")
                                        .and_then(|t| t.as_str())
                                        .unwrap_or("drc-violation");
                                    let severity = v
                                        .get("severity")
                                        .and_then(|s| s.as_str())
                                        .unwrap_or("error");
                                    diags.push(Diagnostic {
                                        stage: "drc".into(),
                                        code: v_type.to_string(),
                                        severity: if severity == "warning" {
                                            "warning".into()
                                        } else {
                                            "error".into()
                                        },
                                        location: Some(SourceLocation {
                                            file: board_path.display().to_string(),
                                            line: 1,
                                            col: 1,
                                        }),
                                        subject: Subject {
                                            kind: "board".into(),
                                            name: Some(board_path.display().to_string()),
                                            path: None,
                                            id: None,
                                            ref_des: None,
                                            pad: None,
                                        },
                                        related: vec![],
                                        expected: None,
                                        actual: None,
                                        fix: None,
                                        message: desc.to_string(),
                                    });
                                }
                            }
                            if !found_violations && !output.status.success() {
                                let stderr = String::from_utf8_lossy(&output.stderr);
                                diags.push(Diagnostic {
                                    stage: "drc".into(),
                                    code: codes::DRC_ERROR.into(),
                                    severity: "error".into(),
                                    location: Some(SourceLocation {
                                        file: board_path.display().to_string(),
                                        line: 1,
                                        col: 1,
                                    }),
                                    subject: Subject {
                                        kind: "board".into(),
                                        name: Some(board_path.display().to_string()),
                                        path: None,
                                        id: None,
                                        ref_des: None,
                                        pad: None,
                                    },
                                    related: vec![],
                                    expected: None,
                                    actual: None,
                                    fix: None,
                                    message: format!("kicad-cli DRC failed: {}", stderr.trim()),
                                });
                            }
                        }
                        Err(e) => {
                            diags.push(Diagnostic {
                                stage: "drc".into(),
                                code: codes::DRC_JSON_ERROR.into(),
                                severity: "error".into(),
                                location: Some(SourceLocation {
                                    file: temp_drc_json.display().to_string(),
                                    line: 1,
                                    col: 1,
                                }),
                                subject: Subject {
                                    kind: "board".into(),
                                    name: Some(board_path.display().to_string()),
                                    path: None,
                                    id: None,
                                    ref_des: None,
                                    pad: None,
                                },
                                related: vec![],
                                expected: None,
                                actual: None,
                                fix: None,
                                message: format!("Failed to parse DRC JSON report: {}", e),
                            });
                        }
                    },
                    Err(e) => {
                        diags.push(Diagnostic {
                            stage: "drc".into(),
                            code: codes::DRC_IO_ERROR.into(),
                            severity: "error".into(),
                            location: Some(SourceLocation {
                                file: temp_drc_json.display().to_string(),
                                line: 1,
                                col: 1,
                            }),
                            subject: Subject {
                                kind: "board".into(),
                                name: Some(board_path.display().to_string()),
                                path: None,
                                id: None,
                                ref_des: None,
                                pad: None,
                            },
                            related: vec![],
                            expected: None,
                            actual: None,
                            fix: None,
                            message: format!("Failed to read DRC report file: {}", e),
                        });
                    }
                }
                let _ = fs::remove_file(temp_drc_json);
            } else if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                diags.push(Diagnostic {
                    stage: "drc".into(),
                    code: codes::DRC_ERROR.into(),
                    severity: "error".into(),
                    location: Some(SourceLocation {
                        file: board_path.display().to_string(),
                        line: 1,
                        col: 1,
                    }),
                    subject: Subject {
                        kind: "board".into(),
                        name: Some(board_path.display().to_string()),
                        path: None,
                        id: None,
                        ref_des: None,
                        pad: None,
                    },
                    related: vec![],
                    expected: None,
                    actual: None,
                    fix: None,
                    message: format!("kicad-cli DRC failed: {}", stderr.trim()),
                });
            } else {
                diags.push(Diagnostic {
                    stage: "drc".into(),
                    code: codes::DRC_REPORT_MISSING.into(),
                    severity: "error".into(),
                    location: Some(SourceLocation {
                        file: board_path.display().to_string(),
                        line: 1,
                        col: 1,
                    }),
                    subject: Subject {
                        kind: "board".into(),
                        name: Some(board_path.display().to_string()),
                        path: None,
                        id: None,
                        ref_des: None,
                        pad: None,
                    },
                    related: vec![],
                    expected: None,
                    actual: None,
                    fix: Some("Verify kicad-cli output path and permissions".into()),
                    message: "kicad-cli completed successfully but did not create the expected DRC JSON report".into(),
                });
            }
        }
        Err(e) => {
            let severity = if is_strict_or_explicit {
                "error"
            } else {
                "warning"
            };
            diags.push(Diagnostic {
                stage: "drc".into(),
                code: codes::KICAD_CLI_NOT_FOUND.into(),
                severity: severity.into(),
                location: Some(SourceLocation {
                    file: board_path.display().to_string(),
                    line: 1,
                    col: 1,
                }),
                subject: Subject {
                    kind: "tool".into(),
                    name: Some("kicad-cli".into()),
                    path: None,
                    id: None,
                    ref_des: None,
                    pad: None,
                },
                related: vec![],
                expected: None,
                actual: None,
                fix: Some("Install KiCad and ensure kicad-cli is on PATH".into()),
                message: format!("kicad-cli could not be executed: {}", e),
            });
        }
    }

    diags
}
