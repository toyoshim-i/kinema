use std::path::Path;
use std::process::Command;

fn get_workspace_root() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../"))
}

#[test]
fn test_cli_fmt_check() {
    let output = Command::new(env!("CARGO_BIN_EXE_kinema"))
        .current_dir(get_workspace_root())
        .args(["fmt", "examples/timer_core.v", "--check"])
        .output()
        .expect("failed to execute kinema fmt");
    assert!(output.status.success(), "kinema fmt --check must exit 0: {:?}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn test_cli_check_lint_json() {
    let output = Command::new(env!("CARGO_BIN_EXE_kinema"))
        .current_dir(get_workspace_root())
        .args(["check", "--stage", "lint", "--json", "examples/timer_core.v"])
        .output()
        .expect("failed to execute kinema check");
    assert!(output.status.success(), "kinema check must exit 0: {:?}", String::from_utf8_lossy(&output.stderr));
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("output must be valid JSON");
    assert_eq!(parsed["ok"], true);
}

#[test]
fn test_cli_check_stage_fmt_clean() {
    let output = Command::new(env!("CARGO_BIN_EXE_kinema"))
        .current_dir(get_workspace_root())
        .args(["check", "--stage", "fmt", "--json", "examples/timer_core.v"])
        .output()
        .expect("failed to execute kinema check");
    assert!(output.status.success(), "kinema check --stage fmt on clean file must exit 0: {:?}", String::from_utf8_lossy(&output.stderr));
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("output must be valid JSON");
    assert_eq!(parsed["ok"], true);
}

#[test]
fn test_cli_check_stage_fmt_unformatted() {
    let unformatted_path = get_workspace_root().join("target/unformatted_test.v");
    std::fs::write(&unformatted_path, "module  bad_indent  () ;\nendmodule\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_kinema"))
        .current_dir(get_workspace_root())
        .args(["check", "--stage", "fmt", "--json", "target/unformatted_test.v"])
        .output()
        .expect("failed to execute kinema check");
    assert!(!output.status.success(), "kinema check --stage fmt on unformatted file must exit non-zero");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("output must be valid JSON");
    assert_eq!(parsed["ok"], false);
    let diags = parsed["diagnostics"].as_array().unwrap();
    assert!(diags.iter().any(|d| d["stage"] == "fmt" && d["code"] == "not-formatted"));

    let _ = std::fs::remove_file(unformatted_path);
}

#[test]
fn test_cli_check_stage_drc_missing_board() {
    let output = Command::new(env!("CARGO_BIN_EXE_kinema"))
        .current_dir(get_workspace_root())
        .args(["check", "--stage", "drc", "--json", "non_existent_board.kicad_pcb"])
        .output()
        .expect("failed to execute kinema check");
    assert!(!output.status.success(), "kinema check --stage drc on non-existent board must exit non-zero");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("output must be valid JSON");
    assert_eq!(parsed["ok"], false);
    let diags = parsed["diagnostics"].as_array().unwrap();
    assert!(diags.iter().any(|d| d["stage"] == "drc" && d["code"] == "board-missing"));
}

#[test]
fn test_cli_netlist_export() {
    let out_path = "target/cli_test_timer.net";
    let output = Command::new(env!("CARGO_BIN_EXE_kinema"))
        .current_dir(get_workspace_root())
        .args(["netlist", "-o", out_path, "examples/timer_core.v"])
        .output()
        .expect("failed to execute kinema netlist");
    assert!(output.status.success(), "kinema netlist must exit 0: {:?}", String::from_utf8_lossy(&output.stderr));

    let abs_out = get_workspace_root().join(out_path);
    let content = std::fs::read_to_string(abs_out).expect("read generated netlist");
    assert!(content.contains("(export (version \"E\")"));
    assert!(content.contains("(comp (ref \"U1\")"));
    assert!(content.contains("(net (code \""));
}

#[test]
fn test_cli_ir_json() {
    let output = Command::new(env!("CARGO_BIN_EXE_kinema"))
        .current_dir(get_workspace_root())
        .args(["ir", "--json", "examples/timer_core.v"])
        .output()
        .expect("failed to execute kinema ir");
    assert!(output.status.success(), "kinema ir must exit 0: {:?}", String::from_utf8_lossy(&output.stderr));
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("output must be valid JSON");
    assert_eq!(parsed["top_module"], "timer_core");
    assert_eq!(parsed["components"].as_array().unwrap().len(), 3);
}

#[test]
fn test_cli_rules() {
    let temp_dir = std::env::temp_dir();
    let pro_file = temp_dir.join("test_rules.kicad_pro");
    let dru_file = temp_dir.join("test_rules.kicad_dru");
    let output = Command::new(env!("CARGO_BIN_EXE_kinema"))
        .current_dir(get_workspace_root())
        .args([
            "rules",
            "--pro",
            pro_file.to_str().unwrap(),
            "--dru",
            dru_file.to_str().unwrap(),
        ])
        .output()
        .expect("failed to execute kinema rules");
    assert!(output.status.success(), "kinema rules must exit 0: {:?}", String::from_utf8_lossy(&output.stderr));
    let _ = std::fs::remove_file(pro_file);
    let _ = std::fs::remove_file(dru_file);
}

#[test]
fn test_cli_graph() {
    let output = Command::new(env!("CARGO_BIN_EXE_kinema"))
        .current_dir(get_workspace_root())
        .args(["graph", "examples/timer_core.v"])
        .output()
        .expect("failed to execute kinema graph");
    assert!(output.status.success(), "kinema graph must exit 0: {:?}", String::from_utf8_lossy(&output.stderr));
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("output must be valid JSON");
    assert_eq!(parsed["creator"], "kinema");

    let top = &parsed["modules"]["timer_core"];
    let netnames = &top["netnames"];
    assert!(netnames.get("VCC").is_some(), "VCC net must exist in netnames");
    assert!(netnames.get("GND").is_some(), "GND net must exist in netnames");

    let u1_vcc_bits = &top["cells"]["U1"]["connections"]["VCC"];
    let c1_a_bits = &top["cells"]["C1"]["connections"]["A"];
    assert_eq!(u1_vcc_bits, c1_a_bits, "U1.VCC and C1.A must share the same bit ID for VCC net");

    let u1_gnd_bits = &top["cells"]["U1"]["connections"]["GND"];
    let c1_b_bits = &top["cells"]["C1"]["connections"]["B"];
    assert_eq!(u1_gnd_bits, c1_b_bits, "U1.GND and C1.B must share the same bit ID for GND net");

    assert_ne!(u1_vcc_bits, u1_gnd_bits, "VCC and GND must have distinct bit IDs");
}

#[test]
fn test_cli_gen_leaf() {
    let output = Command::new(env!("CARGO_BIN_EXE_kinema"))
        .current_dir(get_workspace_root())
        .args(["gen-leaf", "ATmega328P", "--prefix", "U", "--footprint", "Package_QFP:TQFP-32_7x7mm_P0.8mm"])
        .output()
        .expect("failed to execute kinema gen-leaf");
    assert!(output.status.success(), "kinema gen-leaf must exit 0: {:?}", String::from_utf8_lossy(&output.stderr));
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("module ATmega328P"));
    assert!(stdout.contains("Package_QFP:TQFP-32_7x7mm_P0.8mm"));
}
