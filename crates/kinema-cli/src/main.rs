use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};
use std::path::PathBuf;
use std::process::ExitCode;

mod commands;
mod doc;

use commands::check::Stage;
use commands::CommandResult;

#[derive(Parser)]
#[command(
    name = "kinema",
    version,
    about = "Circuit Description Language & KiCad Equivalence Verification Tool",
    after_help = "Learn more:\n  Run 'kinema guide' to browse design workflows, syntax, and principles.\n  Run 'kinema explain <CODE>' for diagnostic remediation.\n  Run 'kinema mcp' to launch as an MCP server for AI coding agents."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Clone, Debug)]
pub enum Commands {
    /// Format kinema source files into canonical form (modifies files in-place unless --check is specified)
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
        #[arg(long, alias = "signoff")]
        strict: bool,
        /// Explicit path to KiCad configuration directory containing fp-lib-table and kicad_common.json
        #[arg(long)]
        kicad_config_dir: Option<PathBuf>,
        /// Explicit path to KiCad footprint library directory
        #[arg(long, alias = "footprint-dir")]
        kicad_footprint_dir: Option<PathBuf>,
        /// Explicit path to fp-lib-table file
        #[arg(long)]
        fp_lib_table: Option<PathBuf>,
        /// Additional search directories for footprint libraries
        #[arg(short = 'I', long = "search-path")]
        search_paths: Vec<PathBuf>,
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
    /// Generate KiCad .net netlist (writes to file if -o/--output is specified)
    Netlist {
        #[arg(short, long)]
        output: Option<PathBuf>,
        #[arg(default_value = "")]
        files: Vec<String>,
    },
    /// Generate .kicad_pro netclass config and .kicad_dru custom rules (writes files to disk)
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
    /// View guides, best practices, and documentation topic index
    Guide {
        /// Topic to view (e.g. 'identity', 'connection'). If omitted, prints topic index.
        topic: Option<String>,
        /// Output formatted as JSON
        #[arg(long)]
        json: bool,
        /// Output clean plain text without terminal formatting
        #[arg(long)]
        plain: bool,
    },
    /// Explain a diagnostic rule code or error in detail
    Explain {
        /// Diagnostic rule code to explain (e.g. 'identity-missing', 'decouple-missing')
        code: String,
        /// Output formatted as JSON
        #[arg(long)]
        json: bool,
        /// Output clean plain text without terminal formatting
        #[arg(long)]
        plain: bool,
    },
    /// Start Model Context Protocol (MCP) server over stdio
    Mcp,
}

fn execute_command(command: Commands, default_mode: Option<agent_doc::RenderMode>) -> CommandResult {
    match command {
        Commands::Fmt { check, files } => commands::fmt::run_fmt(&files, check),
        Commands::Check {
            stage,
            json,
            deny_warnings,
            strict,
            kicad_config_dir,
            kicad_footprint_dir,
            fp_lib_table,
            search_paths,
            files,
        } => {
            let mode = if json {
                agent_doc::RenderMode::Json
            } else if let Some(m) = default_mode {
                m
            } else if !std::io::IsTerminal::is_terminal(&std::io::stdout()) {
                agent_doc::RenderMode::Agent
            } else {
                agent_doc::RenderMode::Terminal
            };
            let args = commands::check::CheckArgs {
                stage,
                json,
                mode: Some(mode),
                deny_warnings,
                strict,
                kicad_config_dir,
                kicad_footprint_dir,
                fp_lib_table,
                search_paths,
                files,
            };
            commands::check::run_check(&args)
        }
        Commands::Ir { json, files } => commands::ir::run_ir(&files, json),
        Commands::Netlist { output, files } => commands::netlist::run_netlist(&files, output),
        Commands::Rules { pro, dru, files } => commands::rules::run_rules(&files, pro, dru),
        Commands::Graph { json, files } => commands::graph::run_graph(&files, json),
        Commands::GenLeaf {
            name,
            prefix,
            footprint,
        } => commands::gen_leaf::run_gen_leaf(&name, prefix.as_deref(), footprint.as_deref()),
        Commands::Guide { topic, json, plain } => {
            let doc_engine = doc::build_doc_engine();
            let mode = if json {
                agent_doc::RenderMode::Json
            } else if plain || !std::io::IsTerminal::is_terminal(&std::io::stdout()) {
                agent_doc::RenderMode::Agent
            } else {
                agent_doc::RenderMode::Terminal
            };

            match doc_engine.guide(topic.as_deref(), mode) {
                Ok(content) => CommandResult::success(content),
                Err(e) => CommandResult::error(format!("{}\n", e)),
            }
        }
        Commands::Explain { code, json, plain } => {
            let doc_engine = doc::build_doc_engine();
            let mode = if json {
                agent_doc::RenderMode::Json
            } else if plain || !std::io::IsTerminal::is_terminal(&std::io::stdout()) {
                agent_doc::RenderMode::Agent
            } else {
                agent_doc::RenderMode::Terminal
            };

            match doc_engine.explain(&code, mode) {
                Ok(content) => CommandResult::success(content),
                Err(e) => CommandResult::error(format!("{}\n", e)),
            }
        }
        Commands::Mcp => {
            // Handled separately in main
            unreachable!("Mcp subcommand is handled directly in main")
        }
    }
}

fn run_mcp() -> ExitCode {
    let mut engine = doc::build_doc_engine();
    let cmd = Cli::command();

    if let Err(e) = engine.register_clap_tools(cmd, &["guide", "explain", "mcp"], |_name, matches| {
        let res = match Cli::from_arg_matches(&matches) {
            Ok(cli) => execute_command(cli.command, Some(agent_doc::RenderMode::Mcp)),
            Err(e) => commands::CommandResult::error(e.to_string()),
        };

        res.to_tool_result().into()
    }) {
        eprintln!("Error registering MCP tools: {}", e);
        return ExitCode::from(1);
    }

    for tool in &mut engine.tools {
        match tool.name.as_str() {
            "check" => tool.see_page = Some("workflow".into()),
            "fmt" => tool.see_page = Some("syntax".into()),
            "ir" => tool.see_page = Some("syntax".into()),
            "netlist" => tool.see_page = Some("workflow".into()),
            "rules" => tool.see_page = Some("syntax".into()),
            "graph" => tool.see_page = Some("workflow".into()),
            "gen-leaf" => tool.see_page = Some("syntax".into()),
            _ => {}
        }
    }

    if let Err(e) = engine.serve_stdio() {
        eprintln!("Error running MCP server: {}", e);
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    if matches!(cli.command, Commands::Mcp) {
        return run_mcp();
    }

    let result = execute_command(cli.command, None);
    result.print();
    result.exit_code()
}
