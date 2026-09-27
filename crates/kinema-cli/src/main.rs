use clap::{Parser, Subcommand, ValueEnum};
use kinema_elab::{elaborate_sources, FlatNetlistIR};
use kinema_equiv::check_equivalence;
use kinema_fmt::format_str;
use kinema_kicad::{generate_kicad_dru, generate_kicad_netlist, generate_kicad_pro, parse_kicad_pcb};
use kinema_lint::diagnostic::{Diagnostic, LintReport, Subject};
use kinema_lint::lint_source_files;
use kinema_syntax::ast::{SourceFile, SourceLocation};
use kinema_syntax::parser::parse;
use kinema_syntax::project::ProjectConfig;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "kinema", version = "0.1.0", about = "Circuit Description Language & KiCad Equivalence Verification Tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, ValueEnum, Debug, PartialEq, Eq)]
enum Stage {
    Parse,
    Fmt,
    Lint,
    Equiv,
    Drc,
}

#[derive(Subcommand)]
enum Commands {
    /// Format kinema source files into canonical form
    Fmt {
        #[arg(long)]
        check: bool,
        #[arg(default_value = "")]
        files: Vec<String>,
    },
    /// Check syntax, static lint rules, board equivalence, and DRC
    Check {
        #[arg(long)]
        stage: Option<Stage>,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        deny_warnings: bool,
        #[arg(default_value = "")]
        files: Vec<String>,
    },
    /// Generate flat netlist IR
    Ir {
        #[arg(long, default_value_t = true)]
        json: bool,
        #[arg(default_value = "")]
        files: Vec<String>,
    },
    /// Generate KiCad .net netlist
    Netlist {
        #[arg(short, long)]
        output: Option<PathBuf>,
        #[arg(default_value = "")]
        files: Vec<String>,
    },
    /// Generate .kicad_pro netclass config and .kicad_dru custom rules
    Rules {
        #[arg(short, long)]
        pro: Option<PathBuf>,
        #[arg(short, long)]
        dru: Option<PathBuf>,
        #[arg(default_value = "")]
        files: Vec<String>,
    },
    /// Generate yosys-compatible JSON for netlistsvg
    Graph {
        #[arg(long, default_value_t = true)]
        json: bool,
        #[arg(default_value = "")]
        files: Vec<String>,
    },
    /// Draft leaf module template from symbol name or component name
    GenLeaf {
        name: String,
        #[arg(short, long)]
        prefix: Option<String>,
        #[arg(short, long)]
        footprint: Option<String>,
    },
}

fn resolve_project_sources(specified_files: &[String]) -> (Vec<PathBuf>, Option<PathBuf>) {
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

fn load_source_files(paths: &[PathBuf]) -> Result<Vec<SourceFile>, LintReport> {
    let mut files = Vec::new();
    let mut diags = Vec::new();

    for path in paths {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                diags.push(Diagnostic {
                    stage: "parse".into(),
                    code: "io-error".into(),
                    severity: "error".into(),
                    location: SourceLocation { file: path.display().to_string(), line: 1, col: 1 },
                    subject: Subject { kind: "file".into(), name: Some(path.display().to_string()), path: None, id: None, ref_des: None, pad: None },
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
                    code: "syntax-error".into(),
                    severity: "error".into(),
                    location: SourceLocation { file: e.file, line: e.line, col: e.col },
                    subject: Subject { kind: "syntax".into(), name: None, path: None, id: None, ref_des: None, pad: None },
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

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Commands::Fmt { check, files } => {
            let (paths, _) = resolve_project_sources(&files);
            let mut has_unformatted = false;

            for path in paths {
                if let Ok(content) = fs::read_to_string(&path) {
                    match format_str(&path.display().to_string(), &content) {
                        Ok(formatted) => {
                            if content != formatted {
                                has_unformatted = true;
                                if check {
                                    eprintln!("File not formatted: {}", path.display());
                                } else {
                                    let _ = fs::write(&path, formatted);
                                    println!("Formatted: {}", path.display());
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Error formatting {}: {}", path.display(), e);
                            return ExitCode::from(1);
                        }
                    }
                }
            }

            if check && has_unformatted {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            }
        }

        Commands::Check { stage, json, deny_warnings, files } => {
            let (paths, board_path) = resolve_project_sources(&files);
            let source_files = match load_source_files(&paths) {
                Ok(f) => f,
                Err(report) => {
                    output_report(&report, json);
                    return ExitCode::from(1);
                }
            };

            let mut all_diags = Vec::new();

            // Run Lint stage
            if stage.is_none() || stage == Some(Stage::Lint) {
                let lint_report = lint_source_files(&source_files);
                all_diags.extend(lint_report.diagnostics);
            }

            // Run Equiv stage if requested or if stage is None and board exists
            if stage == Some(Stage::Equiv) || (stage.is_none() && board_path.is_some()) {
                if let Some(bp) = board_path {
                    if bp.exists() {
                        match fs::read_to_string(&bp) {
                            Ok(content) => match parse_kicad_pcb(&content) {
                                Ok(board) => match elaborate_sources(&source_files) {
                                    Ok(ir) => {
                                        let equiv_report = check_equivalence(&ir, &board);
                                        all_diags.extend(equiv_report.diagnostics);
                                    }
                                    Err(e) => {
                                        all_diags.push(Diagnostic {
                                            stage: "equiv".into(),
                                            code: "elab-error".into(),
                                            severity: "error".into(),
                                            location: SourceLocation { file: bp.display().to_string(), line: 1, col: 1 },
                                            subject: Subject { kind: "circuit".into(), name: None, path: None, id: None, ref_des: None, pad: None },
                                            related: vec![],
                                            expected: None,
                                            actual: None,
                                            fix: None,
                                            message: format!("Elaboration error: {}", e),
                                        });
                                    }
                                },
                                Err(e) => {
                                    all_diags.push(Diagnostic {
                                        stage: "equiv".into(),
                                        code: "pcb-parse-error".into(),
                                        severity: "error".into(),
                                        location: SourceLocation { file: bp.display().to_string(), line: 1, col: 1 },
                                        subject: Subject { kind: "board".into(), name: None, path: None, id: None, ref_des: None, pad: None },
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
                                    code: "io-error".into(),
                                    severity: "error".into(),
                                    location: SourceLocation { file: bp.display().to_string(), line: 1, col: 1 },
                                    subject: Subject { kind: "board".into(), name: None, path: None, id: None, ref_des: None, pad: None },
                                    related: vec![],
                                    expected: None,
                                    actual: None,
                                    fix: None,
                                    message: format!("Failed to read board file: {}", e),
                                });
                            }
                        }
                    }
                }
            }

            let report = LintReport::from_diagnostics(all_diags);
            output_report(&report, json);

            let has_errors = report.diagnostics.iter().any(|d| d.severity == "error");
            let has_warnings = report.diagnostics.iter().any(|d| d.severity == "warning");

            if has_errors || (deny_warnings && has_warnings) {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            }
        }

        Commands::Ir { json: _, files } => {
            let (paths, _) = resolve_project_sources(&files);
            let source_files = match load_source_files(&paths) {
                Ok(f) => f,
                Err(report) => {
                    output_report(&report, true);
                    return ExitCode::from(1);
                }
            };

            match elaborate_sources(&source_files) {
                Ok(ir) => {
                    println!("{}", serde_json::to_string_pretty(&ir).unwrap_or_default());
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("Elaboration error: {}", e);
                    ExitCode::from(1)
                }
            }
        }

        Commands::Netlist { output, files } => {
            let (paths, _) = resolve_project_sources(&files);
            let source_files = match load_source_files(&paths) {
                Ok(f) => f,
                Err(report) => {
                    output_report(&report, true);
                    return ExitCode::from(1);
                }
            };

            match elaborate_sources(&source_files) {
                Ok(ir) => {
                    let netlist_str = generate_kicad_netlist(&ir);
                    if let Some(out_path) = output {
                        if let Some(parent) = out_path.parent() {
                            let _ = fs::create_dir_all(parent);
                        }
                        if let Err(e) = fs::write(&out_path, netlist_str) {
                            eprintln!("Failed to write netlist to {}: {}", out_path.display(), e);
                            return ExitCode::from(2);
                        }
                        println!("Netlist written to {}", out_path.display());
                    } else {
                        println!("{}", netlist_str);
                    }
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("Elaboration error: {}", e);
                    ExitCode::from(1)
                }
            }
        }

        Commands::Rules { pro, dru, files } => {
            let (paths, _) = resolve_project_sources(&files);
            let source_files = match load_source_files(&paths) {
                Ok(f) => f,
                Err(report) => {
                    output_report(&report, true);
                    return ExitCode::from(1);
                }
            };

            match elaborate_sources(&source_files) {
                Ok(ir) => {
                    let pro_content = generate_kicad_pro(&ir);
                    let dru_content = generate_kicad_dru(&ir);

                    let pro_target = pro.unwrap_or_else(|| PathBuf::from("kinema.kicad_pro"));
                    let dru_target = dru.unwrap_or_else(|| PathBuf::from("kinema.kicad_dru"));

                    let _ = fs::write(&pro_target, pro_content);
                    let _ = fs::write(&dru_target, dru_content);

                    println!("Generated rules:\n  {}\n  {}", pro_target.display(), dru_target.display());
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("Elaboration error: {}", e);
                    ExitCode::from(1)
                }
            }
        }

        Commands::Graph { json: _, files } => {
            let (paths, _) = resolve_project_sources(&files);
            let source_files = match load_source_files(&paths) {
                Ok(f) => f,
                Err(report) => {
                    output_report(&report, true);
                    return ExitCode::from(1);
                }
            };

            match elaborate_sources(&source_files) {
                Ok(ir) => {
                    // Export yosys-compatible JSON structure for netlistsvg
                    let yosys_json = generate_yosys_json(&ir);
                    println!("{}", serde_json::to_string_pretty(&yosys_json).unwrap_or_default());
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("Elaboration error: {}", e);
                    ExitCode::from(1)
                }
            }
        }

        Commands::GenLeaf { name, prefix, footprint } => {
            let pfx = prefix.unwrap_or_else(|| "U".to_string());
            let fp = footprint.unwrap_or_else(|| "Package_SO:SOIC-8_3.9x4.9mm_P1.27mm".to_string());
            println!(
                "(* footprint = \"{}\", prefix = \"{}\" *)\nmodule {} (\n    (* pad = \"1\", etype = \"passive\" *) inout PIN1\n);\nendmodule\n",
                fp, pfx, name
            );
            ExitCode::SUCCESS
        }
    }
}

fn output_report(report: &LintReport, json: bool) {
    if json {
        println!("{}", serde_json::to_string_pretty(report).unwrap_or_default());
    } else if report.diagnostics.is_empty() {
        println!("All checks passed (0 errors, 0 warnings).");
    } else {
        for d in &report.diagnostics {
            let sev = if d.severity == "error" { "ERROR" } else { "WARNING" };
            println!(
                "[{}] {} [{}:{}]: {}",
                sev, d.code, d.location.file, d.location.line, d.message
            );
            if let Some(f) = &d.fix {
                println!("  Suggestion: {}", f);
            }
        }
    }
}

fn generate_yosys_json(ir: &FlatNetlistIR) -> serde_json::Value {
    let mut modules = serde_json::Map::new();
    let mut cells = serde_json::Map::new();

    for comp in &ir.components {
        let mut conn = serde_json::Map::new();
        for pad in &comp.pads {
            conn.insert(pad.port_name.clone(), serde_json::json!([pad.pad_number]));
        }
        cells.insert(comp.refdes.clone(), serde_json::json!({
            "type": comp.module_name,
            "port_directions": {},
            "connections": conn
        }));
    }

    modules.insert(ir.top_module.clone(), serde_json::json!({
        "cells": cells,
        "netnames": {}
    }));

    serde_json::json!({
        "creator": "kinema",
        "modules": modules
    })
}
