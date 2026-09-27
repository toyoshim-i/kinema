pub mod netlist;
pub mod pcb_parser;
pub mod rules_gen;

pub use netlist::generate_kicad_netlist;
pub use pcb_parser::{parse_kicad_pcb, PcbBoard, PcbFootprint, PcbPad, PcbParser};
pub use rules_gen::{generate_kicad_dru, generate_kicad_pro};

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
}
