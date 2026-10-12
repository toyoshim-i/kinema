use super::common::{format_report, load_source_files, resolve_project_sources};
use super::CommandResult;
use kinema_elab::elaborate_sources;
use kinema_kicad::generate_kicad_netlist;
use std::fs;
use std::path::PathBuf;

pub fn run_netlist(files: &[String], output: Option<PathBuf>) -> CommandResult {
    let (paths, _) = resolve_project_sources(files);
    let source_files = match load_source_files(&paths) {
        Ok(f) => f,
        Err(report) => {
            let output = format_report(&report, true);
            let data = serde_json::to_value(&report).ok();
            let mut res = CommandResult::error(output);
            res.data = data;
            return res;
        }
    };

    match elaborate_sources(&source_files) {
        Ok(ir) => {
            let netlist_str = generate_kicad_netlist(&ir);
            if let Some(out_path) = output {
                if let Some(parent) = out_path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                if let Err(e) = fs::write(&out_path, &netlist_str) {
                    return CommandResult::error(format!(
                        "Failed to write netlist to {}: {}\n",
                        out_path.display(),
                        e
                    ));
                }
                CommandResult::success(format!("Netlist written to {}\n", out_path.display()))
            } else {
                CommandResult::success(netlist_str)
            }
        }
        Err(e) => CommandResult::error(format!("Elaboration error: {}\n", e)),
    }
}
