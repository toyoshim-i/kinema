pub mod elaborator;
pub mod ir;

pub use elaborator::{ElabError, Elaborator};
pub use ir::*;
use kinema_syntax::ast::SourceFile;

pub fn elaborate_source(file: &SourceFile) -> Result<FlatNetlistIR, ElabError> {
    elaborate_sources(std::slice::from_ref(file))
}

pub fn elaborate_sources(files: &[SourceFile]) -> Result<FlatNetlistIR, ElabError> {
    let mut elab = Elaborator::new(files);
    elab.elaborate()
}

#[cfg(test)]
mod tests {
    use super::*;
    use kinema_syntax::parser::parse;

    #[test]
    fn test_elaborate_timer_core() {
        let code = r#"
(* footprint = "Package_SO:SOIC-8_3.9x4.9mm_P1.27mm", mpn = "NE555DR", prefix = "U" *)
module NE555 (
    (* pad = "1", etype = "power_in" *) inout GND,
    (* pad = "2", etype = "input" *) inout TRIG,
    (* pad = "3", etype = "output" *) inout OUT,
    (* pad = "4", etype = "input" *) inout RESET,
    (* pad = "5", etype = "input" *) inout CTRL,
    (* pad = "6", etype = "input" *) inout THR,
    (* pad = "7", etype = "open_collector" *) inout DIS,
    (* pad = "8", etype = "power_in", decouple = "required" *) inout VCC
);
endmodule

(* prefix = "C" *)
module C #(parameter value = "") (
    (* pad = "1", etype = "passive" *) inout A,
    (* pad = "2", etype = "passive" *) inout B
);
endmodule

module join (inout P, inout C); endmodule

module timer_core (
    (* etype = "power_in" *) inout VCC,
    (* etype = "power_in" *) inout GND
);
    (* nearby *)
    wire vcc_u1;
    wire U1_VCC;
    wire C1_A;

    join j_vcc_u1 (.P(VCC), .C(vcc_u1));
    join j_U1_VCC (.P(vcc_u1), .C(U1_VCC));
    join j_C1_A (.P(vcc_u1), .C(C1_A));

    (* id = "a3f9" *)
    NE555 U1 (
        .GND(GND),
        .TRIG(),
        .OUT(),
        .RESET(),
        .CTRL(),
        .THR(),
        .DIS(),
        .VCC(U1_VCC)
    );

    (* footprint = "Capacitor_SMD:C_0603_1608Metric" *)
    C #(.value("100n")) C1 (
        .A(C1_A),
        .B(GND)
    );
endmodule
"#;
        let ast = parse("timer_core.v", code).expect("parse");
        let ir = elaborate_source(&ast).expect("elaborate");

        assert_eq!(ir.top_module, "timer_core");
        assert_eq!(ir.components.len(), 2);

        let u1 = ir.components.iter().find(|c| c.refdes == "U1").expect("find U1");
        assert_eq!(u1.identity_key, "a3f9");
        assert_eq!(u1.footprint, "Package_SO:SOIC-8_3.9x4.9mm_P1.27mm");

        let c1 = ir.components.iter().find(|c| c.refdes == "C1").expect("find C1");
        assert_eq!(c1.value.as_deref(), Some("100n"));

        // Canonical net resolution: U1_VCC and C1_A must merge into root net VCC!
        let vcc_net = ir.nets.iter().find(|n| n.name == "VCC").expect("find VCC net");
        let has_u1_pad8 = vcc_net.pads.iter().any(|p| p.component_ref == "U1" && p.pad_number == "8");
        let has_c1_pad1 = vcc_net.pads.iter().any(|p| p.component_ref == "C1" && p.pad_number == "1");
        assert!(has_u1_pad8, "VCC net must contain U1 pad 8");
        assert!(has_c1_pad1, "VCC net must contain C1 pad 1");

        // Nearby group extraction
        assert_eq!(ir.nearby_groups.len(), 1);
        assert_eq!(ir.nearby_groups[0].hub_wire, "vcc_u1");
        assert_eq!(ir.nearby_groups[0].members.len(), 2); // U1_VCC, C1_A
        assert_eq!(
            ir.nearby_groups[0].pads.len(),
            2,
            "nearby group must only contain the 2 connected pads (U1.8 and C1.1), found: {:?}",
            ir.nearby_groups[0].pads
        );
        assert!(ir.nearby_groups[0].pads.iter().any(|p| p.component_ref == "U1" && p.pad_number == "8"));
        assert!(ir.nearby_groups[0].pads.iter().any(|p| p.component_ref == "C1" && p.pad_number == "1"));
    }

    #[test]
    fn test_uuid_namespace_scoping() {
        let src1 = r#"
            (* prefix = "U", footprint = "Package_SO:SOIC-8_3.9x4.9mm_P1.27mm" *)
            module Chip((* pad = "1" *) inout A);
            endmodule

            module ProjectA ();
                wire sig;
                (* id = "shared_id" *)
                Chip u1(.A(sig));
            endmodule
        "#;

        let src2 = r#"
            (* prefix = "U", footprint = "Package_SO:SOIC-8_3.9x4.9mm_P1.27mm" *)
            module Chip((* pad = "1" *) inout A);
            endmodule

            module ProjectB ();
                wire sig;
                (* id = "shared_id" *)
                Chip u1(.A(sig));
            endmodule
        "#;

        let ast1 = parse("a.kin", src1).expect("parse a");
        let ir1_a = elaborate_source(&ast1).expect("elaborate a");
        let ir1_b = elaborate_source(&ast1).expect("elaborate a again");

        let ast2 = parse("b.kin", src2).expect("parse b");
        let ir2 = elaborate_source(&ast2).expect("elaborate b");

        // Deterministic within same project
        assert_eq!(ir1_a.components[0].uuid, ir1_b.components[0].uuid);

        // Different between different projects even with identical id
        assert_ne!(ir1_a.components[0].uuid, ir2.components[0].uuid);
    }

    #[test]
    fn test_bus_bit_indexing_distinct_nets() {
        let code = r#"
            (* prefix = "R", footprint = "Resistor_SMD:R_0603_1608Metric" *)
            module Resistor (
                (* pad = "1", etype = "passive" *) inout A,
                (* pad = "2", etype = "passive" *) inout B
            );
            endmodule

            module Top ();
                wire [1:0] bus;
                Resistor R1 (
                    .A(bus[0]),
                    .B(bus[1])
                );
            endmodule
        "#;

        let ast = parse("top.kin", code).expect("parse");
        let ir = elaborate_source(&ast).expect("elaborate");

        assert_eq!(ir.components.len(), 1);
        let comp = &ir.components[0];
        assert_eq!(comp.refdes, "R1");

        // bus[0] and bus[1] must be distinct nets!
        let net_names: Vec<_> = ir.nets.iter().map(|n| n.name.as_str()).collect();
        assert!(net_names.contains(&"bus[0]"), "ir.nets must contain bus[0], found: {:?}", net_names);
        assert!(net_names.contains(&"bus[1]"), "ir.nets must contain bus[1], found: {:?}", net_names);
    }

    #[test]
    fn test_auto_ref_allocation_collision_prevention() {
        let code = r#"
            (* prefix = "R", footprint = "Resistor_SMD:R_0603_1608Metric" *)
            module Resistor (
                (* pad = "1", etype = "passive" *) inout A,
                (* pad = "2", etype = "passive" *) inout B
            );
            endmodule

            module Top ();
                wire a;
                wire b;
                wire c;
                Resistor R1 (.A(a), .B(b));
                Resistor spare (.A(b), .B(c));
            endmodule
        "#;

        let ast = parse("top.kin", code).expect("parse");
        let ir = elaborate_source(&ast).expect("elaborate");

        assert_eq!(ir.components.len(), 2);
        let refs: Vec<_> = ir.components.iter().map(|c| c.refdes.as_str()).collect();
        assert!(refs.contains(&"R1"), "Must contain R1");
        assert!(refs.contains(&"R2"), "spare must be allocated as R2 without colliding with R1, found: {:?}", refs);
    }

    #[test]
    fn test_duplicate_refdes_error() {
        let code = r#"
            (* prefix = "R", footprint = "Resistor_SMD:R_0603_1608Metric" *)
            module Resistor (
                (* pad = "1", etype = "passive" *) inout A,
                (* pad = "2", etype = "passive" *) inout B
            );
            endmodule

            module Top ();
                wire a;
                wire b;
                Resistor R1 (.A(a), .B(b));
                (* ref = "R1" *)
                Resistor other (.A(a), .B(b));
            endmodule
        "#;

        let ast = parse("top.kin", code).expect("parse");
        let err = elaborate_source(&ast).expect_err("Must fail due to duplicate R1 refdes");
        match err {
            ElabError::DuplicateRef(r) => assert_eq!(r, "R1"),
            other => panic!("Expected DuplicateRef, got {:?}", other),
        }
    }

    #[test]
    fn test_default_parameter_value_preservation() {
        let code = r#"
            (* prefix = "R", footprint = "Resistor_SMD:R_0603_1608Metric" *)
            module Resistor #(parameter value = "10k") (
                (* pad = "1", etype = "passive" *) inout A,
                (* pad = "2", etype = "passive" *) inout B
            );
            endmodule

            module Top ();
                wire a;
                wire b;
                wire c;
                wire d;
                Resistor R1 (.A(a), .B(b));
                Resistor #(.value("22k")) R2 (.A(c), .B(d));
            endmodule
        "#;

        let ast = parse("top.kin", code).expect("parse");
        let ir = elaborate_source(&ast).expect("elaborate");

        let r1 = ir.components.iter().find(|c| c.refdes == "R1").unwrap();
        assert_eq!(r1.value.as_deref(), Some("10k"), "R1 must preserve default parameter value '10k'");

        let r2 = ir.components.iter().find(|c| c.refdes == "R2").unwrap();
        assert_eq!(r2.value.as_deref(), Some("22k"), "R2 must have overridden parameter value '22k'");
    }

    #[test]
    fn test_port_attribute_and_join_constraint_inheritance() {
        let code = r#"
            (* prefix = "U", footprint = "Package_SO:SOIC-8_3.9x4.9mm_P1.27mm" *)
            module Chip (
                (* pad = "1", etype = "power_in" *) inout VCC
            );
            endmodule

            module Top (
                (* width = "0.8mm", netclass = "Power" *) inout VCC
            );
                (* nearby, width = "0.6mm" *)
                wire vcc_hub;
                wire u1_vcc;

                join j_hub (.P(VCC), .C(vcc_hub));
                join j_u1 (.P(vcc_hub), .C(u1_vcc));

                Chip U1 (.VCC(u1_vcc));
            endmodule
        "#;

        let ast = parse("top.kin", code).expect("parse");
        let ir = elaborate_source(&ast).expect("elaborate");

        let vcc_net = ir.nets.iter().find(|n| n.name == "VCC").expect("find VCC net");
        assert_eq!(vcc_net.width.as_deref(), Some("0.8mm"), "Must inherit width from port");
        assert_eq!(vcc_net.netclass.as_deref(), Some("Power"), "Must inherit netclass from port");
    }

    #[test]
    fn test_concat_port_connection() {
        let code = r#"
            (* prefix = "D", footprint = "Diode_SMD:D_SOD-123" *)
            module DualDiode (
                (* pad = "1,2", etype = "passive" *) inout [1:0] pins
            );
            endmodule

            module Top ();
                wire sigA;
                wire sigB;
                DualDiode D1 (.pins({sigA, sigB}));
            endmodule
        "#;

        let ast = parse("top.kin", code).expect("parse");
        let ir = elaborate_source(&ast).expect("elaborate");

        let siga_net = ir.nets.iter().find(|n| n.name == "sigA").expect("find sigA");
        let sigb_net = ir.nets.iter().find(|n| n.name == "sigB").expect("find sigB");

        assert_eq!(siga_net.pads.len(), 1);
        assert_eq!(siga_net.pads[0].pad_number, "1");
        assert_eq!(sigb_net.pads.len(), 1);
        assert_eq!(sigb_net.pads[0].pad_number, "2");
    }

    #[test]
    fn test_ir_validation_dangling_pad() {
        let ir = FlatNetlistIR {
            top_module: "Top".into(),
            components: vec![],
            nets: vec![FlatNet {
                name: "NET1".into(),
                pads: vec![ComponentPad {
                    component_path: "U1".into(),
                    component_ref: "U1".into(),
                    pad_number: "1".into(),
                    port_name: "P".into(),
                    etype: "passive".into(),
                }],
                width: None,
                current: None,
                netclass: None,
                diffpair: None,
            }],
            nearby_groups: vec![],
            join_nodes: vec![],
        };
        let err = ir.validate().expect_err("dangling pad must fail validation");
        assert!(matches!(err, ElabError::DanglingPad(..)));
    }

    #[test]
    fn test_ir_validation_pad_multi_net() {
        let pad = ComponentPad {
            component_path: "R1".into(),
            component_ref: "R1".into(),
            pad_number: "1".into(),
            port_name: "A".into(),
            etype: "passive".into(),
        };
        let ir = FlatNetlistIR {
            top_module: "Top".into(),
            components: vec![FlatComponent {
                path: "R1".into(),
                module_name: "R".into(),
                identity_key: "r1".into(),
                uuid: "uuid-r1".into(),
                refdes: "R1".into(),
                prefix: "R".into(),
                footprint: "R_0603".into(),
                mpn: None,
                value: None,
                dnp: false,
                pads: vec![pad.clone()],
            }],
            nets: vec![
                FlatNet {
                    name: "NET_A".into(),
                    pads: vec![pad.clone()],
                    width: None,
                    current: None,
                    netclass: None,
                    diffpair: None,
                },
                FlatNet {
                    name: "NET_B".into(),
                    pads: vec![pad],
                    width: None,
                    current: None,
                    netclass: None,
                    diffpair: None,
                },
            ],
            nearby_groups: vec![],
            join_nodes: vec![],
        };
        let err = ir.validate().expect_err("pad mapped to multiple nets must fail");
        assert!(matches!(err, ElabError::PadMultiNet(..)));
    }

    #[test]
    fn test_ir_validation_duplicate_ref() {
        let ir = FlatNetlistIR {
            top_module: "Top".into(),
            components: vec![
                FlatComponent {
                    path: "R1a".into(),
                    module_name: "R".into(),
                    identity_key: "r1a".into(),
                    uuid: "uuid-1".into(),
                    refdes: "R1".into(),
                    prefix: "R".into(),
                    footprint: "R_0603".into(),
                    mpn: None,
                    value: None,
                    dnp: false,
                    pads: vec![],
                },
                FlatComponent {
                    path: "R1b".into(),
                    module_name: "R".into(),
                    identity_key: "r1b".into(),
                    uuid: "uuid-2".into(),
                    refdes: "R1".into(),
                    prefix: "R".into(),
                    footprint: "R_0603".into(),
                    mpn: None,
                    value: None,
                    dnp: false,
                    pads: vec![],
                },
            ],
            nets: vec![],
            nearby_groups: vec![],
            join_nodes: vec![],
        };
        let err = ir.validate().expect_err("duplicate refdes must fail");
        assert!(matches!(err, ElabError::DuplicateRef(..)));
    }

    #[test]
    fn test_ir_validation_invalid_constraint() {
        let ir = FlatNetlistIR {
            top_module: "Top".into(),
            components: vec![],
            nets: vec![FlatNet {
                name: "PWR".into(),
                pads: vec![],
                width: Some("invalid_width".into()),
                current: None,
                netclass: None,
                diffpair: None,
            }],
            nearby_groups: vec![],
            join_nodes: vec![],
        };
        let err = ir.validate().expect_err("invalid width constraint must fail");
        assert!(matches!(err, ElabError::InvalidConstraint(..)));
    }
}



