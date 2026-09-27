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
    let output = Command::new(env!("CARGO_BIN_EXE_kinema"))
        .current_dir(get_workspace_root())
        .args(["rules"])
        .output()
        .expect("failed to execute kinema rules");
    assert!(output.status.success(), "kinema rules must exit 0: {:?}", String::from_utf8_lossy(&output.stderr));
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
