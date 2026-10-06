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

    #[test]
    fn test_disallow_assign_keyword() {
        let code = "module foo (); wire a; wire b; assign a = b; endmodule";
        let err = parse("test.v", code).expect_err("assign must be rejected");
        assert!(err.message.contains("Continuous assignment 'assign' is not supported"));
    }

    #[test]
    fn test_disallow_input_output_keywords() {
        let code1 = "module foo (input a); endmodule";
        let err1 = parse("test.v", code1).expect_err("input must be rejected");
        assert!(err1.message.contains("Keyword 'input' is not supported"));

        let code2 = "module foo (output a); endmodule";
        let err2 = parse("test.v", code2).expect_err("output must be rejected");
        assert!(err2.message.contains("Keyword 'output' is not supported"));
    }

    #[test]
    fn test_disallow_reg_and_behavioral() {
        let code1 = "module foo (); reg a; endmodule";
        let err1 = parse("test.v", code1).expect_err("reg must be rejected");
        assert!(err1.message.contains("Type keyword 'reg' is not supported"));

        let code2 = "module foo (); always @* ; endmodule";
        let err2 = parse("test.v", code2).expect_err("always must be rejected");
        assert!(err2.message.contains("Behavioral construct 'always' is not supported"));
    }

    #[test]
    fn test_disallow_ascending_slice() {
        let code = "module foo (); wire [3:0] bus; R R1 (.A(bus[0:3])); endmodule";
        let err = parse("test.v", code).expect_err("ascending slice must be rejected");
        assert!(err.message.contains("Bit range slice must be descending"));
    }

    #[test]
    fn test_disallow_replication_and_literal_constants() {
        let code1 = "module foo (); wire sig; R R1 (.A({4{sig}})); endmodule";
        let err1 = parse("test.v", code1).expect_err("replication must be rejected");
        assert!(err1.message.contains("Replication operator"));

        let code2 = "module foo (); R R1 (.A(0)); endmodule";
        let err2 = parse("test.v", code2).expect_err("literal constant must be rejected");
        assert!(err2.message.contains("Literal numbers and constants are not supported"));
    }

    #[test]
    fn test_project_config_standard_section() {
        let toml_str = r#"
[project]
name = "my_custom_project"
sources = ["src/top.v", "src/sub.v"]
libraries = ["lib/custom.v"]
board = "pcb/my_board.kicad_pcb"
"#;
        let config: ProjectConfig = toml::from_str(toml_str).expect("parse standard project config");
        assert_eq!(config.project.name.as_deref(), Some("my_custom_project"));
        assert_eq!(config.project.sources, vec!["src/top.v", "src/sub.v"]);
        assert_eq!(config.project.libraries, vec!["lib/custom.v"]);
        assert_eq!(config.project.board.as_deref(), Some("pcb/my_board.kicad_pcb"));
    }

    #[test]
    fn test_project_config_flat_format() {
        let toml_str = r#"
name = "flat_project"
sources = ["flat_src/*.v"]
libraries = ["flat_lib/*.v"]
board = "flat_board.kicad_pcb"
"#;
        let config: ProjectConfig = toml::from_str(toml_str).expect("parse flat project config");
        assert_eq!(config.project.name.as_deref(), Some("flat_project"));
        assert_eq!(config.project.sources, vec!["flat_src/*.v"]);
        assert_eq!(config.project.libraries, vec!["flat_lib/*.v"]);
        assert_eq!(config.project.board.as_deref(), Some("flat_board.kicad_pcb"));
    }

    #[test]
    fn test_project_config_default_empty() {
        let config: ProjectConfig = toml::from_str("").expect("parse empty project config");
        let default_sec = ProjectSection::default();
        assert_eq!(config.project, default_sec);
        assert_eq!(config.kicad, None);
    }

    #[test]
    fn test_project_config_with_kicad_section() {
        let toml_str = r#"
[project]
name = "kicad_project"

[kicad]
config_dir = "/custom/kicad/config"
footprint_dir = "/custom/kicad/footprints"
fp_lib_table = "/custom/kicad/fp-lib-table"
search_paths = ["/extra/footprints", "local/footprints"]
"#;
        let config: ProjectConfig = toml::from_str(toml_str).expect("parse kicad config section");
        let kicad = config.kicad.expect("kicad section present");
        assert_eq!(kicad.config_dir, Some(std::path::PathBuf::from("/custom/kicad/config")));
        assert_eq!(kicad.footprint_dir, Some(std::path::PathBuf::from("/custom/kicad/footprints")));
        assert_eq!(kicad.fp_lib_table, Some(std::path::PathBuf::from("/custom/kicad/fp-lib-table")));
        assert_eq!(kicad.search_paths.len(), 2);
    }
}
