//! Diagnostic code constants and comprehensive list across kinema.

macro_rules! define_diagnostic_codes {
    ($( $(#[$meta:meta])* $name:ident => $val:literal ),* $(,)?) => {
        $(
            $(#[$meta])*
            pub const $name: &str = $val;
        )*

        /// All diagnostic codes produced by the kinema verification pipeline.
        pub const ALL: &[&str] = &[
            $( $name, )*
        ];
    };
}

define_diagnostic_codes! {
    BOARD_MISSING => "board-missing",
    COMPONENT_EXTRA => "component-extra",
    COMPONENT_MISSING => "component-missing",
    CONSTRAINT_CONFLICT => "constraint-conflict",
    DECOUPLE_MISSING => "decouple-missing",
    DECOUPLE_MULTI_PAD => "decouple-multi-pad",
    DIFFPAIR_INVALID => "diffpair-invalid",
    DRC_ERROR => "drc-error",
    DRC_IO_ERROR => "drc-io-error",
    DRC_JSON_ERROR => "drc-json-error",
    DRC_REPORT_MISSING => "drc-report-missing",
    DUPLICATE_IDENTITY => "duplicate-identity",
    DUPLICATE_MODULE => "duplicate-module",
    ELAB_ERROR => "elab-error",
    FIELD_MISMATCH => "field-mismatch",
    FOOTPRINT_MISMATCH => "footprint-mismatch",
    FOOTPRINT_MISSING => "footprint-missing",
    HUB_DIRECT_PAD => "hub-direct-pad",
    HUB_TOO_FEW => "hub-too-few",
    IDENTITY_MISSING => "identity-missing",
    IO_ERROR => "io-error",
    JOIN_CYCLE => "join-cycle",
    JOIN_NAME => "join-name",
    KICAD_CLI_NOT_FOUND => "kicad-cli-not-found",
    LEAF_INCOMPLETE => "leaf-incomplete",
    MISSING_PORT => "missing-port",
    NC_CONNECTED => "nc-connected",
    NC_PAD_CONNECTED => "nc-pad-connected",
    NET_EXTRA => "net-extra",
    NET_NAME_MISMATCH => "net-name-mismatch",
    NET_PARTITION_MISMATCH => "net-partition-mismatch",
    NOT_FORMATTED => "not-formatted",
    PAD_COUNT => "pad-count",
    PAD_DUPLICATE => "pad-duplicate",
    PARAM_MISSING => "param-missing",
    PCB_PARSE_ERROR => "pcb-parse-error",
    PIN_WIRE_ATTR => "pin-wire-attr",
    PIN_WIRE_NAME => "pin-wire-name",
    PIN_WIRE_SHAPE => "pin-wire-shape",
    POWER_CONFLICT => "power-conflict",
    POWER_UNCONNECTED => "power-unconnected",
    RECURSIVE_INSTANCE => "recursive-instance",
    REF_MISMATCH => "ref-mismatch",
    SINGLE_PIN_NET => "single-pin-net",
    SYNTAX_ERROR => "syntax-error",
    TOP_MODULE => "top-module",
    UNDECLARED_NET => "undeclared-net",
    UNDRIVEN_NET => "undriven-net",
    UNIT_INVALID => "unit-invalid",
    UNKNOWN_ATTR => "unknown-attr",
    UNKNOWN_PARAM => "unknown-param",
    WIDTH_MISMATCH => "width-mismatch",
}

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
