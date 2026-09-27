use kinema_elab::ir::*;
use kinema_kicad::pcb_parser::*;
use kinema_lint::diagnostic::*;
use kinema_syntax::ast::SourceLocation;
use std::collections::{HashMap, HashSet};

pub fn check_equivalence(ir: &FlatNetlistIR, board: &PcbBoard) -> LintReport {
    let mut diags = Vec::new();

    // Map PCB footprints by UUID / tstamp and by ref
    let mut pcb_by_tstamp: HashMap<String, &PcbFootprint> = HashMap::new();
    let mut pcb_by_ref: HashMap<String, &PcbFootprint> = HashMap::new();
    let mut matched_pcb_fps: HashSet<String> = HashSet::new();

    for fp in &board.footprints {
        if fp.board_only {
            continue;
        }
        pcb_by_tstamp.insert(fp.tstamp.clone(), fp);
        if let Some(path) = &fp.path {
            pcb_by_tstamp.insert(path.trim_start_matches('/').to_string(), fp);
        }
        pcb_by_ref.insert(fp.refdes.clone(), fp);
    }

    // Map IR components to PCB footprints
    let mut comp_to_pcb: HashMap<String, &PcbFootprint> = HashMap::new(); // comp.identity_key -> PcbFootprint

    for comp in &ir.components {
        let pcb_fp = if let Some(fp) = pcb_by_tstamp.get(&comp.uuid).or_else(|| pcb_by_tstamp.get(&comp.identity_key)) {
            matched_pcb_fps.insert(fp.tstamp.clone());
            Some(*fp)
        } else if let Some(fp) = pcb_by_ref.get(&comp.refdes) {
            matched_pcb_fps.insert(fp.tstamp.clone());
            diags.push(Diagnostic {
                stage: "equiv".into(),
                code: "identity-missing".into(),
                severity: "warning".into(),
                location: SourceLocation { file: "".into(), line: 1, col: 1 },
                subject: Subject {
                    kind: "component".into(),
                    name: Some(comp.refdes.clone()),
                    path: Some(comp.path.clone()),
                    id: Some(comp.identity_key.clone()),
                    ref_des: Some(comp.refdes.clone()),
                    pad: None,
                },
                related: vec![],
                expected: Some(serde_json::json!(comp.uuid.clone())),
                actual: Some(serde_json::json!(fp.tstamp.clone())),
                fix: None,
                message: format!("Component '{}' path does not match identity key; matched by reference designator", comp.refdes),
            });
            Some(*fp)
        } else {
            diags.push(Diagnostic {
                stage: "equiv".into(),
                code: "component-missing".into(),
                severity: "error".into(),
                location: SourceLocation { file: "".into(), line: 1, col: 1 },
                subject: Subject {
                    kind: "component".into(),
                    name: Some(comp.refdes.clone()),
                    path: Some(comp.path.clone()),
                    id: Some(comp.identity_key.clone()),
                    ref_des: Some(comp.refdes.clone()),
                    pad: None,
                },
                related: vec![],
                expected: None,
                actual: None,
                fix: Some("Re-import netlist into KiCad and place components".into()),
                message: format!("Component '{}' ({}) in circuit description does not exist on PCB", comp.refdes, comp.path),
            });
            None
        };

        if let Some(fp) = pcb_fp {
            comp_to_pcb.insert(comp.identity_key.clone(), fp);

            // Check footprint
            if !comp.footprint.is_empty() && comp.footprint != fp.footprint {
                diags.push(Diagnostic {
                    stage: "equiv".into(),
                    code: "footprint-mismatch".into(),
                    severity: "error".into(),
                    location: SourceLocation { file: "".into(), line: 1, col: 1 },
                    subject: Subject {
                        kind: "component".into(),
                        name: Some(comp.refdes.clone()),
                        path: Some(comp.path.clone()),
                        id: Some(comp.identity_key.clone()),
                        ref_des: Some(comp.refdes.clone()),
                        pad: None,
                    },
                    related: vec![],
                    expected: Some(serde_json::json!(comp.footprint.clone())),
                    actual: Some(serde_json::json!(fp.footprint.clone())),
                    fix: None,
                    message: format!("Footprint mismatch for component '{}' (expected: {}, actual: {})", comp.refdes, comp.footprint, fp.footprint),
                });
            }

            // Check ref
            if comp.refdes != fp.refdes {
                diags.push(Diagnostic {
                    stage: "equiv".into(),
                    code: "ref-mismatch".into(),
                    severity: "error".into(),
                    location: SourceLocation { file: "".into(), line: 1, col: 1 },
                    subject: Subject {
                        kind: "component".into(),
                        name: Some(comp.refdes.clone()),
                        path: Some(comp.path.clone()),
                        id: Some(comp.identity_key.clone()),
                        ref_des: Some(comp.refdes.clone()),
                        pad: None,
                    },
                    related: vec![],
                    expected: Some(serde_json::json!(comp.refdes.clone())),
                    actual: Some(serde_json::json!(fp.refdes.clone())),
                    fix: None,
                    message: format!("Fixed reference designator '{}' does not match board reference '{}'", comp.refdes, fp.refdes),
                });
            }

            // Check fields (value, mpn, dnp)
            if let Some(exp_val) = &comp.value {
                if exp_val != &fp.value {
                    diags.push(Diagnostic {
                        stage: "equiv".into(),
                        code: "field-mismatch".into(),
                        severity: "error".into(),
                        location: SourceLocation { file: "".into(), line: 1, col: 1 },
                        subject: Subject {
                            kind: "component".into(),
                            name: Some(comp.refdes.clone()),
                            path: Some(comp.path.clone()),
                            id: Some(comp.identity_key.clone()),
                            ref_des: Some(comp.refdes.clone()),
                            pad: None,
                        },
                        related: vec![],
                        expected: Some(serde_json::json!(exp_val)),
                        actual: Some(serde_json::json!(fp.value.clone())),
                        fix: None,
                        message: format!("Value mismatch for component '{}' (expected: {}, actual: {})", comp.refdes, exp_val, fp.value),
                    });
                }
            }

            if comp.dnp != fp.dnp {
                diags.push(Diagnostic {
                    stage: "equiv".into(),
                    code: "field-mismatch".into(),
                    severity: "error".into(),
                    location: SourceLocation { file: "".into(), line: 1, col: 1 },
                    subject: Subject {
                        kind: "component".into(),
                        name: Some(comp.refdes.clone()),
                        path: Some(comp.path.clone()),
                        id: Some(comp.identity_key.clone()),
                        ref_des: Some(comp.refdes.clone()),
                        pad: None,
                    },
                    related: vec![],
                    expected: Some(serde_json::json!(comp.dnp)),
                    actual: Some(serde_json::json!(fp.dnp)),
                    fix: None,
                    message: format!("DNP attribute mismatch for component '{}'", comp.refdes),
                });
            }
        }
    }

    // Check extra components on board
    for fp in &board.footprints {
        if fp.board_only {
            continue;
        }
        if !matched_pcb_fps.contains(&fp.tstamp) {
            diags.push(Diagnostic {
                stage: "equiv".into(),
                code: "component-extra".into(),
                severity: "error".into(),
                location: SourceLocation { file: "".into(), line: 1, col: 1 },
                subject: Subject {
                    kind: "component".into(),
                    name: Some(fp.refdes.clone()),
                    path: None,
                    id: Some(fp.tstamp.clone()),
                    ref_des: Some(fp.refdes.clone()),
                    pad: None,
                },
                related: vec![],
                expected: None,
                actual: None,
                fix: Some("Remove extraneous component from PCB or add to circuit description".into()),
                message: format!("Component '{}' on PCB does not exist in circuit description", fp.refdes),
            });
        }
    }

    // Net partition verification
    // 1. Build PCB pad lookup: (component_identity_key, pad_number) -> net_name
    let mut pcb_pad_to_net: HashMap<(String, String), String> = HashMap::new();
    let mut pcb_net_to_pads: HashMap<String, HashSet<(String, String)>> = HashMap::new();

    for (comp_key, fp) in &comp_to_pcb {
        for pad in &fp.pads {
            if pad.net_code != 0 && !pad.net_name.is_empty() {
                let key = (comp_key.clone(), pad.pad_number.clone());
                pcb_pad_to_net.insert(key.clone(), pad.net_name.clone());
                pcb_net_to_pads.entry(pad.net_name.clone()).or_default().insert(key);
            }
        }
    }

    // 2. Map IR nets to pad sets
    for ir_net in &ir.nets {
        let mut ir_pad_set: HashSet<(String, String)> = HashSet::new();
        for pad in &ir_net.pads {
            // Find component by path or ref
            let comp_opt = ir.components.iter().find(|c| c.path == pad.component_path || c.refdes == pad.component_ref);
            if let Some(comp) = comp_opt {
                ir_pad_set.insert((comp.identity_key.clone(), pad.pad_number.clone()));
            }
        }

        if ir_pad_set.is_empty() {
            continue;
        }

        // Find which PCB nets these pads are connected to
        let mut pcb_nets_involved: HashSet<String> = HashSet::new();
        for pad_key in &ir_pad_set {
            if let Some(net) = pcb_pad_to_net.get(pad_key) {
                pcb_nets_involved.insert(net.clone());
            } else {
                // Pad not connected on PCB
                pcb_nets_involved.insert("<unconnected>".into());
            }
        }

        if pcb_nets_involved.len() == 1 {
            let pcb_net_name = pcb_nets_involved.iter().next().unwrap();
            if pcb_net_name == "<unconnected>" {
                // Pads are all unconnected on PCB
                let first_pad = ir_pad_set.iter().next().unwrap();
                let comp = ir.components.iter().find(|c| c.identity_key == first_pad.0).unwrap();
                diags.push(Diagnostic {
                    stage: "equiv".into(),
                    code: "net-partition-mismatch".into(),
                    severity: "error".into(),
                    location: SourceLocation { file: "".into(), line: 1, col: 1 },
                    subject: Subject {
                        kind: "pad".into(),
                        name: Some(comp.refdes.clone()),
                        path: Some(comp.path.clone()),
                        id: Some(comp.identity_key.clone()),
                        ref_des: Some(comp.refdes.clone()),
                        pad: Some(first_pad.1.clone()),
                    },
                    related: vec![],
                    expected: Some(serde_json::json!({ "net": ir_net.name })),
                    actual: Some(serde_json::json!({ "net": "<unconnected>" })),
                    fix: None,
                    message: format!("Pad in net '{}' is unconnected on PCB", ir_net.name),
                });
            } else {
                let pcb_pads = pcb_net_to_pads.get(pcb_net_name).unwrap();
                if &ir_pad_set == pcb_pads {
                    // Perfect partition match!
                    // Check net name
                    if &ir_net.name != pcb_net_name {
                        let first_pad = ir_pad_set.iter().next().unwrap();
                        let comp = ir.components.iter().find(|c| c.identity_key == first_pad.0).unwrap();
                        diags.push(Diagnostic {
                            stage: "equiv".into(),
                            code: "net-name-mismatch".into(),
                            severity: "error".into(),
                            location: SourceLocation { file: "".into(), line: 1, col: 1 },
                            subject: Subject {
                                kind: "net".into(),
                                name: Some(comp.refdes.clone()),
                                path: Some(comp.path.clone()),
                                id: Some(comp.identity_key.clone()),
                                ref_des: Some(comp.refdes.clone()),
                                pad: Some(first_pad.1.clone()),
                            },
                            related: vec![],
                            expected: Some(serde_json::json!({ "net": ir_net.name })),
                            actual: Some(serde_json::json!({ "net": pcb_net_name })),
                            fix: None,
                            message: format!("Net partition matches, but net name differs (expected: {}, actual: {})", ir_net.name, pcb_net_name),
                        });
                    }
                } else {
                    // PCB net has different set of pads (partition mismatch)
                    let first_pad = ir_pad_set.iter().next().unwrap();
                    let comp = ir.components.iter().find(|c| c.identity_key == first_pad.0).unwrap();
                    diags.push(Diagnostic {
                        stage: "equiv".into(),
                        code: "net-partition-mismatch".into(),
                        severity: "error".into(),
                        location: SourceLocation { file: "".into(), line: 1, col: 1 },
                        subject: Subject {
                            kind: "pad".into(),
                            name: Some(comp.refdes.clone()),
                            path: Some(comp.path.clone()),
                            id: Some(comp.identity_key.clone()),
                            ref_des: Some(comp.refdes.clone()),
                            pad: Some(first_pad.1.clone()),
                        },
                        related: vec![],
                        expected: Some(serde_json::json!({ "net": ir_net.name })),
                        actual: Some(serde_json::json!({ "net": pcb_net_name })),
                        fix: None,
                        message: format!("Pad partition for net '{}' does not match PCB net '{}'", ir_net.name, pcb_net_name),
                    });
                }
            }
        } else {
            // Pads belong to multiple PCB nets (partition mismatch)
            let first_pad = ir_pad_set.iter().next().unwrap();
            let comp = ir.components.iter().find(|c| c.identity_key == first_pad.0).unwrap();
            let actual_nets: Vec<String> = pcb_nets_involved.into_iter().collect();
            diags.push(Diagnostic {
                stage: "equiv".into(),
                code: "net-partition-mismatch".into(),
                severity: "error".into(),
                location: SourceLocation { file: "".into(), line: 1, col: 1 },
                subject: Subject {
                    kind: "pad".into(),
                    name: Some(comp.refdes.clone()),
                    path: Some(comp.path.clone()),
                    id: Some(comp.identity_key.clone()),
                    ref_des: Some(comp.refdes.clone()),
                    pad: Some(first_pad.1.clone()),
                },
                related: vec![],
                expected: Some(serde_json::json!({ "net": ir_net.name })),
                actual: Some(serde_json::json!({ "nets": actual_nets })),
                fix: None,
                message: format!("Pads of {} expected on net {}, but split across multiple nets on PCB", comp.refdes, ir_net.name),
            });
        }
    }

    LintReport::from_diagnostics(diags)
}
