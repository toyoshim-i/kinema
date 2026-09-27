pub mod diagnostic;
pub mod rules;

pub use diagnostic::{Diagnostic, LintReport, Related, Subject};
pub use rules::check_rules;

use kinema_syntax::ast::SourceFile;
use kinema_syntax::parser::parse;

pub fn lint_source_files(files: &[SourceFile]) -> LintReport {
    let diags = check_rules(files);
    LintReport::from_diagnostics(diags)
}

pub fn lint_str(filename: &str, content: &str) -> Result<LintReport, kinema_syntax::SyntaxError> {
    let ast = parse(filename, content)?;
    Ok(lint_source_files(&[ast]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_rule_triggered(code: &str, expected_rule: &str) {
        let report = lint_str("test.v", code).expect("must parse");
        let found = report.diagnostics.iter().any(|d| d.code == expected_rule);
        assert!(
            found,
            "Expected rule '{}' to trigger, but got diagnostics: {:?}",
            expected_rule,
            report.diagnostics
        );

        // Verify JSON serialization conforms to spec
        let json_str = serde_json::to_string(&report).expect("must serialize");
        assert!(json_str.contains(expected_rule));
    }

    #[test]
    fn test_rule_undeclared_net() {
        let code = r#"
module leaf (inout A); endmodule
module top ();
    leaf u1 (.A(undeclared_net));
endmodule
"#;
        check_rule_triggered(code, "undeclared-net");
    }

    #[test]
    fn test_rule_width_mismatch() {
        let code = r#"
module leaf (inout [7:0] bus); endmodule
module top ();
    wire wire1;
    leaf u1 (.bus(wire1));
endmodule
"#;
        check_rule_triggered(code, "width-mismatch");
    }

    #[test]
    fn test_rule_missing_port() {
        let code = r#"
module leaf (inout A, inout B); endmodule
module top ();
    wire w;
    leaf u1 (.A(w));
endmodule
"#;
        check_rule_triggered(code, "missing-port");
    }

    #[test]
    fn test_rule_unknown_attr() {
        let code = r#"
(* bogus_attribute = "bad" *)
module leaf (inout A); endmodule
module top ();
    leaf u1 (.A());
endmodule
"#;
        check_rule_triggered(code, "unknown-attr");
    }

    #[test]
    fn test_rule_unknown_param() {
        let code = r#"
module leaf (inout A); endmodule
module top ();
    leaf #(.bad_param("1")) u1 (.A());
endmodule
"#;
        check_rule_triggered(code, "unknown-param");
    }

    #[test]
    fn test_rule_param_missing() {
        let code = r#"
module leaf #(parameter value = "") (inout A); endmodule
module top ();
    leaf u1 (.A());
endmodule
"#;
        check_rule_triggered(code, "param-missing");
    }

    #[test]
    fn test_rule_unit_invalid() {
        let code = r#"
module top ();
    (* width = "5" *)
    wire bad_width;
endmodule
"#;
        check_rule_triggered(code, "unit-invalid");
    }

    #[test]
    fn test_rule_duplicate_module() {
        let code = r#"
module dupe (); endmodule
module dupe (); endmodule
"#;
        check_rule_triggered(code, "duplicate-module");
    }

    #[test]
    fn test_rule_top_module() {
        let code = r#"
module top1 (); endmodule
module top2 (); endmodule
"#;
        check_rule_triggered(code, "top-module");
    }

    #[test]
    fn test_rule_recursive_instance() {
        let code = r#"
module rec ();
    rec u1 ();
endmodule
"#;
        check_rule_triggered(code, "recursive-instance");
    }

    #[test]
    fn test_rule_leaf_incomplete() {
        let code = r#"
module leaf (inout A); endmodule
module top ();
    leaf u1 (.A());
endmodule
"#;
        check_rule_triggered(code, "leaf-incomplete");
    }

    #[test]
    fn test_rule_pad_duplicate() {
        let code = r#"
(* prefix = "U" *)
module leaf (
    (* pad = "1", etype = "input" *) inout A,
    (* pad = "1", etype = "output" *) inout B
);
endmodule
module top ();
    leaf u1 (.A(), .B());
endmodule
"#;
        check_rule_triggered(code, "pad-duplicate");
    }

    #[test]
    fn test_rule_pad_count() {
        let code = r#"
(* prefix = "U", footprint = "Package_SO:SOIC-8_3.9x4.9mm_P1.27mm" *)
module leaf (
    (* pad = "1", etype = "input" *) inout A
);
endmodule
module top ();
    leaf u1 (.A());
endmodule
"#;
        check_rule_triggered(code, "pad-count");
    }

    #[test]
    fn test_rule_footprint_missing() {
        let code = r#"
(* prefix = "R" *)
module R (
    (* pad = "1", etype = "passive" *) inout A
);
endmodule
module top ();
    R r1 (.A());
endmodule
"#;
        check_rule_triggered(code, "footprint-missing");
    }

    #[test]
    fn test_rule_hub_direct_pad() {
        let code = r#"
module leaf (inout A); endmodule
module top ();
    (* nearby *)
    wire hub1;
    leaf u1 (.A(hub1));
endmodule
"#;
        check_rule_triggered(code, "hub-direct-pad");
    }

    #[test]
    fn test_rule_hub_too_few() {
        let code = r#"
module join (inout P, inout C); endmodule
module leaf (inout A); endmodule
module top ();
    (* nearby *)
    wire hub1;
    wire u1_A;
    join j_u1_A (.P(hub1), .C(u1_A));
    leaf u1 (.A(u1_A));
endmodule
"#;
        check_rule_triggered(code, "hub-too-few");
    }

    #[test]
    fn test_rule_join_cycle() {
        let code = r#"
module join (inout P, inout C); endmodule
module top ();
    wire w1;
    wire w2;
    join j_w1 (.P(w2), .C(w1));
    join j_w2 (.P(w1), .C(w2));
endmodule
"#;
        check_rule_triggered(code, "join-cycle");
    }

    #[test]
    fn test_rule_join_name() {
        let code = r#"
module join (inout P, inout C); endmodule
module top ();
    wire w1;
    wire w2;
    join wrong_name (.P(w1), .C(w2));
endmodule
"#;
        check_rule_triggered(code, "join-name");
    }

    #[test]
    fn test_rule_pin_wire_name() {
        let code = r#"
module join (inout P, inout C); endmodule
module leaf (inout A); endmodule
module top ();
    (* nearby *)
    wire hub1;
    wire wrong_wire_name;
    join j_wrong_wire_name (.P(hub1), .C(wrong_wire_name));
    leaf u1 (.A(wrong_wire_name));
endmodule
"#;
        check_rule_triggered(code, "pin-wire-name");
    }

    #[test]
    fn test_rule_pin_wire_shape() {
        let code = r#"
module join (inout P, inout C); endmodule
module leaf (inout A, inout B); endmodule
module top ();
    (* nearby *)
    wire hub1;
    wire u1_A;
    join j_u1_A (.P(hub1), .C(u1_A));
    leaf u1 (.A(u1_A), .B(u1_A));
endmodule
"#;
        check_rule_triggered(code, "pin-wire-shape");
    }

    #[test]
    fn test_rule_pin_wire_attr() {
        let code = r#"
module join (inout P, inout C); endmodule
module leaf (inout A); endmodule
module top ();
    (* nearby *)
    wire hub1;
    (* width = "0.5mm" *)
    wire u1_A;
    join j_u1_A (.P(hub1), .C(u1_A));
    leaf u1 (.A(u1_A));
endmodule
"#;
        check_rule_triggered(code, "pin-wire-attr");
    }

    #[test]
    fn test_rule_decouple_missing() {
        let code = r#"
(* prefix = "U", footprint = "Package_SO:SOIC-8_3.9x4.9mm_P1.27mm" *)
module leaf (
    (* pad = "8", etype = "power_in", decouple = "required" *) inout VCC
);
endmodule
module top ();
    wire VCC;
    leaf u1 (.VCC(VCC));
endmodule
"#;
        check_rule_triggered(code, "decouple-missing");
    }

    #[test]
    fn test_rule_decouple_multi_pad() {
        let code = r#"
(* prefix = "U" *)
module leaf (
    (* pad = "1,8", etype = "power_in", decouple = "required" *) inout VCC
);
endmodule
module top ();
    leaf u1 (.VCC());
endmodule
"#;
        check_rule_triggered(code, "decouple-multi-pad");
    }

    #[test]
    fn test_rule_power_unconnected() {
        let code = r#"
(* prefix = "U" *)
module leaf (
    (* pad = "1", etype = "power_in" *) inout VCC
);
endmodule
module top ();
    leaf u1 (.VCC());
endmodule
"#;
        check_rule_triggered(code, "power-unconnected");
    }

    #[test]
    fn test_rule_power_conflict() {
        let code = r#"
(* prefix = "U" *)
module leaf (
    (* pad = "1", etype = "power_out" *) inout OUT1
);
endmodule
module top ();
    wire net1;
    leaf u1 (.OUT1(net1));
    leaf u2 (.OUT1(net1));
endmodule
"#;
        check_rule_triggered(code, "power-conflict");
    }

    #[test]
    fn test_rule_nc_connected() {
        let code = r#"
(* prefix = "U" *)
module leaf (
    (* pad = "1", etype = "no_connect" *) inout NC
);
endmodule
module top ();
    wire net1;
    leaf u1 (.NC(net1));
endmodule
"#;
        check_rule_triggered(code, "nc-connected");
    }

    #[test]
    fn test_rule_diffpair_invalid() {
        let code = r#"
module top ();
    (* diffpair = "USB_D" *)
    wire USB_DP;
    (* diffpair = "USB_D" *)
    wire USB_WRONG;
endmodule
"#;
        check_rule_triggered(code, "diffpair-invalid");
    }

    #[test]
    fn test_rule_duplicate_identity() {
        let code = r#"
module leaf (inout A); endmodule
module top ();
    (* id = "same_id" *)
    leaf u1 (.A());
    (* id = "same_id" *)
    leaf u2 (.A());
endmodule
"#;
        check_rule_triggered(code, "duplicate-identity");
    }

    #[test]
    fn test_rule_constraint_conflict() {
        let code = r#"
module top ();
    (* width = "0.5mm" *)
    wire net1;
    (* width = "1.0mm" *)
    wire net1;
endmodule
"#;
        check_rule_triggered(code, "constraint-conflict");
    }

    #[test]
    fn test_rule_single_pin_net() {
        let code = r#"
(* prefix = "U" *)
module leaf (
    (* pad = "1", etype = "output" *) inout A
);
endmodule
module top ();
    wire lone_wire;
    leaf u1 (.A(lone_wire));
endmodule
"#;
        check_rule_triggered(code, "single-pin-net");
    }

    #[test]
    fn test_rule_undriven_net() {
        let code = r#"
(* prefix = "U" *)
module leaf (
    (* pad = "1", etype = "input" *) inout IN1,
    (* pad = "2", etype = "input" *) inout IN2
);
endmodule
module top ();
    wire float_wire;
    leaf u1 (.IN1(float_wire), .IN2(float_wire));
endmodule
"#;
        check_rule_triggered(code, "undriven-net");
    }

    #[test]
    fn test_lint_timer_core_clean() {
        let path = std::path::Path::new("../../examples/timer_core.v");
        let std_path = std::path::Path::new("../../lib/std.v");
        if path.exists() && std_path.exists() {
            let tc_content = std::fs::read_to_string(path).expect("read timer_core.v");
            let std_content = std::fs::read_to_string(std_path).expect("read std.v");

            let tc_ast = parse("timer_core.v", &tc_content).expect("parse timer_core.v");
            let std_ast = parse("std.v", &std_content).expect("parse std.v");

            let report = lint_source_files(&[std_ast, tc_ast]);
            let errors: Vec<_> = report.diagnostics.iter().filter(|d| d.severity == "error").collect();
            assert!(
                errors.is_empty(),
                "timer_core.v must have 0 lint errors, but found: {:?}",
                errors
            );
        }
    }
}
