use crate::diagnostic::*;
use kinema_syntax::ast::*;
use std::collections::{HashMap, HashSet};

pub fn check_rules(source_files: &[SourceFile]) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    let project_dir = source_files
        .first()
        .and_then(|f| f.modules.first())
        .map(|m| std::path::Path::new(&m.span.file).parent().unwrap_or(std::path::Path::new(".")))
        .unwrap_or(std::path::Path::new("."));
    let fp_resolver = kinema_kicad::FootprintResolver::auto_discover(project_dir);

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
                location: Some(dup.span.to_location()),
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
                location: Some(loc),
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
            location: Some(second.span.to_location()),
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
            check_module(m, &module_map, &fp_resolver, &mut diags);
        }
    }

    diags
}

fn find_module_cycle<'a>(
    curr: &'a str,
    target: &'a str,
    module_map: &'a HashMap<String, Vec<&ModuleDef>>,
    visited: &mut HashSet<&'a str>,
    path: &mut Vec<&'a str>,
) -> bool {
    if curr == target {
        path.push(target);
        return true;
    }
    if !visited.insert(curr) {
        return false;
    }
    path.push(curr);
    if let Some(target_mods) = module_map.get(curr) {
        if let Some(m) = target_mods.first() {
            for item in &m.items {
                if let Item::Instance(child_inst) = item {
                    if child_inst.module_name == "join" {
                        continue;
                    }
                    if find_module_cycle(&child_inst.module_name, target, module_map, visited, path) {
                        return true;
                    }
                }
            }
        }
    }
    path.pop();
    false
}

fn check_module(
    module: &ModuleDef,
    module_map: &HashMap<String, Vec<&ModuleDef>>,
    fp_resolver: &kinema_kicad::FootprintResolver,
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
                location: Some(module.span.to_location()),
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
                    location: Some(port.span.to_location()),
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
                            location: Some(port.span.to_location()),
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
                                location: Some(port.span.to_location()),
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

        // 13. pad-count and 14. footprint-missing on leaf module
        if let Some(fp_attr) = module.attrs.iter().find(|a| a.key == "footprint") {
            if let Some(fp) = &fp_attr.value {
                let resolved = fp_resolver.resolve(fp);
                let expected_count = match resolved {
                    Ok(info) => Some(info.pad_count),
                    Err(kinema_kicad::FootprintResolveError::FootprintNotFound(_))
                    | Err(kinema_kicad::FootprintResolveError::LibraryNotFound(_)) => {
                        diags.push(Diagnostic {
                            stage: "lint".into(),
                            code: "footprint-missing".into(),
                            severity: "error".into(),
                            location: Some(module.span.to_location()),
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
                            actual: Some(serde_json::json!(fp)),
                            fix: Some(format!("Ensure footprint '{}' exists in library or fp-lib-table", fp)),
                            message: format!("Footprint '{}' does not exist in footprint libraries", fp),
                        });
                        None
                    }
                    _ => extract_expected_pad_count(fp),
                };

                if let Some(expected_count) = expected_count {
                    if seen_pads.len() != expected_count {
                        diags.push(Diagnostic {
                            stage: "lint".into(),
                            code: "pad-count".into(),
                            severity: "error".into(),
                            location: Some(module.span.to_location()),
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
                location: Some(wire_decl.span.to_location()),
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

    // Pass 1: Collect all join instances and validate join-name
    for item in &module.items {
        if let Item::Instance(inst) = item {
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
                            location: Some(inst.span.to_location()),
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
                location: Some(inst.span.to_location()),
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

    let resolve_canonical = |mut w: String| -> String {
        let mut visited = HashSet::new();
        visited.insert(w.clone());
        while let Some(parent) = parent_map.get(&w) {
            if visited.contains(parent) {
                break;
            }
            w = parent.clone();
            visited.insert(w.clone());
        }
        w
    };

    // Pass 2: Check regular instances
    let mut seen_ids: HashMap<String, SourceLocation> = HashMap::new();
    let mut seen_refs: HashMap<String, SourceLocation> = HashMap::new();
    let mut direct_pad_counts: HashMap<String, usize> = HashMap::new();
    let mut net_pad_counts: HashMap<String, Vec<String>> = HashMap::new(); // canonical net -> Vec<pad_desc>
    let mut net_drivers: HashMap<String, usize> = HashMap::new();
    let mut net_power_outs: HashMap<String, Vec<SourceLocation>> = HashMap::new();

    for item in &module.items {
        if let Item::Instance(inst) = item {
            if inst.module_name == "join" {
                continue;
            }

            // 29. duplicate-identity (id, ref)
            for attr in &inst.attrs {
                if attr.key == "id" {
                    if let Some(id_val) = &attr.value {
                        if let Some(prev_loc) = seen_ids.get(id_val) {
                            diags.push(Diagnostic {
                                stage: "lint".into(),
                                code: "duplicate-identity".into(),
                                severity: "error".into(),
                                location: Some(attr.span.to_location()),
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
                                location: Some(attr.span.to_location()),
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

            let has_explicit_ref = inst.attrs.iter().any(|a| a.key == "ref");
            if !has_explicit_ref {
                if let Some(target_mod) = module_map.get(&inst.module_name).and_then(|v| v.first()) {
                    let mut prefix = "U".to_string();
                    for a in &target_mod.attrs {
                        if a.key == "prefix" {
                            if let Some(p) = &a.value {
                                prefix = p.clone();
                            }
                        }
                    }
                    if inst.instance_name.starts_with(&prefix)
                        && inst.instance_name.len() > prefix.len()
                        && inst.instance_name[prefix.len()..].chars().all(|c| c.is_ascii_digit())
                    {
                        let ref_val = inst.instance_name.clone();
                        if let Some(prev_loc) = seen_refs.get(&ref_val) {
                            diags.push(Diagnostic {
                                stage: "lint".into(),
                                code: "duplicate-identity".into(),
                                severity: "error".into(),
                                location: Some(inst.span.to_location()),
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
                            seen_refs.insert(ref_val, inst.span.to_location());
                        }
                    }
                }
            }

            // Regular instance
            // 10. recursive-instance
            let is_direct_recursive = inst.module_name == module.name;
            let mut cycle_path = Vec::new();
            let is_indirect_recursive = if !is_direct_recursive {
                let mut visited = HashSet::new();
                find_module_cycle(&inst.module_name, &module.name, module_map, &mut visited, &mut cycle_path)
            } else {
                false
            };

            if is_direct_recursive || is_indirect_recursive {
                let (related, msg) = if is_direct_recursive {
                    (
                        vec![],
                        format!("Module '{}' recursively instantiates itself", module.name),
                    )
                } else {
                    let full_path = format!("{} -> {}", module.name, cycle_path.join(" -> "));
                    (
                        vec![Related {
                            role: "cycle".into(),
                            name: Some(full_path.clone()),
                            location: Some(inst.span.to_location()),
                        }],
                        format!(
                            "Module '{}' recursively instantiates itself via cycle: {}",
                            module.name, full_path
                        ),
                    )
                };
                diags.push(Diagnostic {
                    stage: "lint".into(),
                    code: "recursive-instance".into(),
                    severity: "error".into(),
                    location: Some(inst.span.to_location()),
                    subject: Subject {
                        kind: "component".into(),
                        name: Some(inst.instance_name.clone()),
                        path: Some(format!("{}.{}", module.name, inst.instance_name)),
                        id: None,
                        ref_des: None,
                        pad: None,
                    },
                    related,
                    expected: None,
                    actual: None,
                    fix: None,
                    message: msg,
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
                        location: Some(inst.span.to_location()),
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
                } else if let Some(fp_val) = inst_fp.or(leaf_fp).and_then(|a| a.value.as_deref()) {
                    match fp_resolver.resolve(fp_val) {
                        Err(kinema_kicad::FootprintResolveError::FootprintNotFound(_))
                        | Err(kinema_kicad::FootprintResolveError::LibraryNotFound(_)) => {
                            diags.push(Diagnostic {
                                stage: "lint".into(),
                                code: "footprint-missing".into(),
                                severity: "error".into(),
                                location: Some(inst.span.to_location()),
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
                                actual: Some(serde_json::json!(fp_val)),
                                fix: Some(format!("Ensure footprint '{}' exists in library or fp-lib-table", fp_val)),
                                message: format!("Footprint '{}' does not exist in footprint libraries", fp_val),
                            });
                        }
                        _ => {}
                    }
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
                        location: Some(p_ovr.span.to_location()),
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
                            location: Some(inst.span.to_location()),
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
                        location: Some(inst.span.to_location()),
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
                        location: Some(conn.span.to_location()),
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
                        location: Some(conn.span.to_location()),
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
                                location: Some(conn.span.to_location()),
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
                                location: Some(conn.span.to_location()),
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
                        *direct_pad_counts.entry(w.clone()).or_default() += 1;
                        let canonical_w = resolve_canonical(w.clone());

                        if port_etype == "power_out" {
                            net_power_outs.entry(canonical_w.clone()).or_default().push(conn.span.to_location());
                            *net_drivers.entry(canonical_w.clone()).or_default() += 1;
                        } else if port_etype == "output"
                            || port_etype == "bidirectional"
                            || port_etype == "power_in"
                            || port_etype == "open_collector"
                            || port_etype == "open_emitter"
                            || port_etype == "tri_state"
                        {
                            *net_drivers.entry(canonical_w.clone()).or_default() += 1;
                        }

                        net_pad_counts.entry(canonical_w).or_default().push(format!("{}.{}", inst.instance_name, port.name));

                        // 16. pin-wire-name check if wire is joined to a hub
                        let is_hub_child = joins_by_child.contains_key(&w) && !hub_wires.contains_key(&w);
                        if is_hub_child {
                            pin_wires_used.insert(w.clone());
                            let expected_wire_name = format!("{}_{}", inst.instance_name, port.name);
                            if w != expected_wire_name {
                                let wire_decl = declared_wires.get(&w);
                                diags.push(Diagnostic {
                                    stage: "lint".into(),
                                    code: "pin-wire-name".into(),
                                    severity: "error".into(),
                                    location: Some(conn.span.to_location()),
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
                                location: Some(conn.span.to_location()),
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
                    let port_width = port.range.as_ref().map(|r| r.msb - r.lsb + 1).unwrap_or(1);
                    let mut out_of_bounds = None;

                    let get_ref_width = |r: &RefExpr, oob: &mut Option<String>| -> u32 {
                        let declared_range = if let Some(w) = declared_wires.get(&r.ident) {
                            w.range.as_ref().map(|rng| (rng.msb, rng.lsb))
                        } else if let Some(p) = declared_ports.get(&r.ident) {
                            p.range.as_ref().map(|rng| (rng.msb, rng.lsb))
                        } else {
                            None
                        };

                        if let Some(idx) = &r.index {
                            match idx {
                                RefIndex::Single(i) => {
                                    if let Some((msb, lsb)) = declared_range {
                                        if *i < lsb || *i > msb {
                                            *oob = Some(format!("Bit index [{}] out of bounds for net '{}' (declared [{}:{}])", i, r.ident, msb, lsb));
                                        }
                                    }
                                    1
                                }
                                RefIndex::Range(msb, lsb) => {
                                    if let Some((d_msb, d_lsb)) = declared_range {
                                        if *msb > d_msb || *lsb < d_lsb {
                                            *oob = Some(format!("Bit slice [{}:{}] out of bounds for net '{}' (declared [{}:{}])", msb, lsb, r.ident, d_msb, d_lsb));
                                        }
                                    }
                                    if msb >= lsb { msb - lsb + 1 } else { 0 }
                                }
                            }
                        } else if let Some((msb, lsb)) = declared_range {
                            msb - lsb + 1
                        } else {
                            1
                        }
                    };

                    let conn_width = match expr {
                        Expr::Ref(r) => get_ref_width(r, &mut out_of_bounds),
                        Expr::Concat(list) => list.iter().map(|r| get_ref_width(r, &mut out_of_bounds)).sum(),
                    };

                    if let Some(oob_msg) = out_of_bounds {
                        diags.push(Diagnostic {
                            stage: "lint".into(),
                            code: "width-mismatch".into(),
                            severity: "error".into(),
                            location: Some(conn.span.to_location()),
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
                            message: oob_msg,
                        });
                    } else if port_width != conn_width {
                        diags.push(Diagnostic {
                            stage: "lint".into(),
                            code: "width-mismatch".into(),
                            severity: "error".into(),
                            location: Some(conn.span.to_location()),
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

    // 15. pin-wire-shape check
    for wire_name in &pin_wires_used {
        let pad_count = direct_pad_counts.get(wire_name).copied().unwrap_or(0);
        let join_count = joins_by_child.get(wire_name).map(|v| v.len()).unwrap_or(0);
        if pad_count != 1 || join_count != 1 {
            let wire_decl = declared_wires.get(wire_name);
            diags.push(Diagnostic {
                stage: "lint".into(),
                code: "pin-wire-shape".into(),
                severity: "error".into(),
                location: Some(wire_decl.map(|w| w.span.to_location()).unwrap_or(module.span.to_location())),
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
    let mut net_constraints: HashMap<String, HashMap<String, Vec<(String, SourceLocation)>>> = HashMap::new();

    let mut collect_constraints = |name: &str, attrs: &[Attr]| {
        let canonical = resolve_canonical(name.to_string());
        for attr in attrs {
            if attr.key == "width" || attr.key == "current" || attr.key == "netclass" {
                if let Some(val) = &attr.value {
                    net_constraints
                        .entry(canonical.clone())
                        .or_default()
                        .entry(attr.key.clone())
                        .or_default()
                        .push((val.clone(), attr.span.to_location()));
                }
            }
        }
    };

    for port in &module.ports {
        collect_constraints(&port.name, &port.attrs);
    }
    for item in &module.items {
        if let Item::Wire(w) = item {
            collect_constraints(&w.name, &w.attrs);
        }
    }

    for (canonical_net, kind_map) in &net_constraints {
        for (kind, vals) in kind_map {
            if vals.len() > 1 && vals.windows(2).any(|w| w[0].0 != w[1].0) {
                diags.push(Diagnostic {
                    stage: "lint".into(),
                    code: "constraint-conflict".into(),
                    severity: "error".into(),
                    location: Some(vals[1].1.clone()),
                    subject: Subject {
                        kind: "net".into(),
                        name: Some(canonical_net.clone()),
                        path: Some(format!("{}.{}", module.name, canonical_net)),
                        id: None,
                        ref_des: None,
                        pad: None,
                    },
                    related: vec![Related {
                        role: format!("conflicting-{}", kind),
                        name: Some(vals[0].0.clone()),
                        location: Some(vals[0].1.clone()),
                    }],
                    expected: None,
                    actual: None,
                    fix: Some(format!("Unify {} constraints on net '{}'", kind, canonical_net)),
                    message: format!("Conflicting {} constraints specified for net '{}'", kind, canonical_net),
                });
            }
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
                    location: Some(wire.span.to_location()),
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
                location: Some(hub_wire.span.to_location()),
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
                location: Some(locations[1].clone()),
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
                location: Some(loc),
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
                location: Some(loc),
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

pub const VALID_ETYPES: &[&str] = &[
    "input",
    "output",
    "bidirectional",
    "tri_state",
    "passive",
    "power_in",
    "power_out",
    "open_collector",
    "open_emitter",
    "no_connect",
];

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
        "property" => (target_kind == "module" && is_leaf) || target_kind == "instance",
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
            location: Some(attr.span.to_location()),
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
    } else if attr.key == "etype" {
        if let Some(val) = &attr.value {
            if !VALID_ETYPES.contains(&val.as_str()) {
                diags.push(Diagnostic {
                    stage: "lint".into(),
                    code: "unknown-attr".into(),
                    severity: "error".into(),
                    location: Some(attr.span.to_location()),
                    subject: Subject {
                        kind: target_kind.into(),
                        name: Some(attr.key.clone()),
                        path: None,
                        id: None,
                        ref_des: None,
                        pad: None,
                    },
                    related: vec![],
                    expected: Some(serde_json::json!(VALID_ETYPES)),
                    actual: Some(serde_json::json!(val)),
                    fix: Some("Specify valid etype (e.g. \"input\", \"output\", \"passive\", \"power_in\", etc.)".into()),
                    message: format!("Invalid etype '{}'. Must be one of {:?}", val, VALID_ETYPES),
                });
            }
        }
    } else if attr.key == "property" {
        let is_valid_format = if let Some(val) = &attr.value {
            let sep_pos = val.find('=').or_else(|| val.find(':'));
            if let Some(pos) = sep_pos {
                !val[..pos].trim().is_empty()
            } else {
                false
            }
        } else {
            false
        };
        if !is_valid_format {
            diags.push(Diagnostic {
                stage: "lint".into(),
                code: "unknown-attr".into(),
                severity: "error".into(),
                location: Some(attr.span.to_location()),
                subject: Subject {
                    kind: target_kind.into(),
                    name: Some(attr.key.clone()),
                    path: None,
                    id: None,
                    ref_des: None,
                    pad: None,
                },
                related: vec![],
                expected: Some(serde_json::json!("Name=Value or Name:Value")),
                actual: attr.value.as_ref().map(|v| serde_json::json!(v)),
                fix: Some("Specify property attribute in \"Name=Value\" format (e.g. \"LCSC PN=C546649\")".into()),
                message: "Invalid property attribute format. Must be \"Name=Value\" or \"Name:Value\" (e.g. \"LCSC PN=C546649\")".into(),
            });
        }
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
                    location: Some(attr.span.to_location()),
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
                    location: Some(attr.span.to_location()),
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
    } else if footprint.contains("SOT-23-5") {
        Some(5)
    } else if footprint.contains("SOT-23-6") {
        Some(6)
    } else if footprint.contains("SOT-23") {
        Some(3)
    } else if footprint.contains("QFP-32") {
        Some(32)
    } else {
        None
    }
}
