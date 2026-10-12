use super::common::{format_report, load_source_files, resolve_project_sources};
use super::CommandResult;
use kinema_elab::elaborate_sources;
use kinema_kicad::{generate_kicad_dru, merge_kicad_pro};
use std::fs;
use std::path::{Path, PathBuf};

pub fn run_rules(
    files: &[String],
    pro: Option<PathBuf>,
    dru: Option<PathBuf>,
) -> CommandResult {
    let (paths, _) = resolve_project_sources(files);
    let source_files = match load_source_files(&paths) {
        Ok(f) => f,
        Err(report) => {
            let output = format_report(&report, agent_man::RenderMode::Json);
            let data = serde_json::to_value(&report).ok();
            let mut res = CommandResult::error(output);
            res.data = data;
            return res;
        }
    };

    match elaborate_sources(&source_files) {
        Ok(ir) => {
            let project_dir = paths
                .first()
                .and_then(|p| p.parent())
                .unwrap_or(Path::new("."));

            let pro_target = if let Some(p) = pro {
                p
            } else {
                let mut found_pro = None;
                if let Ok(entries) = fs::read_dir(project_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path
                            .extension()
                            .and_then(|ext| ext.to_str())
                            == Some("kicad_pro")
                        {
                            found_pro = Some(path);
                            break;
                        }
                    }
                }
                found_pro.unwrap_or_else(|| {
                    project_dir.join(format!("{}.kicad_pro", ir.top_module))
                })
            };

            let dru_target = if let Some(d) = dru {
                d
            } else {
                pro_target.with_extension("kicad_dru")
            };

            let existing_pro_content = fs::read_to_string(&pro_target).ok();
            let pro_content = match merge_kicad_pro(&ir, existing_pro_content.as_deref()) {
                Ok(c) => c,
                Err(e) => {
                    return CommandResult::error(format!(
                        "Failed to merge project file {}: {}\n",
                        pro_target.display(),
                        e
                    ));
                }
            };
            let dru_content = generate_kicad_dru(&ir);

            if let Err(e) = fs::write(&pro_target, pro_content) {
                return CommandResult::error(format!(
                    "Failed to write {}: {}\n",
                    pro_target.display(),
                    e
                ));
            }
            if let Err(e) = fs::write(&dru_target, dru_content) {
                return CommandResult::error(format!(
                    "Failed to write {}: {}\n",
                    dru_target.display(),
                    e
                ));
            }

            CommandResult::success(format!(
                "Generated rules:\n  {}\n  {}\n",
                pro_target.display(),
                dru_target.display()
            ))
        }
        Err(e) => CommandResult::error(format!("Elaboration error: {}\n", e)),
    }
}
