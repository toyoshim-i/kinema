use super::common::{format_report, load_source_files, resolve_project_sources};
use super::CommandResult;
use kinema_elab::elaborate_sources;

pub fn run_ir(files: &[String], _json: bool) -> CommandResult {
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
            let json_str = format!("{}\n", serde_json::to_string_pretty(&ir).unwrap_or_default());
            let data = serde_json::to_value(&ir).ok();
            let mut res = CommandResult::success(json_str);
            res.data = data;
            res
        }
        Err(e) => CommandResult::error(format!("Elaboration error: {}\n", e)),
    }
}
