use crate::diagnostic::*;
use kinema_syntax::ast::*;
use std::collections::{HashMap, HashSet};

pub fn check_rules(source_files: &[SourceFile]) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    // 8. duplicate-module
    let mut module_map: HashMap<String, Vec<&ModuleDef>> = HashMap::new();
    for file in source_files {
        for m in &file.modules {
            module_map.entry(m.name.clone()).or_default().push(m);
        }
    }

    for (name, defs) in &module_map {
        if defs.len() > 1 {
            let first = defs[0];
            let dup = defs[1];
            diags.push(Diagnostic {
                stage: "lint".into(),
                code: "duplicate-module".into(),
                severity: "error".into(),
                location: dup.span.to_location(),
                subject: Subject {
                    kind: "component".into(),
                    name: Some(name.clone()),
                    path: Some(name.clone()),
                    id: None,
                    ref_des: None,
                    pad: None,
                },
                related: vec![Related {
                    role: "previous-definition".into(),
                    name: Some(name.clone()),
                    location: Some(first.span.to_location()),
                }],
                expected: None,
                actual: None,
                fix: None,
                message: format!("Module '{}' is defined multiple times", name),
            });
        }
    }

    // 9. top-module
    let mut instantiated = HashSet::new();
    for file in source_files {
        for m in &file.modules {
            for item in &m.items {
                if let Item::Instance(inst) = item {
                    if inst.module_name != "join" {
                        instantiated.insert(inst.module_name.clone());
                    }
                }
            }
        }
    }

    let top_modules: Vec<&ModuleDef> = source_files
        .iter()
        .flat_map(|f| &f.modules)
        .filter(|m| m.name != "join" && !m.items.is_empty() && !instantiated.contains(&m.name))
        .collect();

    if top_modules.is_empty() {
        if let Some(first_file) = source_files.first() {
            let loc = first_file
                .modules
                .first()
                .map(|m| m.span.to_location())
                .unwrap_or(SourceLocation {
                    file: "".into(),
                    line: 1,
                    col: 1,
                });
            diags.push(Diagnostic {
                stage: "lint".into(),
                code: "top-module".into(),
                severity: "error".into(),
                location: loc,
                subject: Subject {
                    kind: "component".into(),
                    name: None,
                    path: None,
                    id: None,
                    ref_des: None,
                    pad: None,
                },
                related: vec![],
                expected: Some(serde_json::json!(1)),
                actual: Some(serde_json::json!(0)),
                fix: None,
                message: "No top module found".into(),
            });
        }
    } else if top_modules.len() > 1 {
        let first = top_modules[0];
        let second = top_modules[1];
        diags.push(Diagnostic {
            stage: "lint".into(),
            code: "top-module".into(),
            severity: "error".into(),
            location: second.span.to_location(),
            subject: Subject {
                kind: "component".into(),
                name: Some(second.name.clone()),
                path: Some(second.name.clone()),
                id: None,
                ref_des: None,
                pad: None,
            },
            related: vec![Related {
                role: "other-top".into(),
                name: Some(first.name.clone()),
                location: Some(first.span.to_location()),
            }],
            expected: Some(serde_json::json!(1)),
            actual: Some(serde_json::json!(top_modules.len())),
            fix: None,
            message: format!("Expected exactly 1 top module, but found {}", top_modules.len()),
        });
    }

    // Check modules individually
    for file in source_files {
        for m in &file.modules {
            check_module(m, &module_map, &mut diags);
        }
    }

    diags
}

fn check_module(
    module: &ModuleDef,
    module_map: &HashMap<String, Vec<&ModuleDef>>,
    diags: &mut Vec<Diagnostic>,
) {
    let is_leaf = module.items.is_empty() && module.name != "join";

    // 4. unknown-attr on module
    for attr in &module.attrs {
        check_attr_target(attr, "module", is_leaf, diags);
        check_unit(attr, diags);
    }

    if is_leaf {
        // 11. leaf-incomplete (missing prefix)
        let has_prefix = module.attrs.iter().any(|a| a.key == "prefix");
        if !has_prefix {
            diags.push(Diagnostic {
                stage: "lint".into(),
                code: "leaf-incomplete".into(),
                severity: "error".into(),
                location: module.span.to_location(),
                subject: Subject {
                    kind: "component".into(),
                    name: Some(module.name.clone()),
                    path: Some(module.name.clone()),
                    id: None,
                    ref_des: None,
                    pad: None,
                },
                related: vec![],
                expected: None,
                actual: None,
                fix: Some("Add prefix attribute such as (* prefix = \"U\" *)".into()),
                message: format!("Leaf module '{}' does not specify a prefix attribute", module.name),
            });
        }

        // Ports in leaf
        let mut seen_pads: HashMap<String, SourceLocation> = HashMap::new();
        for port in &module.ports {
            for attr in &port.attrs {
                check_attr_target(attr, "port", is_leaf, diags);
                check_unit(attr, diags);
            }

            let pad_attr = port.attrs.iter().find(|a| a.key == "pad");
            let etype_attr = port.attrs.iter().find(|a| a.key == "etype");

            // 11. leaf-incomplete (port pad / etype)
            if pad_attr.is_none() || etype_attr.is_none() {
                diags.push(Diagnostic {
                    stage: "lint".into(),
                    code: "leaf-incomplete".into(),
                    severity: "error".into(),
                    location: port.span.to_location(),
                    subject: Subject {
                        kind: "pin".into(),
                        name: Some(port.name.clone()),
                        path: Some(format!("{}.{}", module.name, port.name)),
                        id: None,
                        ref_des: None,
                        pad: None,
                    },
                    related: vec![],
                    expected: None,
                    actual: None,
                    fix: Some(format!("Add (* pad = \"...\", etype = \"...\" *) to port '{}'", port.name)),
                    message: format!("Port '{}' in leaf module does not specify pad or etype attribute", port.name),
                });
            }

            // 23. decouple-multi-pad
            let is_decouple_required = port
                .attrs
                .iter()
                .any(|a| a.key == "decouple" && a.value.as_deref() == Some("required"));

            if let Some(pa) = pad_attr {
                if let Some(val) = &pa.value {
                    let pads: Vec<&str> = val.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
                    if is_decouple_required && pads.len() > 1 {
                        diags.push(Diagnostic {
                            stage: "lint".into(),
                            code: "decouple-multi-pad".into(),
                            severity: "error".into(),
                            location: port.span.to_location(),
                            subject: Subject {
                                kind: "pin".into(),
                                name: Some(port.name.clone()),
                                path: Some(format!("{}.{}", module.name, port.name)),
                                id: None,
                                ref_des: None,
                                pad: None,
                            },
                            related: vec![Related {
                                role: "pin-decl".into(),
                                name: Some(port.name.clone()),
                                location: Some(port.span.to_location()),
                            }],
                            expected: None,
                            actual: None,
                            fix: Some("Split port into separate ports per pad".into()),
                            message: format!("Port '{}' with decouple = \"required\" has multiple pads; split into per-pad ports", port.name),
                        });
                    }

                    // 12. pad-duplicate
                    for p in pads {
                        if let Some(prev_loc) = seen_pads.get(p) {
                            diags.push(Diagnostic {
                                stage: "lint".into(),
                                code: "pad-duplicate".into(),
                                severity: "error".into(),
                                location: port.span.to_location(),
                                subject: Subject {
                                    kind: "pin".into(),
                                    name: Some(port.name.clone()),
                                    path: Some(format!("{}.{}", module.name, port.name)),
                                    id: None,
                                    ref_des: None,
                                    pad: Some(p.to_string()),
                                },
                                related: vec![Related {
                                    role: "previous-pad".into(),
                                    name: Some(p.to_string()),
                                    location: Some(prev_loc.clone()),
                                }],
                                expected: None,
                                actual: None,
                                fix: None,
                                message: format!("Pad number '{}' is assigned to multiple ports", p),
                            });
                        } else {
                            seen_pads.insert(p.to_string(), port.span.to_location());
                        }
                    }
                }
            }
        }

        // 13. pad-count (if footprint specifies pad count hint, e.g., SOIC-8 has 8 pads)
        if let Some(fp_attr) = module.attrs.iter().find(|a| a.key == "footprint") {
            if let Some(fp) = &fp_attr.value {
                if let Some(expected_count) = extract_expected_pad_count(fp) {
                    if seen_pads.len() != expected_count {
                        diags.push(Diagnostic {
                            stage: "lint".into(),
                            code: "pad-count".into(),
                            severity: "error".into(),
                            location: module.span.to_location(),
                            subject: Subject {
                                kind: "component".into(),
                                name: Some(module.name.clone()),
                                path: Some(module.name.clone()),
                                id: None,
                                ref_des: None,
                                pad: None,
                            },
                            related: vec![],
                            expected: Some(serde_json::json!(expected_count)),
                            actual: Some(serde_json::json!(seen_pads.len())),
                            fix: None,
                            message: format!("Leaf pad count ({}) does not match footprint pad count ({})", seen_pads.len(), expected_count),
                        });
                    }
                }
            }
        }

        return;
    }

    // Non-leaf module checks
    let mut declared_wires: HashMap<String, &WireDecl> = HashMap::new();
    let mut declared_ports: HashMap<String, &PortDef> = HashMap::new();
    let mut hub_wires: HashMap<String, &WireDecl> = HashMap::new();
    let mut diffpair_map: HashMap<String, Vec<String>> = HashMap::new();

    for p in &module.ports {
        declared_ports.insert(p.name.clone(), p);
        for attr in &p.attrs {
            check_attr_target(attr, "port", false, diags);
            check_unit(attr, diags);
        }
    }

    for item in &module.items {
        match item {
            Item::Wire(wire) => {
                declared_wires.insert(wire.name.clone(), wire);
                for attr in &wire.attrs {
                    check_attr_target(attr, "wire", false, diags);
                    check_unit(attr, diags);
                    if attr.key == "nearby" {
                        hub_wires.insert(wire.name.clone(), wire);
                    } else if attr.key == "diffpair" {
                        if let Some(dp) = &attr.value {
                            diffpair_map.entry(dp.clone()).or_default().push(wire.name.clone());
                        }
                    }
                }
            }
            Item::Instance(inst) => {
                for attr in &inst.attrs {
                    check_attr_target(attr, "instance", false, diags);
                    check_unit(attr, diags);
                }
            }
        }
    }

    // 28. diffpair-invalid
    for (dp_name, wires) in &diffpair_map {
        let is_valid_pair = wires.len() == 2 && {
            let (w1, w2) = (&wires[0], &wires[1]);
            (w1.ends_with("_P") && w2.ends_with("_N"))
                || (w1.ends_with("_N") && w2.ends_with("_P"))
                || (w1.ends_with('+') && w2.ends_with('-'))
                || (w1.ends_with('-') && w2.ends_with('+'))
        };
        if !is_valid_pair {
            let wire_decl = declared_wires.get(&wires[0]).unwrap();
            diags.push(Diagnostic {
                stage: "lint".into(),
                code: "diffpair-invalid".into(),
                severity: "error".into(),
                location: wire_decl.span.to_location(),
                subject: Subject {
                    kind: "net".into(),
                    name: Some(dp_name.clone()),
                    path: Some(format!("{}.{}", module.name, dp_name)),
                    id: None,
                    ref_des: None,
                    pad: None,
                },
                related: vec![],
                expected: None,
                actual: None,
                fix: Some("Ensure differential pair names end with _P / _N or + / - pairs".into()),
                message: format!("Differential pair '{}' does not form a valid 2-wire pair", dp_name),
            });
        }
    }

    let mut pin_wires_used: HashSet<String> = HashSet::new();
    let mut joins_by_child: HashMap<String, Vec<&Instance>> = HashMap::new();
    let mut joins_by_parent: HashMap<String, Vec<&Instance>> = HashMap::new();
    let mut join_edges: Vec<(String, String, &Instance)> = Vec::new(); // (child, parent, inst)

    // Check instances
    let mut seen_ids: HashMap<String, SourceLocation> = HashMap::new();
    let mut seen_refs: HashMap<String, SourceLocation> = HashMap::new();
    let mut net_pad_counts: HashMap<String, Vec<String>> = HashMap::new(); // net -> Vec<pad_desc>
    let mut net_drivers: HashMap<String, usize> = HashMap::new();
    let mut net_power_outs: HashMap<String, Vec<SourceLocation>> = HashMap::new();

    for item in &module.items {
        if let Item::Instance(inst) = item {
            // 29. duplicate-identity (id, ref)
            for attr in &inst.attrs {
                if attr.key == "id" {
                    if let Some(id_val) = &attr.value {
                        if let Some(prev_loc) = seen_ids.get(id_val) {
                            diags.push(Diagnostic {
                                stage: "lint".into(),
                                code: "duplicate-identity".into(),
                                severity: "error".into(),
                                location: attr.span.to_location(),
                                subject: Subject {
                                    kind: "component".into(),
                                    name: Some(inst.instance_name.clone()),
                                    path: Some(format!("{}.{}", module.name, inst.instance_name)),
                                    id: Some(id_val.clone()),
                                    ref_des: None,
                                    pad: None,
                                },
                                related: vec![Related {
                                    role: "previous-id".into(),
                                    name: Some(id_val.clone()),
                                    location: Some(prev_loc.clone()),
                                }],
                                expected: None,
                                actual: None,
                                fix: None,
                                message: format!("Duplicate component ID '{}'", id_val),
                            });
                        } else {
                            seen_ids.insert(id_val.clone(), attr.span.to_location());
                        }
                    }
                } else if attr.key == "ref" {
                    if let Some(ref_val) = &attr.value {
                        if let Some(prev_loc) = seen_refs.get(ref_val) {
                            diags.push(Diagnostic {
                                stage: "lint".into(),
                                code: "duplicate-identity".into(),
                                severity: "error".into(),
                                location: attr.span.to_location(),
                                subject: Subject {
                                    kind: "component".into(),
                                    name: Some(inst.instance_name.clone()),
                                    path: Some(format!("{}.{}", module.name, inst.instance_name)),
                                    id: None,
                                    ref_des: Some(ref_val.clone()),
                                    pad: None,
                                },
                                related: vec![Related {
                                    role: "previous-ref".into(),
                                    name: Some(ref_val.clone()),
                                    location: Some(prev_loc.clone()),
                                }],
                                expected: None,
                                actual: None,
                                fix: None,
                                message: format!("Duplicate reference designator '{}'", ref_val),
                            });
                        } else {
                            seen_refs.insert(ref_val.clone(), attr.span.to_location());
                        }
                    }
                }
            }

            if inst.module_name == "join" {
                // 21. join-name check: must be j_<C_wire>
                let mut p_wire = None;
                let mut c_wire = None;

                for conn in &inst.connections {
                    let w_name = conn.expr.as_ref().and_then(|e| match e {
                        Expr::Ref(r) => Some(r.ident.clone()),
                        _ => None,
                    });
                    if conn.port_name == "P" {
                        p_wire = w_name;
                    } else if conn.port_name == "C" {
                        c_wire = w_name;
                    }
                }

                if let Some(c) = &c_wire {
                    let expected_name = format!("j_{}", c);
                    if inst.instance_name != expected_name {
                        diags.push(Diagnostic {
                            stage: "lint".into(),
                            code: "join-name".into(),
                            severity: "error".into(),
                            location: inst.span.to_location(),
                            subject: Subject {
                                kind: "join".into(),
                                name: Some(inst.instance_name.clone()),
                                path: Some(format!("{}.{}", module.name, inst.instance_name)),
                                id: None,
                                ref_des: None,
                                pad: None,
                            },
                            related: vec![Related {
                                role: "join-decl".into(),
                                name: Some(inst.instance_name.clone()),
                                location: Some(inst.span.to_location()),
                            }],
                            expected: Some(serde_json::json!(expected_name)),
                            actual: Some(serde_json::json!(inst.instance_name.clone())),
                            fix: Some(format!("Change join instance name to '{}'", expected_name)),
                            message: format!("Join instance name must be '{}'", expected_name),
                        });
                    }
                    joins_by_child.entry(c.clone()).or_default().push(inst);
                }

                if let (Some(p), Some(c)) = (&p_wire, &c_wire) {
                    joins_by_parent.entry(p.clone()).or_default().push(inst);
                    join_edges.push((c.clone(), p.clone(), inst));
                }

                continue;
            }

            // Regular instance
            // 10. recursive-instance
            if inst.module_name == module.name {
                diags.push(Diagnostic {
                    stage: "lint".into(),
                    code: "recursive-instance".into(),
                    severity: "error".into(),
                    location: inst.span.to_location(),
                    subject: Subject {
                        kind: "component".into(),
                        name: Some(inst.instance_name.clone()),
                        path: Some(format!("{}.{}", module.name, inst.instance_name)),
                        id: None,
                        ref_des: None,
                        pad: None,
                    },
                    related: vec![],
                    expected: None,
                    actual: None,
                    fix: None,
                    message: format!("Module '{}' recursively instantiates itself", module.name),
                });
            }

            let target_mod = match module_map.get(&inst.module_name).and_then(|v| v.first()) {
                Some(m) => *m,
                None => continue,
            };

            let target_is_leaf = target_mod.items.is_empty();

            // 14. footprint-missing
            if target_is_leaf {
                let inst_fp = inst.attrs.iter().find(|a| a.key == "footprint");
                let leaf_fp = target_mod.attrs.iter().find(|a| a.key == "footprint");
                if inst_fp.is_none() && leaf_fp.is_none() {
                    diags.push(Diagnostic {
                        stage: "lint".into(),
                        code: "footprint-missing".into(),
                        severity: "error".into(),
                        location: inst.span.to_location(),
                        subject: Subject {
                            kind: "component".into(),
                            name: Some(inst.instance_name.clone()),
                            path: Some(format!("{}.{}", module.name, inst.instance_name)),
                            id: None,
                            ref_des: None,
                            pad: None,
                        },
                        related: vec![],
                        expected: None,
                        actual: None,
                        fix: Some("Specify (* footprint = \"...\" *) on the instance".into()),
                        message: format!("Footprint not specified for component '{}'", inst.instance_name),
                    });
                }
            }

            // 5. unknown-param & 6. param-missing
            for p_ovr in &inst.param_overrides {
                let exists = target_mod.params.iter().any(|p| p.name == p_ovr.name);
                if !exists {
                    diags.push(Diagnostic {
                        stage: "lint".into(),
                        code: "unknown-param".into(),
                        severity: "error".into(),
                        location: p_ovr.span.to_location(),
                        subject: Subject {
                            kind: "component".into(),
                            name: Some(inst.instance_name.clone()),
                            path: Some(format!("{}.{}", module.name, inst.instance_name)),
                            id: None,
                            ref_des: None,
                            pad: None,
                        },
                        related: vec![],
                        expected: None,
                        actual: None,
                        fix: None,
                        message: format!("Parameter '{}' overridden on instance does not exist in module '{}'", p_ovr.name, inst.module_name),
                    });
                }
            }

            for p_def in &target_mod.params {
                if p_def.value.is_empty() {
                    // required parameter
                    let overridden = inst.param_overrides.iter().any(|p| p.name == p_def.name);
                    if !overridden {
                        diags.push(Diagnostic {
                            stage: "lint".into(),
                            code: "param-missing".into(),
                            severity: "error".into(),
                            location: inst.span.to_location(),
                            subject: Subject {
                                kind: "component".into(),
                                name: Some(inst.instance_name.clone()),
                                path: Some(format!("{}.{}", module.name, inst.instance_name)),
                                id: None,
                                ref_des: None,
                                pad: None,
                            },
                            related: vec![],
                            expected: None,
                            actual: None,
                            fix: Some(format!("#(.{}(\"...\"))", p_def.name)),
                            message: format!("Required parameter '{}' is not overridden", p_def.name),
                        });
                    }
                }
            }

            // 3. missing-port
            for port in &target_mod.ports {
                let conn_opt = inst.connections.iter().find(|c| c.port_name == port.name);
                if conn_opt.is_none() {
                    diags.push(Diagnostic {
                        stage: "lint".into(),
                        code: "missing-port".into(),
                        severity: "error".into(),
                        location: inst.span.to_location(),
                        subject: Subject {
                            kind: "component".into(),
                            name: Some(inst.instance_name.clone()),
                            path: Some(format!("{}.{}", module.name, inst.instance_name)),
                            id: None,
                            ref_des: None,
                            pad: None,
                        },
                        related: vec![],
                        expected: None,
                        actual: None,
                        fix: Some(format!("Add .{}()", port.name)),
                        message: format!("Port '{}' connection missing (use .{}() for deliberate unconnected)", port.name, port.name),
                    });
                    continue;
                }

                let conn = conn_opt.unwrap();
                let port_etype = port
                    .attrs
                    .iter()
                    .find(|a| a.key == "etype")
                    .and_then(|a| a.value.as_deref())
                    .unwrap_or("passive");

                // 25. power-unconnected
                if port_etype == "power_in" && conn.expr.is_none() {
                    diags.push(Diagnostic {
                        stage: "lint".into(),
                        code: "power-unconnected".into(),
                        severity: "error".into(),
                        location: conn.span.to_location(),
                        subject: Subject {
                            kind: "pin".into(),
                            name: Some(format!("{}.{}", inst.instance_name, port.name)),
                            path: Some(format!("{}.{}.{}", module.name, inst.instance_name, port.name)),
                            id: None,
                            ref_des: None,
                            pad: None,
                        },
                        related: vec![],
                        expected: None,
                        actual: None,
                        fix: None,
                        message: format!("power_in pin '{}.{}' is unconnected", inst.instance_name, port.name),
                    });
                }

                // 27. nc-connected
                if port_etype == "no_connect" && conn.expr.is_some() {
                    diags.push(Diagnostic {
                        stage: "lint".into(),
                        code: "nc-connected".into(),
                        severity: "error".into(),
                        location: conn.span.to_location(),
                        subject: Subject {
                            kind: "pin".into(),
                            name: Some(format!("{}.{}", inst.instance_name, port.name)),
                            path: Some(format!("{}.{}.{}", module.name, inst.instance_name, port.name)),
                            id: None,
                            ref_des: None,
                            pad: None,
                        },
                        related: vec![],
                        expected: None,
                        actual: None,
                        fix: Some(format!("Change to .{}()", port.name)),
                        message: format!("no_connect pin '{}.{}' is connected to a net", inst.instance_name, port.name),
                    });
                }

                // Check connection expression
                if let Some(expr) = &conn.expr {
                    let mut referenced_wires = Vec::new();
                    match expr {
                        Expr::Ref(r) => referenced_wires.push(r.ident.clone()),
                        Expr::Concat(list) => {
                            for r in list {
                                referenced_wires.push(r.ident.clone());
                            }
                        }
                    }

                    for w in referenced_wires {
                        // 1. undeclared-net
                        let is_declared = declared_wires.contains_key(&w) || declared_ports.contains_key(&w);
                        if !is_declared {
                            diags.push(Diagnostic {
                                stage: "lint".into(),
                                code: "undeclared-net".into(),
                                severity: "error".into(),
                                location: conn.span.to_location(),
                                subject: Subject {
                                    kind: "net".into(),
                                    name: Some(w.clone()),
                                    path: Some(format!("{}.{}", module.name, w)),
                                    id: None,
                                    ref_des: None,
                                    pad: None,
                                },
                                related: vec![],
                                expected: None,
                                actual: None,
                                fix: Some(format!("Declare 'wire {};'", w)),
                                message: format!("Reference to undeclared net '{}'", w),
                            });
                        }

                        // 18. hub-direct-pad
                        if hub_wires.contains_key(&w) {
                            let hub_decl = declared_wires.get(&w).unwrap();
                            diags.push(Diagnostic {
                                stage: "lint".into(),
                                code: "hub-direct-pad".into(),
                                severity: "error".into(),
                                location: conn.span.to_location(),
                                subject: Subject {
                                    kind: "hub".into(),
                                    name: Some(w.clone()),
                                    path: Some(format!("{}.{}", module.name, w)),
                                    id: None,
                                    ref_des: None,
                                    pad: None,
                                },
                                related: vec![
                                    Related {
                                        role: "hub-decl".into(),
                                        name: Some(w.clone()),
                                        location: Some(hub_decl.span.to_location()),
                                    },
                                    Related {
                                        role: "offending-pin".into(),
                                        name: Some(format!("{}.{}", inst.instance_name, port.name)),
                                        location: Some(conn.span.to_location()),
                                    },
                                ],
                                expected: None,
                                actual: None,
                                fix: Some(format!("Connect {}.{} via pin-wire {}_{} and add join j_{}_{} (.P({}), .C({}_{}));",
                                    inst.instance_name, port.name, inst.instance_name, port.name, inst.instance_name, port.name, w, inst.instance_name, port.name)),
                                message: format!("Pad of {}.{} is directly connected to nearby hub '{}'", inst.instance_name, port.name, w),
                            });
                        }

                        // Collect drivers and nets
                        if port_etype == "power_out" {
                            net_power_outs.entry(w.clone()).or_default().push(conn.span.to_location());
                            *net_drivers.entry(w.clone()).or_default() += 1;
                        } else if port_etype == "output"
                            || port_etype == "bidirectional"
                            || port_etype == "power_in"
                            || port_etype == "open_collector"
                            || port_etype == "open_emitter"
                            || port_etype == "tri_state"
                        {
                            *net_drivers.entry(w.clone()).or_default() += 1;
                        }

                        net_pad_counts.entry(w.clone()).or_default().push(format!("{}.{}", inst.instance_name, port.name));

                        // 16. pin-wire-name check if wire is joined to a hub
                        let is_hub_child = joins_by_child.contains_key(&w);
                        if is_hub_child {
                            pin_wires_used.insert(w.clone());
                            let expected_wire_name = format!("{}_{}", inst.instance_name, port.name);
                            if w != expected_wire_name {
                                let wire_decl = declared_wires.get(&w);
                                diags.push(Diagnostic {
                                    stage: "lint".into(),
                                    code: "pin-wire-name".into(),
                                    severity: "error".into(),
                                    location: conn.span.to_location(),
                                    subject: Subject {
                                        kind: "pin-wire".into(),
                                        name: Some(w.clone()),
                                        path: Some(format!("{}.{}", module.name, w)),
                                        id: None,
                                        ref_des: None,
                                        pad: None,
                                    },
                                    related: vec![
                                        Related {
                                            role: "wire-decl".into(),
                                            name: Some(w.clone()),
                                            location: wire_decl.map(|wd| wd.span.to_location()),
                                        },
                                        Related {
                                            role: "pin".into(),
                                            name: Some(format!("{}.{}", inst.instance_name, port.name)),
                                            location: Some(conn.span.to_location()),
                                        },
                                    ],
                                    expected: Some(serde_json::json!(expected_wire_name)),
                                    actual: Some(serde_json::json!(w.clone())),
                                    fix: Some(format!("Rename wire to '{}'", expected_wire_name)),
                                    message: format!("Pin-wire name '{}' does not match connected pin '{}.{}'", w, inst.instance_name, port.name),
                                });
                            }
                        }

                        // 22. decouple-missing
                        let is_decouple_required = port.attrs.iter().any(|a| a.key == "decouple" && a.value.as_deref() == Some("required"));
                        if is_decouple_required && !is_hub_child {
                            diags.push(Diagnostic {
                                stage: "lint".into(),
                                code: "decouple-missing".into(),
                                severity: "error".into(),
                                location: conn.span.to_location(),
                                subject: Subject {
                                    kind: "pin".into(),
                                    name: Some(format!("{}.{}", inst.instance_name, port.name)),
                                    path: Some(format!("{}.{}.{}", module.name, inst.instance_name, port.name)),
                                    id: None,
                                    ref_des: None,
                                    pad: None,
                                },
                                related: vec![
                                    Related {
                                        role: "pin-decl".into(),
                                        name: Some(port.name.clone()),
                                        location: Some(port.span.to_location()),
                                    },
                                    Related {
                                        role: "connection".into(),
                                        name: Some(w.clone()),
                                        location: Some(conn.span.to_location()),
                                    },
                                ],
                                expected: None,
                                actual: None,
                                fix: Some(format!("Connect {}.{} via pin-wire and join to decoupling hub", inst.instance_name, port.name)),
                                message: format!("Pin '{}.{}' with decouple = \"required\" is not joined to a nearby hub via a pin-wire", inst.instance_name, port.name),
                            });
                        }
                    }

                    // 2. width-mismatch
                    let port_width = port.range.as_ref().map(|r| r.msb + 1).unwrap_or(1);
                    let conn_width = match expr {
                        Expr::Ref(r) => {
                            if let Some(idx) = &r.index {
                                match idx {
                                    RefIndex::Single(_) => 1,
                                    RefIndex::Range(msb, lsb) => msb - lsb + 1,
                                }
                            } else if let Some(w) = declared_wires.get(&r.ident) {
                                w.range.as_ref().map(|rng| rng.msb + 1).unwrap_or(1)
                            } else if let Some(p) = declared_ports.get(&r.ident) {
                                p.range.as_ref().map(|rng| rng.msb + 1).unwrap_or(1)
                            } else {
                                1
                            }
                        }
                        Expr::Concat(list) => list.len() as u32,
                    };

                    if port_width != conn_width {
                        diags.push(Diagnostic {
                            stage: "lint".into(),
                            code: "width-mismatch".into(),
                            severity: "error".into(),
                            location: conn.span.to_location(),
                            subject: Subject {
                                kind: "pin".into(),
                                name: Some(format!("{}.{}", inst.instance_name, port.name)),
                                path: Some(format!("{}.{}.{}", module.name, inst.instance_name, port.name)),
                                id: None,
                                ref_des: None,
                                pad: None,
                            },
                            related: vec![],
                            expected: Some(serde_json::json!(port_width)),
                            actual: Some(serde_json::json!(conn_width)),
                            fix: None,
                            message: format!("Bit width mismatch for port '{}.{}' (port width {}, connection width {})", inst.instance_name, port.name, port_width, conn_width),
                        });
                    }
                }
            }
        }
    }

    // 20. join-cycle check
    let mut parent_map: HashMap<String, String> = HashMap::new();
    for (child, parent, inst) in &join_edges {
        let mut curr = parent.clone();
        let mut cycle_detected = false;
        let mut cycle_path = vec![curr.clone()];

        while let Some(next) = parent_map.get(&curr) {
            if next == child {
                cycle_detected = true;
                cycle_path.push(next.clone());
                break;
            }
            cycle_path.push(next.clone());
            curr = next.clone();
        }

        if cycle_detected {
            diags.push(Diagnostic {
                stage: "lint".into(),
                code: "join-cycle".into(),
                severity: "error".into(),
                location: inst.span.to_location(),
                subject: Subject {
                    kind: "join".into(),
                    name: Some(inst.instance_name.clone()),
                    path: Some(format!("{}.{}", module.name, inst.instance_name)),
                    id: None,
                    ref_des: None,
                    pad: None,
                },
                related: vec![Related {
                    role: "cycle".into(),
                    name: Some(cycle_path.join(" -> ")),
                    location: Some(inst.span.to_location()),
                }],
                expected: None,
                actual: None,
                fix: None,
                message: format!("Join instance '{}' forms a cycle", inst.instance_name),
            });
        } else {
            parent_map.insert(child.clone(), parent.clone());
        }
    }

    // 15. pin-wire-shape check
    for wire_name in &pin_wires_used {
        let pad_count = net_pad_counts.get(wire_name).map(|v| v.len()).unwrap_or(0);
        let join_count = joins_by_child.get(wire_name).map(|v| v.len()).unwrap_or(0);
        if pad_count != 1 || join_count != 1 {
            let wire_decl = declared_wires.get(wire_name);
            diags.push(Diagnostic {
                stage: "lint".into(),
                code: "pin-wire-shape".into(),
                severity: "error".into(),
                location: wire_decl.map(|w| w.span.to_location()).unwrap_or(module.span.to_location()),
                subject: Subject {
                    kind: "pin-wire".into(),
                    name: Some(wire_name.clone()),
                    path: Some(format!("{}.{}", module.name, wire_name)),
                    id: None,
                    ref_des: None,
                    pad: None,
                },
                related: vec![
                    Related {
                        role: "wire-decl".into(),
                        name: Some(wire_name.clone()),
                        location: wire_decl.map(|w| w.span.to_location()),
                    },
                ],
                expected: None,
                actual: None,
                fix: Some("Ensure pin-wire has exactly 1 pad connection and exactly 1 join to a hub".into()),
                message: format!("Pin-wire '{}' must have exactly 1 pad (currently {}) and 1 join to hub (currently {})", wire_name, pad_count, join_count),
            });
        }
    }

    // 24. constraint-conflict check
    let mut net_widths: HashMap<String, Vec<(String, SourceLocation)>> = HashMap::new();
    for item in &module.items {
        if let Item::Wire(wire) = item {
            for attr in &wire.attrs {
                if attr.key == "width" {
                    if let Some(val) = &attr.value {
                        net_widths.entry(wire.name.clone()).or_default().push((val.clone(), attr.span.to_location()));
                    }
                }
            }
        }
    }
    for (net_name, widths) in &net_widths {
        if widths.len() > 1 && widths.windows(2).any(|w| w[0].0 != w[1].0) {
            diags.push(Diagnostic {
                stage: "lint".into(),
                code: "constraint-conflict".into(),
                severity: "error".into(),
                location: widths[1].1.clone(),
                subject: Subject {
                    kind: "net".into(),
                    name: Some(net_name.clone()),
                    path: Some(format!("{}.{}", module.name, net_name)),
                    id: None,
                    ref_des: None,
                    pad: None,
                },
                related: vec![Related {
                    role: "conflicting-width".into(),
                    name: Some(widths[0].0.clone()),
                    location: Some(widths[0].1.clone()),
                }],
                expected: None,
                actual: None,
                fix: Some("Unify width constraints on net '{}'".into()),
                message: format!("Conflicting width constraints specified for net '{}'", net_name),
            });
        }
    }

    // 17. pin-wire-attr check
    for wire_name in &pin_wires_used {
        if let Some(wire) = declared_wires.get(wire_name) {
            if !wire.attrs.is_empty() {
                diags.push(Diagnostic {
                    stage: "lint".into(),
                    code: "pin-wire-attr".into(),
                    severity: "error".into(),
                    location: wire.span.to_location(),
                    subject: Subject {
                        kind: "pin-wire".into(),
                        name: Some(wire.name.clone()),
                        path: Some(format!("{}.{}", module.name, wire.name)),
                        id: None,
                        ref_des: None,
                        pad: None,
                    },
                    related: vec![Related {
                        role: "wire-decl".into(),
                        name: Some(wire.name.clone()),
                        location: Some(wire.span.to_location()),
                    }],
                    expected: None,
                    actual: None,
                    fix: Some("Remove attributes from pin-wire".into()),
                    message: format!("Pin-wire '{}' cannot have attributes", wire.name),
                });
            }
        }
    }

    // 19. hub-too-few check
    for (hub_name, hub_wire) in &hub_wires {
        let members = joins_by_parent.get(hub_name).map(|v| v.len()).unwrap_or(0);
        if members < 2 {
            diags.push(Diagnostic {
                stage: "lint".into(),
                code: "hub-too-few".into(),
                severity: "error".into(),
                location: hub_wire.span.to_location(),
                subject: Subject {
                    kind: "hub".into(),
                    name: Some(hub_name.clone()),
                    path: Some(format!("{}.{}", module.name, hub_name)),
                    id: None,
                    ref_des: None,
                    pad: None,
                },
                related: vec![Related {
                    role: "hub-decl".into(),
                    name: Some(hub_name.clone()),
                    location: Some(hub_wire.span.to_location()),
                }],
                expected: Some(serde_json::json!(2)),
                actual: Some(serde_json::json!(members)),
                fix: None,
                message: format!("Nearby hub '{}' has fewer than 2 pin-wires attached (currently {})", hub_name, members),
            });
        }
    }

    // 26. power-conflict check
    for (net_name, locations) in &net_power_outs {
        if locations.len() > 1 {
            diags.push(Diagnostic {
                stage: "lint".into(),
                code: "power-conflict".into(),
                severity: "error".into(),
                location: locations[1].clone(),
                subject: Subject {
                    kind: "net".into(),
                    name: Some(net_name.clone()),
                    path: Some(format!("{}.{}", module.name, net_name)),
                    id: None,
                    ref_des: None,
                    pad: None,
                },
                related: vec![Related {
                    role: "power-out".into(),
                    name: Some(net_name.clone()),
                    location: Some(locations[0].clone()),
                }],
                expected: Some(serde_json::json!(1)),
                actual: Some(serde_json::json!(locations.len())),
                fix: None,
                message: format!("Net '{}' has multiple power_out driving pins connected", net_name),
            });
        }
    }

    // 30. single-pin-net (warning)
    for (net_name, pads) in &net_pad_counts {
        if pads.len() == 1
            && !declared_ports.contains_key(net_name)
            && !joins_by_child.contains_key(net_name)
            && !joins_by_parent.contains_key(net_name)
        {
            let loc = declared_wires.get(net_name).map(|w| w.span.to_location()).unwrap_or(module.span.to_location());
            diags.push(Diagnostic {
                stage: "lint".into(),
                code: "single-pin-net".into(),
                severity: "warning".into(),
                location: loc,
                subject: Subject {
                    kind: "net".into(),
                    name: Some(net_name.clone()),
                    path: Some(format!("{}.{}", module.name, net_name)),
                    id: None,
                    ref_des: None,
                    pad: None,
                },
                related: vec![],
                expected: None,
                actual: None,
                fix: None,
                message: format!("Net '{}' has only 1 component pad connected", net_name),
            });
        }
    }

    // 31. undriven-net (warning)
    for (net_name, pads) in &net_pad_counts {
        let drivers = net_drivers.get(net_name).copied().unwrap_or(0);
        if drivers == 0 && pads.len() > 1 && !declared_ports.contains_key(net_name) {
            let loc = declared_wires.get(net_name).map(|w| w.span.to_location()).unwrap_or(module.span.to_location());
            diags.push(Diagnostic {
                stage: "lint".into(),
                code: "undriven-net".into(),
                severity: "warning".into(),
                location: loc,
                subject: Subject {
                    kind: "net".into(),
                    name: Some(net_name.clone()),
                    path: Some(format!("{}.{}", module.name, net_name)),
                    id: None,
                    ref_des: None,
                    pad: None,
                },
                related: vec![],
                expected: None,
                actual: None,
                fix: None,
                message: format!("Net '{}' has no driving pins (only input pins)", net_name),
            });
        }
    }
}

fn check_attr_target(attr: &Attr, target_kind: &str, is_leaf: bool, diags: &mut Vec<Diagnostic>) {
    let valid = match attr.key.as_str() {
        "footprint" => (target_kind == "module" && is_leaf) || target_kind == "instance",
        "mpn" => (target_kind == "module" && is_leaf) || target_kind == "instance",
        "prefix" => target_kind == "module" && is_leaf,
        "pad" => target_kind == "port" && is_leaf,
        "etype" => target_kind == "port",
        "decouple" => target_kind == "port" && is_leaf,
        "id" => target_kind == "instance",
        "ref" => target_kind == "instance",
        "dnp" => target_kind == "instance",
        "nearby" => target_kind == "wire",
        "width" => target_kind == "wire" || target_kind == "port",
        "current" => target_kind == "wire" || target_kind == "port",
        "netclass" => target_kind == "wire" || target_kind == "port",
        "diffpair" => target_kind == "wire",
        _ => false,
    };

    if !valid {
        diags.push(Diagnostic {
            stage: "lint".into(),
            code: "unknown-attr".into(),
            severity: "error".into(),
            location: attr.span.to_location(),
            subject: Subject {
                kind: target_kind.into(),
                name: Some(attr.key.clone()),
                path: None,
                id: None,
                ref_des: None,
                pad: None,
            },
            related: vec![],
            expected: None,
            actual: None,
            fix: None,
            message: format!("Unknown attribute '{}' or attribute attached to invalid target", attr.key),
        });
    }
}

fn check_unit(attr: &Attr, diags: &mut Vec<Diagnostic>) {
    if attr.key == "width" {
        if let Some(val) = &attr.value {
            let valid = (val.ends_with("mm") && val[..val.len() - 2].parse::<f64>().is_ok())
                || (val.ends_with("mil") && val[..val.len() - 3].parse::<f64>().is_ok());
            if !valid {
                diags.push(Diagnostic {
                    stage: "lint".into(),
                    code: "unit-invalid".into(),
                    severity: "error".into(),
                    location: attr.span.to_location(),
                    subject: Subject {
                        kind: "wire".into(),
                        name: Some(attr.key.clone()),
                        path: None,
                        id: None,
                        ref_des: None,
                        pad: None,
                    },
                    related: vec![],
                    expected: Some(serde_json::json!("'mm' or 'mil'")),
                    actual: Some(serde_json::json!(val)),
                    fix: Some("Specify unit (mm or mil, e.g. \"0.5mm\")".into()),
                    message: format!("Length '{}' is missing unit or has invalid unit", val),
                });
            }
        }
    } else if attr.key == "current" {
        if let Some(val) = &attr.value {
            let valid = (val.ends_with("mA") && val[..val.len() - 2].parse::<f64>().is_ok())
                || (val.ends_with('A') && val[..val.len() - 1].parse::<f64>().is_ok());
            if !valid {
                diags.push(Diagnostic {
                    stage: "lint".into(),
                    code: "unit-invalid".into(),
                    severity: "error".into(),
                    location: attr.span.to_location(),
                    subject: Subject {
                        kind: "wire".into(),
                        name: Some(attr.key.clone()),
                        path: None,
                        id: None,
                        ref_des: None,
                        pad: None,
                    },
                    related: vec![],
                    expected: Some(serde_json::json!("'A' or 'mA'")),
                    actual: Some(serde_json::json!(val)),
                    fix: Some("Specify unit (A or mA, e.g. \"1.5A\")".into()),
                    message: format!("Current '{}' is missing unit or has invalid unit", val),
                });
            }
        }
    }
}

fn extract_expected_pad_count(footprint: &str) -> Option<usize> {
    if footprint.contains("SOIC-8") || footprint.contains("DIP-8") {
        Some(8)
    } else if footprint.contains("SOT-23") {
        Some(3)
    } else if footprint.contains("QFP-32") {
        Some(32)
    } else {
        None
    }
}
