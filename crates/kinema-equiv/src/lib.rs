pub mod checker;

pub use checker::check_equivalence;

#[cfg(test)]
mod tests {
    use super::*;
    use kinema_elab::ir::*;
    use kinema_kicad::pcb_parser::*;
    use kinema_lint::diagnostic::*;

    fn sample_ir() -> FlatNetlistIR {
        FlatNetlistIR {
            top_module: "timer_core".into(),
            components: vec![
                FlatComponent {
                    path: "U1".into(),
                    module_name: "NE555".into(),
                    identity_key: "a3f9".into(),
                    uuid: "a3f9-uuid".into(),
                    refdes: "U1".into(),
                    prefix: "U".into(),
                    footprint: "Package_SO:SOIC-8_3.9x4.9mm_P1.27mm".into(),
                    mpn: Some("NE555DR".into()),
                    value: None,
                    dnp: false,
                    pads: vec![
                        ComponentPad {
                            component_path: "U1".into(),
                            component_ref: "U1".into(),
                            pad_number: "7".into(),
                            port_name: "DIS".into(),
                            etype: "open_collector".into(),
                        },
                        ComponentPad {
                            component_path: "U1".into(),
                            component_ref: "U1".into(),
                            pad_number: "8".into(),
                            port_name: "VCC".into(),
                            etype: "power_in".into(),
                        },
                    ],
                },
                FlatComponent {
                    path: "R1".into(),
                    module_name: "R".into(),
                    identity_key: "r1-key".into(),
                    uuid: "r1-uuid".into(),
                    refdes: "R1".into(),
                    prefix: "R".into(),
                    footprint: "Resistor_SMD:R_0603_1608Metric".into(),
                    mpn: None,
                    value: Some("10k".into()),
                    dnp: false,
                    pads: vec![
                        ComponentPad {
                            component_path: "R1".into(),
                            component_ref: "R1".into(),
                            pad_number: "1".into(),
                            port_name: "A".into(),
                            etype: "passive".into(),
                        },
                        ComponentPad {
                            component_path: "R1".into(),
                            component_ref: "R1".into(),
                            pad_number: "2".into(),
                            port_name: "B".into(),
                            etype: "passive".into(),
                        },
                    ],
                },
            ],
            nets: vec![
                FlatNet {
                    name: "DIS".into(),
                    pads: vec![
                        ComponentPad {
                            component_path: "U1".into(),
                            component_ref: "U1".into(),
                            pad_number: "7".into(),
                            port_name: "DIS".into(),
                            etype: "open_collector".into(),
                        },
                        ComponentPad {
                            component_path: "R1".into(),
                            component_ref: "R1".into(),
                            pad_number: "2".into(),
                            port_name: "B".into(),
                            etype: "passive".into(),
                        },
                    ],
                    width: None,
                    current: None,
                    netclass: None,
                    diffpair: None,
                },
            ],
            nearby_groups: vec![],
            join_nodes: vec![],
        }
    }

    #[test]
    fn test_equiv_matching_board() {
        let ir = sample_ir();
        let pcb_content = r#"
(kicad_pcb (version 20240108) (generator "test")
  (net 1 "DIS")
  (footprint "Package_SO:SOIC-8_3.9x4.9mm_P1.27mm"
    (tstamp "a3f9-uuid")
    (property "Reference" "U1")
    (property "Value" "NE555DR")
    (property "mpn" "NE555DR")
    (pad "7" smd rect (net 1 "DIS"))
  )
  (footprint "Resistor_SMD:R_0603_1608Metric"
    (tstamp "r1-uuid")
    (property "Reference" "R1")
    (property "Value" "10k")
    (pad "2" smd rect (net 1 "DIS"))
  )
)
"#;
        let board = parse_kicad_pcb(pcb_content).expect("parse pcb");
        let report = check_equivalence(&ir, &board);
        assert!(report.ok, "Equivalence should succeed on matching board: {:?}", report.diagnostics);
    }

    #[test]
    fn test_equiv_component_missing() {
        let ir = sample_ir();
        // PCB is missing R1
        let pcb_content = r#"
(kicad_pcb (version 20240108) (generator "test")
  (net 1 "DIS")
  (footprint "Package_SO:SOIC-8_3.9x4.9mm_P1.27mm"
    (tstamp "a3f9-uuid")
    (property "Reference" "U1")
    (pad "7" smd rect (net 1 "DIS"))
  )
)
"#;
        let board = parse_kicad_pcb(pcb_content).expect("parse pcb");
        let report = check_equivalence(&ir, &board);
        assert!(!report.ok);
        assert!(report.diagnostics.iter().any(|d| d.code == "component-missing"));
    }

    #[test]
    fn test_equiv_net_partition_mismatch() {
        let ir = sample_ir();
        // U1 pad 7 is connected to TRIG instead of DIS
        let pcb_content = r#"
(kicad_pcb (version 20240108) (generator "test")
  (net 1 "DIS")
  (net 2 "TRIG")
  (footprint "Package_SO:SOIC-8_3.9x4.9mm_P1.27mm"
    (tstamp "a3f9-uuid")
    (property "Reference" "U1")
    (pad "7" smd rect (net 2 "TRIG"))
  )
  (footprint "Resistor_SMD:R_0603_1608Metric"
    (tstamp "r1-uuid")
    (property "Reference" "R1")
    (property "Value" "10k")
    (pad "2" smd rect (net 1 "DIS"))
  )
)
"#;
        let board = parse_kicad_pcb(pcb_content).expect("parse pcb");
        let report = check_equivalence(&ir, &board);
        assert!(!report.ok);
        assert!(report.diagnostics.iter().any(|d| d.code == "net-partition-mismatch"));
    }

    #[test]
    fn test_equiv_kicad10_board_matching() {
        let ir = sample_ir();
        let pcb_content = r#"
(kicad_pcb (version 20241001) (generator "kicad_10")
  (footprint "Package_SO:SOIC-8_3.9x4.9mm_P1.27mm"
    (tstamp "a3f9-uuid")
    (property "Reference" "U1")
    (property "Value" "NE555DR")
    (property "mpn" "NE555DR")
    (pad "7" smd rect (net "DIS"))
  )
  (footprint "Resistor_SMD:R_0603_1608Metric"
    (tstamp "r1-uuid")
    (property "Reference" "R1")
    (property "Value" "10k")
    (pad "2" smd rect (net "DIS"))
  )
)
"#;
        let board = parse_kicad_pcb(pcb_content).expect("parse pcb");
        let report = check_equivalence(&ir, &board);
        assert!(report.ok, "KiCad 10 board without net numbers must pass equivalence: {:?}", report.diagnostics);
    }

    #[test]
    fn test_equiv_diagnostic_reports_correct_component_and_pads() {
        let ir = sample_ir();
        // In this test, U1 pad 7 is correctly on "DIS", but R1 pad 2 is moved to "OTHER"
        let pcb_content = r#"
(kicad_pcb (version 20241001) (generator "kicad_10")
  (footprint "Package_SO:SOIC-8_3.9x4.9mm_P1.27mm"
    (tstamp "a3f9-uuid")
    (property "Reference" "U1")
    (property "Value" "NE555DR")
    (property "mpn" "NE555DR")
    (pad "7" smd rect (net "DIS"))
  )
  (footprint "Resistor_SMD:R_0603_1608Metric"
    (tstamp "r1-uuid")
    (property "Reference" "R1")
    (property "Value" "10k")
    (pad "2" smd rect (net "OTHER"))
  )
)
"#;
        let board = parse_kicad_pcb(pcb_content).expect("parse pcb");
        let report = check_equivalence(&ir, &board);
        assert!(!report.ok);
        let diag = report.diagnostics.iter().find(|d| d.code == "net-partition-mismatch").expect("must have net-partition-mismatch");
        // Verify location is None (omitted in JSON)
        assert!(diag.location.is_none(), "Equivalence diagnostic location must be None/omitted, got: {:?}", diag.location);
        // Verify it reports the missing pad R1.2, NOT blaming U1
        assert!(diag.message.contains("R1.2") || diag.message.contains("R1"), "Message should mention R1 or R1.2, got: {}", diag.message);
        assert!(!diag.message.contains("Pads of U1 expected on net"), "Must not falsely blame U1: {}", diag.message);
    }

    #[test]
    fn test_equiv_nc_pad_connected() {
        let mut ir = sample_ir();
        ir.components[0].pads.push(ComponentPad {
            component_path: "U1".into(),
            component_ref: "U1".into(),
            pad_number: "5".into(),
            port_name: "CTRL".into(),
            etype: "no_connect".into(),
        });
        let pcb_content = r#"
(kicad_pcb (version 20241001) (generator "kicad_10")
  (footprint "Package_SO:SOIC-8_3.9x4.9mm_P1.27mm"
    (tstamp "a3f9-uuid")
    (property "Reference" "U1")
    (property "Value" "NE555DR")
    (property "mpn" "NE555DR")
    (pad "7" smd rect (net "DIS"))
    (pad "5" smd rect (net "GND"))
  )
  (footprint "Resistor_SMD:R_0603_1608Metric"
    (tstamp "r1-uuid")
    (property "Reference" "R1")
    (property "Value" "10k")
    (pad "2" smd rect (net "DIS"))
  )
)
"#;
        let board = parse_kicad_pcb(pcb_content).expect("parse pcb");
        let report = check_equivalence(&ir, &board);
        assert!(!report.ok);
        assert!(report.diagnostics.iter().any(|d| d.code == "nc-pad-connected"), "Expected nc-pad-connected diagnostic: {:?}", report.diagnostics);
    }

    #[test]
    fn test_equiv_field_mismatch_mpn() {
        let ir = sample_ir(); // U1 has mpn: Some("NE555DR")
        let pcb_content = r#"
(kicad_pcb (version 20241001) (generator "kicad_10")
  (footprint "Package_SO:SOIC-8_3.9x4.9mm_P1.27mm"
    (tstamp "a3f9-uuid")
    (property "Reference" "U1")
    (property "Value" "NE555DR")
    (property "mpn" "WRONG_MPN")
    (pad "7" smd rect (net "DIS"))
  )
  (footprint "Resistor_SMD:R_0603_1608Metric"
    (tstamp "r1-uuid")
    (property "Reference" "R1")
    (property "Value" "10k")
    (pad "2" smd rect (net "DIS"))
  )
)
"#;
        let board = parse_kicad_pcb(pcb_content).expect("parse pcb");
        let report = check_equivalence(&ir, &board);
        assert!(!report.ok);
        assert!(report.diagnostics.iter().any(|d| d.code == "field-mismatch" && d.message.contains("MPN")), "Expected field-mismatch for MPN: {:?}", report.diagnostics);
    }

    #[test]
    fn test_equiv_diagnostic_json_omits_empty_location() {
        let diag = Diagnostic {
            stage: "equiv".into(),
            code: "net-partition-mismatch".into(),
            severity: "error".into(),
            location: None,
            subject: Subject {
                kind: "net".into(),
                name: Some("DIS".into()),
                path: None,
                id: None,
                ref_des: None,
                pad: None,
            },
            related: vec![],
            expected: None,
            actual: None,
            fix: None,
            message: "test".into(),
        };
        let json_str = serde_json::to_string(&diag).unwrap();
        assert!(!json_str.contains("\"location\""), "JSON should not contain location when None: {}", json_str);
    }
}
