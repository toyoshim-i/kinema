//! Diagnostic code constants and comprehensive list across kinema.

pub const BOARD_MISSING: &str = "board-missing";
pub const COMPONENT_EXTRA: &str = "component-extra";
pub const COMPONENT_MISSING: &str = "component-missing";
pub const CONSTRAINT_CONFLICT: &str = "constraint-conflict";
pub const DECOUPLE_MISSING: &str = "decouple-missing";
pub const DECOUPLE_MULTI_PAD: &str = "decouple-multi-pad";
pub const DIFFPAIR_INVALID: &str = "diffpair-invalid";
pub const DRC_ERROR: &str = "drc-error";
pub const DRC_IO_ERROR: &str = "drc-io-error";
pub const DRC_JSON_ERROR: &str = "drc-json-error";
pub const DRC_REPORT_MISSING: &str = "drc-report-missing";
pub const DUPLICATE_IDENTITY: &str = "duplicate-identity";
pub const DUPLICATE_MODULE: &str = "duplicate-module";
pub const ELAB_ERROR: &str = "elab-error";
pub const FIELD_MISMATCH: &str = "field-mismatch";
pub const FOOTPRINT_MISMATCH: &str = "footprint-mismatch";
pub const FOOTPRINT_MISSING: &str = "footprint-missing";
pub const HUB_DIRECT_PAD: &str = "hub-direct-pad";
pub const HUB_TOO_FEW: &str = "hub-too-few";
pub const IDENTITY_MISSING: &str = "identity-missing";
pub const IO_ERROR: &str = "io-error";
pub const JOIN_CYCLE: &str = "join-cycle";
pub const JOIN_NAME: &str = "join-name";
pub const KICAD_CLI_NOT_FOUND: &str = "kicad-cli-not-found";
pub const LEAF_INCOMPLETE: &str = "leaf-incomplete";
pub const MISSING_PORT: &str = "missing-port";
pub const NC_CONNECTED: &str = "nc-connected";
pub const NC_PAD_CONNECTED: &str = "nc-pad-connected";
pub const NET_EXTRA: &str = "net-extra";
pub const NET_NAME_MISMATCH: &str = "net-name-mismatch";
pub const NET_PARTITION_MISMATCH: &str = "net-partition-mismatch";
pub const NOT_FORMATTED: &str = "not-formatted";
pub const PAD_COUNT: &str = "pad-count";
pub const PAD_DUPLICATE: &str = "pad-duplicate";
pub const PARAM_MISSING: &str = "param-missing";
pub const PCB_PARSE_ERROR: &str = "pcb-parse-error";
pub const PIN_WIRE_ATTR: &str = "pin-wire-attr";
pub const PIN_WIRE_NAME: &str = "pin-wire-name";
pub const PIN_WIRE_SHAPE: &str = "pin-wire-shape";
pub const POWER_CONFLICT: &str = "power-conflict";
pub const POWER_UNCONNECTED: &str = "power-unconnected";
pub const RECURSIVE_INSTANCE: &str = "recursive-instance";
pub const REF_MISMATCH: &str = "ref-mismatch";
pub const SINGLE_PIN_NET: &str = "single-pin-net";
pub const SYNTAX_ERROR: &str = "syntax-error";
pub const TOP_MODULE: &str = "top-module";
pub const UNDECLARED_NET: &str = "undeclared-net";
pub const UNDRIVEN_NET: &str = "undriven-net";
pub const UNIT_INVALID: &str = "unit-invalid";
pub const UNKNOWN_ATTR: &str = "unknown-attr";
pub const UNKNOWN_PARAM: &str = "unknown-param";
pub const WIDTH_MISMATCH: &str = "width-mismatch";

/// All diagnostic codes produced by the kinema verification pipeline.
pub const ALL: &[&str] = &[
    BOARD_MISSING,
    COMPONENT_EXTRA,
    COMPONENT_MISSING,
    CONSTRAINT_CONFLICT,
    DECOUPLE_MISSING,
    DECOUPLE_MULTI_PAD,
    DIFFPAIR_INVALID,
    DRC_ERROR,
    DRC_IO_ERROR,
    DRC_JSON_ERROR,
    DRC_REPORT_MISSING,
    DUPLICATE_IDENTITY,
    DUPLICATE_MODULE,
    ELAB_ERROR,
    FIELD_MISMATCH,
    FOOTPRINT_MISMATCH,
    FOOTPRINT_MISSING,
    HUB_DIRECT_PAD,
    HUB_TOO_FEW,
    IDENTITY_MISSING,
    IO_ERROR,
    JOIN_CYCLE,
    JOIN_NAME,
    KICAD_CLI_NOT_FOUND,
    LEAF_INCOMPLETE,
    MISSING_PORT,
    NC_CONNECTED,
    NC_PAD_CONNECTED,
    NET_EXTRA,
    NET_NAME_MISMATCH,
    NET_PARTITION_MISMATCH,
    NOT_FORMATTED,
    PAD_COUNT,
    PAD_DUPLICATE,
    PARAM_MISSING,
    PCB_PARSE_ERROR,
    PIN_WIRE_ATTR,
    PIN_WIRE_NAME,
    PIN_WIRE_SHAPE,
    POWER_CONFLICT,
    POWER_UNCONNECTED,
    RECURSIVE_INSTANCE,
    REF_MISMATCH,
    SINGLE_PIN_NET,
    SYNTAX_ERROR,
    TOP_MODULE,
    UNDECLARED_NET,
    UNDRIVEN_NET,
    UNIT_INVALID,
    UNKNOWN_ATTR,
    UNKNOWN_PARAM,
    WIDTH_MISMATCH,
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};

    fn collect_rs_files(dir: &Path, files: &mut Vec<PathBuf>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = entry.file_name();
                    if name != "target" && name != ".git" {
                        collect_rs_files(&path, files);
                    }
                } else if path.extension().is_some_and(|ext| ext == "rs") {
                    files.push(path);
                }
            }
        }
    }

    #[test]
    fn test_all_codes_unique_and_count() {
        assert_eq!(ALL.len(), 52, "Must contain exactly 52 diagnostic codes");
        let mut set = std::collections::HashSet::new();
        for &code in ALL {
            assert!(set.insert(code), "Duplicate code in ALL: {}", code);
        }
    }

    #[test]
    fn test_no_raw_code_literals_outside_codes_module() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let workspace_root = manifest_dir.parent().unwrap().parent().unwrap();
        let crates_dir = workspace_root.join("crates");

        let mut rs_files = Vec::new();
        collect_rs_files(&crates_dir, &mut rs_files);

        let mut violations = Vec::new();
        for path in rs_files {
            // Skip the codes.rs file itself
            if path.ends_with("kinema-lint/src/codes.rs") {
                continue;
            }

            let content = fs::read_to_string(&path).expect("failed to read rs file");
            for (line_no, line) in content.lines().enumerate() {
                // Check for patterns like `code: "` or `code: r#"`
                if let Some(pos) = line.find("code:") {
                    let after = line[pos + 5..].trim_start();
                    if after.starts_with('"') || after.starts_with("r#\"") {
                        violations.push(format!(
                            "{}:{}: {}",
                            path.strip_prefix(workspace_root).unwrap_or(&path).display(),
                            line_no + 1,
                            line.trim()
                        ));
                    }
                }
            }
        }

        assert!(
            violations.is_empty(),
            "Found raw 'code: \"...' literals outside codes.rs! Use kinema_lint::codes constants instead:\n{}",
            violations.join("\n")
        );
    }
}
