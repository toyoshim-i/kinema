pub mod ast;
pub mod parser;
pub mod project;

pub use ast::*;
pub use parser::{parse, Lexer, Parser, SyntaxError, Token, TokenKind};
pub use project::{ProjectConfig, ProjectError, ProjectSection};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_leaf_and_circuit() {
        let code = r#"
// Leaf definition
(* footprint = "Package_SO:SOIC-8_3.9x4.9mm_P1.27mm", mpn = "NE555DR", prefix = "U" *)
module NE555 (
    (* pad = "1", etype = "power_in" *) inout GND,
    (* pad = "8", etype = "power_in", decouple = "required" *) inout VCC
);
endmodule

// Circuit
module timer_core (
    (* etype = "power_in" *) inout VCC,
    (* etype = "power_in" *) inout GND
);
    (* nearby *)
    wire vcc_u1;

    join j_vcc_u1 (.P(VCC), .C(vcc_u1));

    (* id = "a3f9" *)
    NE555 U1 (
        .GND(GND),
        .VCC(vcc_u1)
    );
endmodule
"#;
        let ast = parse("test.v", code).expect("parsing should succeed");
        assert_eq!(ast.modules.len(), 2);
        assert_eq!(ast.modules[0].name, "NE555");
        assert_eq!(ast.modules[1].name, "timer_core");
    }

    #[test]
    fn test_disallow_block_comment() {
        let code = "/* block comment */ module foo () ; endmodule";
        assert!(parse("test.v", code).is_err());
    }

    #[test]
    fn test_disallow_preprocessor() {
        let code = "`define FOO 1\nmodule foo () ; endmodule";
        assert!(parse("test.v", code).is_err());
    }

    #[test]
    fn test_disallow_non_zero_range_lsb() {
        let code = "module foo (inout [3:1] a); endmodule";
        assert!(parse("test.v", code).is_err());
    }

    #[test]
    fn test_parse_with_utf8_bom() {
        let code = "\u{feff}module foo (); endmodule\n";
        let ast = parse("test_bom.v", code).expect("parsing with BOM should succeed");
        assert_eq!(ast.modules.len(), 1);
        assert_eq!(ast.modules[0].name, "foo");
    }

    #[test]
    fn test_parse_timer_core_example() {
        let path = std::path::Path::new("../../examples/timer_core.v");
        if path.exists() {
            let content = std::fs::read_to_string(path).expect("read timer_core.v");
            let ast = parse("timer_core.v", &content).expect("parse timer_core.v");
            assert_eq!(ast.modules.len(), 2);
            assert_eq!(ast.modules[0].name, "NE555");
            assert_eq!(ast.modules[1].name, "timer_core");
        }
    }
}
