pub mod footprint;
pub mod netlist;
pub mod pcb_parser;
pub mod rules_gen;

pub use footprint::{parse_kicad_mod, FpLibTable, FootprintInfo, FootprintResolveError, FootprintResolver};
pub use netlist::generate_kicad_netlist;
pub use pcb_parser::{parse_kicad_pcb, PcbBoard, PcbFootprint, PcbPad, PcbParser};
pub use rules_gen::{generate_kicad_dru, generate_kicad_pro, merge_kicad_pro};

#[cfg(test)]
mod tests {
    use super::*;
    use kinema_elab::ir::*;
    use std::time::Instant;

    #[test]
    fn test_generate_kicad_netlist() {
        let ir = FlatNetlistIR {
            top_module: "timer_core".into(),
            components: vec![
                FlatComponent {
                    path: "U1".into(),
                    module_name: "NE555".into(),
                    identity_key: "a3f9".into(),
                    uuid: "12345678-1234-5678-1234-567812345678".into(),
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
                            pad_number: "8".into(),
                            port_name: "VCC".into(),
                            etype: "power_in".into(),
                        },
                    ],
                },
            ],
            nets: vec![
                FlatNet {
                    name: "VCC".into(),
                    pads: vec![
                        ComponentPad {
                            component_path: "U1".into(),
                            component_ref: "U1".into(),
                            pad_number: "8".into(),
                            port_name: "VCC".into(),
                            etype: "power_in".into(),
                        },
                    ],
                    width: Some("0.5mm".into()),
                    current: Some("1.0A".into()),
                    netclass: Some("Power".into()),
                    diffpair: None,
                },
            ],
            nearby_groups: vec![],
            join_nodes: vec![],
        };

        let net_output = generate_kicad_netlist(&ir);
        assert!(net_output.contains("(export (version \"E\")"));
        assert!(net_output.contains("(comp (ref \"U1\")"));
        assert!(net_output.contains("(net (code \"1\") (name \"VCC\")"));
    }

    #[test]
    fn test_fast_selective_parser_large_pcb_benchmark() {
        // Synthesize ~10MB dummy PCB with footprints, nets, and 100,000+ skipped segments/zones
        let mut pcb = String::with_capacity(12 * 1024 * 1024);
        pcb.push_str("(kicad_pcb (version 20240108) (generator \"kinema_bench\")\n");
        pcb.push_str("  (net 0 \"\")\n");
        pcb.push_str("  (net 1 \"GND\")\n");
        pcb.push_str("  (net 2 \"VCC\")\n");

        // Footprint 1
        pcb.push_str("  (footprint \"Package_SO:SOIC-8_3.9x4.9mm_P1.27mm\" (layer \"F.Cu\")\n");
        pcb.push_str("    (tstamp \"a3f9-uuid-0001\")\n");
        pcb.push_str("    (property \"Reference\" \"U1\" (at 0 0 0))\n");
        pcb.push_str("    (property \"Value\" \"NE555DR\" (at 0 1 0))\n");
        pcb.push_str("    (pad \"1\" smd rect (at -1 -1) (size 0.5 1) (net 1 \"GND\") (pinfunction \"GND\") (pintype \"power_in\"))\n");
        pcb.push_str("    (pad \"8\" smd rect (at 1 1) (size 0.5 1) (net 2 \"VCC\") (pinfunction \"VCC\") (pintype \"power_in\"))\n");
        pcb.push_str("  )\n");

        // Massive number of skipped zone polygons and trace segments
        for i in 0..120_000 {
            pcb.push_str(&format!(
                "  (segment (start {} {}) (end {} {}) (width 0.2) (layer \"F.Cu\") (net 1))\n",
                i, i + 1, i + 2, i + 3
            ));
        }

        // Footprint 2 at end of file
        pcb.push_str("  (footprint \"Capacitor_SMD:C_0603_1608Metric\" (layer \"F.Cu\")\n");
        pcb.push_str("    (tstamp \"c1-uuid-0002\")\n");
        pcb.push_str("    (property \"Reference\" \"C1\" (at 5 5 0))\n");
        pcb.push_str("    (property \"Value\" \"100n\" (at 5 6 0))\n");
        pcb.push_str("    (pad \"1\" smd rect (at 5 5) (size 0.5 0.5) (net 2 \"VCC\"))\n");
        pcb.push_str("    (pad \"2\" smd rect (at 6 5) (size 0.5 0.5) (net 1 \"GND\"))\n");
        pcb.push_str("  )\n");

        pcb.push_str(")\n");

        let size_mb = pcb.len() as f64 / (1024.0 * 1024.0);
        assert!(size_mb >= 9.0, "Fixture must be ~10MB+ (actual: {:.2} MB)", size_mb);

        let start = Instant::now();
        let board = parse_kicad_pcb(&pcb).expect("Selective parsing must succeed");
        let duration = start.elapsed();

        println!("Selective parsing of {:.2} MB took {:?}", size_mb, duration);

        // Verification: Correct extraction of footprints and nets despite skipping 120,000 segments
        assert_eq!(board.footprints.len(), 2);
        assert_eq!(board.footprints[0].refdes, "U1");
        assert_eq!(board.footprints[0].pads.len(), 2);
        assert_eq!(board.footprints[1].refdes, "C1");
        assert_eq!(board.nets.len(), 3);

        // DoD requirement: fast parsing of tens of MBs of S-expressions
        assert!(
            duration.as_millis() < 800,
            "Parsing {:.2}MB took too long: {:?}",
            size_mb,
            duration
        );
    }

    #[test]
    fn test_kicad10_pcb_parsing_and_utf8() {
        let pcb = r#"(kicad_pcb (version 20241001) (generator "kicad_10")
  (footprint "Resistor_SMD:R_0603_1608Metric" (layer "F.Cu")
    (tstamp "r1-uuid-0001")
    (property "Reference" "R1" (at 0 0 0))
    (property "Value" "10kΩ" (at 0 1 0))
    (property "mpn" "RT0603BRE0710KL")
    (pad "1" smd rect (at -1 0) (size 0.5 0.5) (net "電源_3.3V") (pinfunction "1") (pintype "passive"))
    (pad "2" smd rect (at 1 0) (size 0.5 0.5) (net "GND") (pinfunction "2") (pintype "passive"))
  )
)
"#;
        let board = parse_kicad_pcb(pcb).expect("Parse KiCad 10 board");
        assert_eq!(board.footprints.len(), 1);
        let fp = &board.footprints[0];
        assert_eq!(fp.refdes, "R1");
        assert_eq!(fp.value, "10kΩ", "UTF-8 string with Greek Omega must not be corrupted");
        assert_eq!(fp.pads.len(), 2);
        assert_eq!(fp.pads[0].pad_number, "1");
        assert_eq!(fp.pads[0].net_name, "電源_3.3V", "KiCad 10 (net \"NAME\") and UTF-8 must be parsed");
        assert_eq!(fp.pads[1].pad_number, "2");
        assert_eq!(fp.pads[1].net_name, "GND");
    }

    #[test]
    fn test_parse_kicad_mod_sot23_5() {
        let content = r#"
(footprint "Package_TO_SOT_SMD:SOT-23-5"
  (version 20240108)
  (pad "1" smd rect (at -0.95 -1.35) (size 0.6 1.05))
  (pad "2" smd rect (at 0 -1.35) (size 0.6 1.05))
  (pad "3" smd rect (at 0.95 -1.35) (size 0.6 1.05))
  (pad "4" smd rect (at 0.95 1.35) (size 0.6 1.05))
  (pad "5" smd rect (at -0.95 1.35) (size 0.6 1.05))
)
"#;
        let info = parse_kicad_mod(content).expect("parse SOT-23-5");
        assert_eq!(info.pad_count, 5, "SOT-23-5 must have exactly 5 pads, never guessed as 3");
        assert_eq!(info.pads, vec!["1", "2", "3", "4", "5"]);
    }

    #[test]
    fn test_fp_lib_table_parsing() {
        let table_str = r#"
(fp_lib_table
  (version 7)
  (lib (name "Package_TO_SOT_SMD")(type "KiCad")(uri "footprints/Package_TO_SOT_SMD.pretty")(options "")(descr ""))
  (lib (name "Resistor_SMD")(type "KiCad")(uri "footprints/Resistor_SMD.pretty")(options "")(descr ""))
)
"#;
        let table = FpLibTable::parse(table_str, Some(std::path::Path::new("/project"))).expect("parse fp-lib-table");
        assert_eq!(table.libraries.len(), 2);
        assert!(table.libraries.contains_key("Package_TO_SOT_SMD"));
        assert!(table.libraries.contains_key("Resistor_SMD"));
    }

    #[test]
    fn test_merge_kicad_pro_preserves_settings_and_emits_patterns() {
        let ir = FlatNetlistIR {
            top_module: "timer_core".into(),
            components: vec![],
            nets: vec![
                FlatNet {
                    name: "VCC".into(),
                    pads: vec![],
                    width: Some("0.5mm".into()),
                    current: None,
                    netclass: Some("Power".into()),
                    diffpair: None,
                },
                FlatNet {
                    name: "GND".into(),
                    pads: vec![],
                    width: Some("0.5mm".into()),
                    current: None,
                    netclass: Some("Power".into()),
                    diffpair: None,
                },
            ],
            nearby_groups: vec![],
            join_nodes: vec![],
        };

        let existing_pro = r#"{
  "board": {
    "design_settings": {
      "rules": {
        "max_error": 0.005
      }
    }
  },
  "sheets": [
    ["uuid-root", ""]
  ]
}"#;

        let merged_json = merge_kicad_pro(&ir, Some(existing_pro)).expect("merge pro");
        let parsed: serde_json::Value = serde_json::from_str(&merged_json).expect("valid json");

        // Existing settings preserved
        assert!(parsed.get("board").is_some(), "Existing board settings must be preserved");
        assert!(parsed.get("sheets").is_some(), "Existing sheets settings must be preserved");

        // Netclass generated
        let classes = parsed["net_settings"]["classes"].as_array().expect("classes array");
        assert!(classes.iter().any(|c| c["name"] == "Power" && c["track_width"] == 0.5));

        // Netclass patterns generated
        let patterns = parsed["net_settings"]["netclass_patterns"].as_array().expect("netclass_patterns array");
        assert!(patterns.iter().any(|p| p["netclass"] == "Power" && p["pattern"] == "VCC"));
        assert!(patterns.iter().any(|p| p["netclass"] == "Power" && p["pattern"] == "GND"));
    }

    #[test]
    fn test_merge_kicad_pro_with_mil_unit() {
        let ir = FlatNetlistIR {
            top_module: "test_mil".into(),
            components: vec![],
            nets: vec![
                FlatNet {
                    name: "RF_SIG".into(),
                    pads: vec![],
                    width: Some("10mil".into()),
                    current: None,
                    netclass: Some("HighSpeed".into()),
                    diffpair: None,
                },
            ],
            nearby_groups: vec![],
            join_nodes: vec![],
        };

        let merged_json = merge_kicad_pro(&ir, None).expect("merge pro");
        let parsed: serde_json::Value = serde_json::from_str(&merged_json).expect("valid json");

        let classes = parsed["net_settings"]["classes"].as_array().expect("classes array");
        let hs_class = classes.iter().find(|c| c["name"] == "HighSpeed").expect("HighSpeed class exists");
        assert_eq!(hs_class["track_width"], 0.254, "10mil must be converted to 0.254mm");
    }
}
