use kinema_syntax::ast::*;

pub fn format_source_file(file: &SourceFile) -> String {
    let mut out = String::new();

    for (i, module) in file.modules.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        format_module(module, &mut out);
    }

    for comment in &file.eof_comments {
        out.push_str("//");
        out.push_str(comment);
        out.push('\n');
    }

    if !out.ends_with('\n') {
        out.push('\n');
    }

    out
}

fn format_attrs(attrs: &[Attr]) -> Option<String> {
    if attrs.is_empty() {
        return None;
    }

    let mut sorted_attrs = attrs.to_vec();
    sorted_attrs.sort_by_key(|a| attr_canonical_index(&a.key));

    let mut parts = Vec::new();
    for attr in sorted_attrs {
        if let Some(val) = attr.value {
            parts.push(format!("{} = \"{}\"", attr.key, val));
        } else {
            parts.push(attr.key);
        }
    }

    Some(format!("(* {} *)", parts.join(", ")))
}

fn format_range(range: &RangeDef) -> String {
    format!("[{}:{}]", range.msb, range.lsb)
}

fn format_ref(r: &RefExpr) -> String {
    match &r.index {
        None => r.ident.clone(),
        Some(RefIndex::Single(i)) => format!("{}[{}]", r.ident, i),
        Some(RefIndex::Range(msb, lsb)) => format!("{}[{}:{}]", r.ident, msb, lsb),
    }
}

fn format_expr(expr: &Expr) -> String {
    match expr {
        Expr::Ref(r) => format_ref(r),
        Expr::Concat(list) => {
            let inner = list.iter().map(format_ref).collect::<Vec<_>>().join(", ");
            format!("{{{}}}", inner)
        }
    }
}

fn format_module(module: &ModuleDef, out: &mut String) {
    // Leading comments for module
    for comment in &module.leading_comments {
        out.push_str("//");
        out.push_str(comment);
        out.push('\n');
    }

    // Module attributes on independent line
    if let Some(attr_str) = format_attrs(&module.attrs) {
        out.push_str(&attr_str);
        out.push('\n');
    }

    out.push_str("module ");
    out.push_str(&module.name);

    if !module.params.is_empty() {
        out.push_str(" #(");
        let param_strs = module
            .params
            .iter()
            .map(|p| format!("parameter {} = \"{}\"", p.name, p.value))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&param_strs);
        out.push(')');
    }

    out.push_str(" (\n");

    for (i, port) in module.ports.iter().enumerate() {
        for comment in &port.leading_comments {
            out.push_str("    //");
            out.push_str(comment);
            out.push('\n');
        }

        out.push_str("    ");
        if let Some(attr_str) = format_attrs(&port.attrs) {
            out.push_str(&attr_str);
            out.push(' ');
        }
        out.push_str("inout ");
        if let Some(range) = &port.range {
            out.push_str(&format_range(range));
            out.push(' ');
        }
        out.push_str(&port.name);

        if i + 1 < module.ports.len() {
            out.push(',');
        }

        if let Some(c) = &port.trailing_comment {
            out.push_str(" //");
            out.push_str(c);
        }
        out.push('\n');
    }

    out.push_str(");\n");

    // Separate items into wires and instances with an empty line between groups
    let mut prev_is_wire: Option<bool> = None;

    for item in &module.items {
        let is_wire = item.is_wire();
        if let Some(prev) = prev_is_wire {
            if prev && !is_wire {
                out.push('\n');
            }
        }
        prev_is_wire = Some(is_wire);

        match item {
            Item::Wire(wire) => {
                for comment in &wire.leading_comments {
                    out.push_str("    //");
                    out.push_str(comment);
                    out.push('\n');
                }

                if let Some(attr_str) = format_attrs(&wire.attrs) {
                    out.push_str("    ");
                    out.push_str(&attr_str);
                    out.push('\n');
                }

                out.push_str("    wire ");
                if let Some(range) = &wire.range {
                    out.push_str(&format_range(range));
                    out.push(' ');
                }
                out.push_str(&wire.name);
                out.push(';');
                if let Some(c) = &wire.trailing_comment {
                    out.push_str(" //");
                    out.push_str(c);
                }
                out.push('\n');
            }
            Item::Instance(inst) => {
                for comment in &inst.leading_comments {
                    out.push_str("    //");
                    out.push_str(comment);
                    out.push('\n');
                }

                if inst.module_name == "join" {
                    // Exception: join instance is formatted on 1 line
                    out.push_str("    join ");
                    out.push_str(&inst.instance_name);
                    out.push_str(" (");
                    let conn_strs = inst
                        .connections
                        .iter()
                        .map(|c| {
                            let expr_str = c.expr.as_ref().map(format_expr).unwrap_or_default();
                            format!(".{}({})", c.port_name, expr_str)
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    out.push_str(&conn_strs);
                    out.push_str(");");
                    if let Some(c) = &inst.trailing_comment {
                        out.push_str(" //");
                        out.push_str(c);
                    }
                    out.push('\n');
                } else {
                    if let Some(attr_str) = format_attrs(&inst.attrs) {
                        out.push_str("    ");
                        out.push_str(&attr_str);
                        out.push('\n');
                    }

                    out.push_str("    ");
                    out.push_str(&inst.module_name);

                    if !inst.param_overrides.is_empty() {
                        out.push_str(" #(");
                        let p_strs = inst
                            .param_overrides
                            .iter()
                            .map(|p| format!(".{}(\"{}\")", p.name, p.value))
                            .collect::<Vec<_>>()
                            .join(", ");
                        out.push_str(&p_strs);
                        out.push(')');
                    }

                    out.push(' ');
                    out.push_str(&inst.instance_name);
                    out.push_str(" (\n");

                    for (i, conn) in inst.connections.iter().enumerate() {
                        out.push_str("        .");
                        out.push_str(&conn.port_name);
                        out.push('(');
                        if let Some(expr) = &conn.expr {
                            out.push_str(&format_expr(expr));
                        }
                        out.push(')');
                        if i + 1 < inst.connections.len() {
                            out.push(',');
                        }
                        if let Some(c) = &conn.trailing_comment {
                            out.push_str(" //");
                            out.push_str(c);
                        }
                        out.push('\n');
                    }

                    out.push_str("    );");
                    if let Some(c) = &inst.trailing_comment {
                        out.push_str(" //");
                        out.push_str(c);
                    }
                    out.push('\n');
                }
            }
        }
    }

    for comment in &module.endmodule_comments {
        out.push_str("    //");
        out.push_str(comment);
        out.push('\n');
    }

    out.push_str("endmodule\n");
}
