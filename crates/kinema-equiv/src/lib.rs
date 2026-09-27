pub mod checker;

pub use checker::check_equivalence;

#[cfg(test)]
mod tests {
    use super::*;
    use kinema_elab::ir::*;
    use kinema_kicad::pcb_parser::*;

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
}
