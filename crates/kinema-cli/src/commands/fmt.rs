use super::common::resolve_project_sources;
use super::CommandResult;
use kinema_fmt::format_str;
use std::fs;

pub fn run_fmt(files: &[String], check: bool) -> CommandResult {
    let (paths, _) = resolve_project_sources(files);
    let mut has_unformatted = false;
    let mut stdout = String::new();
    let mut stderr = String::new();

    for path in paths {
        if let Ok(content) = fs::read_to_string(&path) {
            match format_str(&path.display().to_string(), &content) {
                Ok(formatted) => {
                    if content != formatted {
                        has_unformatted = true;
                        if check {
                            stderr.push_str(&format!("File not formatted: {}\n", path.display()));
                        } else {
                            let _ = fs::write(&path, formatted);
                            stdout.push_str(&format!("Formatted: {}\n", path.display()));
                        }
                    }
                }
                Err(e) => {
                    stderr.push_str(&format!("Error formatting {}: {}\n", path.display(), e));
                    return CommandResult {
                        stdout,
                        stderr,
                        success: false,
                        data: None,
                    };
                }
            }
        }
    }

    if check && has_unformatted {
        CommandResult {
            stdout,
            stderr,
            success: false,
            data: None,
        }
    } else {
        CommandResult {
            stdout,
            stderr,
            success: true,
            data: None,
        }
    }
}
