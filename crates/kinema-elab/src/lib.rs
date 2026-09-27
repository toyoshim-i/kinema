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
}

