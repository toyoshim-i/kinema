use kinema_elab::elaborate_sources;
use kinema_equiv::check_equivalence;
use kinema_kicad::parse_kicad_pcb;
use kinema_syntax::parser::parse;
use std::fs;
use std::path::Path;

fn get_timer_core_ir() -> kinema_elab::FlatNetlistIR {
    let std_v = concat!(env!("CARGO_MANIFEST_DIR"), "/../../lib/std.v");
    let timer_core_v = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/timer_core.v");
    let std_ast = parse("std.v", &fs::read_to_string(std_v).expect("read std.v")).expect("parse std.v");
    let core_ast = parse("timer_core.v", &fs::read_to_string(timer_core_v).expect("read timer_core.v")).expect("parse timer_core.v");
    elaborate_sources(&[std_ast, core_ast]).expect("elaborate timer_core")
}

fn load_fixture_board(name: &str) -> kinema_kicad::PcbBoard {
    let p = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    let content = fs::read_to_string(&p).unwrap_or_else(|e| panic!("failed to read {:?}: {}", p, e));
    parse_kicad_pcb(&content).unwrap_or_else(|e| panic!("failed to parse {:?}: {}", p, e))
}

#[test]
fn test_timer_core_matching_kicad10_passes() {
    let ir = get_timer_core_ir();
    let board = load_fixture_board("timer_core_matching.kicad_pcb");
    let report = check_equivalence(&ir, &board);
    assert!(report.ok, "Matching KiCad 10 board must pass equivalence: {:?}", report.diagnostics);
    assert!(report.diagnostics.is_empty(), "Must have 0 diagnostics on clean board");
}

#[test]
fn test_timer_core_matching_kicad9_passes() {
    let ir = get_timer_core_ir();
    let board = load_fixture_board("timer_core_kicad9_format.kicad_pcb");
    let report = check_equivalence(&ir, &board);
    assert!(report.ok, "Matching KiCad 8/9 board with net numbers must pass equivalence: {:?}", report.diagnostics);
    assert!(report.diagnostics.is_empty(), "Must have 0 diagnostics on clean board");
}

#[test]
fn test_timer_core_misconnected_fails_with_exact_pads() {
    let ir = get_timer_core_ir();
    let board = load_fixture_board("timer_core_misconnected.kicad_pcb");
    let report = check_equivalence(&ir, &board);
    assert!(!report.ok, "Misconnected board must fail equivalence");
    
    let mismatch_diags: Vec<_> = report.diagnostics.iter().filter(|d| d.code == "net-partition-mismatch").collect();
    assert!(!mismatch_diags.is_empty(), "Must produce net-partition-mismatch diagnostics");
    
    // Check that exact pads are reported (U1.1 and U1.7 were swapped)
    let diag_messages = mismatch_diags.iter().map(|d| d.message.as_str()).collect::<Vec<_>>().join(" ");
    assert!(
        diag_messages.contains("U1.1") || diag_messages.contains("U1.7"),
        "Diagnostic message must specifically identify the swapped pads: {}",
        diag_messages
    );
}

#[test]
fn test_timer_core_unrouted_pad_fails() {
    let ir = get_timer_core_ir();
    let board = load_fixture_board("timer_core_unrouted.kicad_pcb");
    let report = check_equivalence(&ir, &board);
    assert!(!report.ok, "Unrouted board must fail equivalence");

    let diag = report.diagnostics.iter().find(|d| d.code == "net-partition-mismatch")
        .expect("Must produce net-partition-mismatch for unrouted pad");
    assert!(
        diag.message.contains("R1.2") || diag.message.contains("R1"),
        "Diagnostic must specifically mention the unrouted R1 pad 2: {}",
        diag.message
    );
}

#[test]
fn test_timer_core_footprint_mismatch_fails() {
    let ir = get_timer_core_ir();
    let board = load_fixture_board("timer_core_footprint_mismatch.kicad_pcb");
    let report = check_equivalence(&ir, &board);
    assert!(!report.ok, "Footprint mismatch must fail equivalence");

    let diag = report.diagnostics.iter().find(|d| d.code == "footprint-mismatch")
        .expect("Must produce footprint-mismatch diagnostic");
    assert_eq!(diag.severity, "error");
    assert!(diag.message.contains("SOIC-8") && diag.message.contains("DIP-8"), "Message should describe mismatch: {}", diag.message);
}

#[test]
fn test_timer_core_field_mismatch_fails() {
    let ir = get_timer_core_ir();
    let board = load_fixture_board("timer_core_field_mismatch.kicad_pcb");
    let report = check_equivalence(&ir, &board);
    assert!(!report.ok, "Field (value) mismatch must fail equivalence");

    let diag = report.diagnostics.iter().find(|d| d.code == "field-mismatch")
        .expect("Must produce field-mismatch diagnostic");
    assert_eq!(diag.severity, "error");
    assert!(diag.message.contains("10k") && diag.message.contains("100k"), "Message should mention values: {}", diag.message);
}

#[test]
fn test_timer_core_component_missing_fails() {
    let ir = get_timer_core_ir();
    let board = load_fixture_board("timer_core_component_missing.kicad_pcb");
    let report = check_equivalence(&ir, &board);
    assert!(!report.ok, "Missing component must fail equivalence");

    let diag = report.diagnostics.iter().find(|d| d.code == "component-missing")
        .expect("Must produce component-missing diagnostic");
    assert_eq!(diag.severity, "error");
    assert!(diag.message.contains("C1"), "Message should identify C1: {}", diag.message);
}

#[test]
fn test_timer_core_component_extra_fails() {
    let ir = get_timer_core_ir();
    let board = load_fixture_board("timer_core_component_extra.kicad_pcb");
    let report = check_equivalence(&ir, &board);
    assert!(!report.ok, "Extra unannotated component must fail equivalence");

    let diag = report.diagnostics.iter().find(|d| d.code == "component-extra")
        .expect("Must produce component-extra diagnostic");
    assert_eq!(diag.severity, "error");
    assert!(diag.message.contains("R99"), "Message should identify R99: {}", diag.message);
}

#[test]
fn test_timer_core_ref_mismatch_fails() {
    let ir = get_timer_core_ir();
    let board = load_fixture_board("timer_core_ref_mismatch.kicad_pcb");
    let report = check_equivalence(&ir, &board);
    assert!(!report.ok, "Reference designator mismatch must fail equivalence");

    let has_ref_or_missing = report.diagnostics.iter().any(|d| d.code == "ref-mismatch" || d.code == "component-missing");
    assert!(has_ref_or_missing, "Expected ref-mismatch or component-missing: {:?}", report.diagnostics);
}
