pub mod formatter;

pub use formatter::format_source_file;
use kinema_syntax::parser::parse;

pub fn format_str(filename: &str, input: &str) -> Result<String, kinema_syntax::SyntaxError> {
    let ast = parse(filename, input)?;
    Ok(format_source_file(&ast))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_format_idempotency_sample() {
        let code = r#"
(* mpn = "NE555DR", footprint = "Package_SO:SOIC-8_3.9x4.9mm_P1.27mm", prefix = "U" *)
module NE555 (
    (* etype = "power_in", pad = "1" *) inout GND,
    (* decouple = "required", etype = "power_in", pad = "8" *) inout VCC
);
endmodule

module timer_core (
    (* etype = "power_in" *) inout VCC
);
    (* nearby *)
    wire vcc_u1;

    join j_vcc_u1 (.P(VCC), .C(vcc_u1));

    (* id = "a3f9" *)
    NE555 U1 (
        .GND(VCC),
        .VCC(vcc_u1)
    );
endmodule
"#;
        let f1 = format_str("test.v", code).expect("first format");
        let f2 = format_str("test.v", &f1).expect("second format");
        assert_eq!(f1, f2, "Formatter must be strictly idempotent");

        // Verify canonical attribute ordering in output:
        // footprint before mpn, prefix, etc.
        assert!(f1.contains("(* footprint = \"Package_SO:SOIC-8_3.9x4.9mm_P1.27mm\", mpn = \"NE555DR\", prefix = \"U\" *)"));
        assert!(f1.contains("join j_vcc_u1 (.P(VCC), .C(vcc_u1));"));
    }

    #[test]
    fn test_format_roundtrip() {
        let code = r#"
(* footprint = "Package_SO:SOIC-8_3.9x4.9mm_P1.27mm", prefix = "U", mpn = "NE555DR" *)
module NE555 (
    (* pad = "1", etype = "power_in" *) inout GND,
    (* pad = "8", etype = "power_in" *) inout VCC
);
endmodule

module timer_core (
    (* etype = "power_in" *) inout VCC
);
    (* nearby *)
    wire vcc_u1;

    join j_vcc_u1 (.P(VCC), .C(vcc_u1));

    NE555 U1 (
        .GND(VCC),
        .VCC(vcc_u1)
    );
endmodule
"#;
        let ast1 = parse("test.v", code).expect("parse original");
        let formatted = format_source_file(&ast1);
        let ast2 = parse("test.v", &formatted).expect("parse formatted");
        assert_eq!(ast1.modules.len(), ast2.modules.len());
        for (m1, m2) in ast1.modules.iter().zip(ast2.modules.iter()) {
            assert_eq!(m1.name, m2.name);
            assert_eq!(m1.ports.len(), m2.ports.len());
            assert_eq!(m1.items.len(), m2.items.len());
        }
    }

    #[test]
    fn test_format_timer_core_example() {
        let path = std::path::Path::new("../../examples/timer_core.v");
        if path.exists() {
            let content = std::fs::read_to_string(path).expect("read timer_core.v");
            let f1 = format_str("timer_core.v", &content).expect("first format");
            let f2 = format_str("timer_core.v", &f1).expect("second format");
            assert_eq!(f1, f2, "timer_core.v format must be idempotent");
        }
    }

    proptest! {
        #[test]
        fn test_proptest_idempotency(
            mod_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            wire_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            has_range in any::<bool>(),
            msb in 1u32..16u32,
        ) {
            // Avoid reserved words
            prop_assume!(!["module", "endmodule", "inout", "wire", "parameter"].contains(&mod_name.as_str()));
            prop_assume!(!["module", "endmodule", "inout", "wire", "parameter"].contains(&wire_name.as_str()));
            prop_assume!(!wire_name.is_empty() && !mod_name.is_empty());
            prop_assume!(mod_name != wire_name);

            let wire_decl = if has_range {
                format!("wire [{}:0] {};", msb, wire_name)
            } else {
                format!("wire {};", wire_name)
            };

            let code = format!(
                "module {} (inout port1);\n    {}\nendmodule\n",
                mod_name, wire_decl
            );

            if let Ok(f1) = format_str("prop.v", &code) {
                let f2 = format_str("prop.v", &f1).expect("second format in proptest");
                prop_assert_eq!(f1, f2);
            }
        }
    }
}
