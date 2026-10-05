use kinema_elab::ir::*;
use kinema_kicad::pcb_parser::*;
use kinema_lint::diagnostic::*;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

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
    let mut comp_ref_by_id: HashMap<String, String> = HashMap::new();

    for comp in &ir.components {
        comp_ref_by_id.insert(comp.identity_key.clone(), comp.refdes.clone());
        let pcb_fp = if let Some(fp) = pcb_by_tstamp.get(&comp.uuid).or_else(|| pcb_by_tstamp.get(&comp.identity_key)) {
            matched_pcb_fps.insert(fp.tstamp.clone());
            Some(*fp)
        } else if let Some(fp) = pcb_by_ref.get(&comp.refdes) {
            matched_pcb_fps.insert(fp.tstamp.clone());
            diags.push(Diagnostic {
                stage: "equiv".into(),
                code: "identity-missing".into(),
                severity: "warning".into(),
                location: None,
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
                location: None,
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
                    location: None,
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
                    location: None,
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
                        location: None,
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

            // MPN check
            if let Some(exp_mpn) = &comp.mpn {
                let actual_mpn = fp.mpn.as_deref().unwrap_or("");
                if exp_mpn != actual_mpn {
                    diags.push(Diagnostic {
                        stage: "equiv".into(),
                        code: "field-mismatch".into(),
                        severity: "error".into(),
                        location: None,
                        subject: Subject {
                            kind: "component".into(),
                            name: Some(comp.refdes.clone()),
                            path: Some(comp.path.clone()),
                            id: Some(comp.identity_key.clone()),
                            ref_des: Some(comp.refdes.clone()),
                            pad: None,
                        },
                        related: vec![],
                        expected: Some(serde_json::json!(exp_mpn)),
                        actual: Some(serde_json::json!(actual_mpn)),
                        fix: None,
                        message: format!("MPN mismatch for component '{}' (expected: {}, actual: {})", comp.refdes, exp_mpn, actual_mpn),
                    });
                }
            }

            if comp.dnp != fp.dnp {
                diags.push(Diagnostic {
                    stage: "equiv".into(),
                    code: "field-mismatch".into(),
                    severity: "error".into(),
                    location: None,
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

            // Check custom properties (e.g. "LCSC PN")
            for (prop_name, exp_prop_val) in &comp.properties {
                let actual_prop_val = fp.properties.get(prop_name).map(|s| s.as_str()).unwrap_or("");
                if exp_prop_val != actual_prop_val {
                    diags.push(Diagnostic {
                        stage: "equiv".into(),
                        code: "field-mismatch".into(),
                        severity: "error".into(),
                        location: None,
                        subject: Subject {
                            kind: "component".into(),
                            name: Some(comp.refdes.clone()),
                            path: Some(comp.path.clone()),
                            id: Some(comp.identity_key.clone()),
                            ref_des: Some(comp.refdes.clone()),
                            pad: None,
                        },
                        related: vec![],
                        expected: Some(serde_json::json!(exp_prop_val)),
                        actual: Some(serde_json::json!(actual_prop_val)),
                        fix: None,
                        message: format!("Property '{}' mismatch for component '{}' (expected: {}, actual: {})", prop_name, comp.refdes, exp_prop_val, actual_prop_val),
                    });
                }
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
                location: None,
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
            if !pad.net_name.is_empty() {
                let key = (comp_key.clone(), pad.pad_number.clone());
                pcb_pad_to_net.insert(key.clone(), pad.net_name.clone());
                pcb_net_to_pads.entry(pad.net_name.clone()).or_default().insert(key);
            }
        }
    }

    // Check nc-pad-connected
    for comp in &ir.components {
        for pad in &comp.pads {
            if pad.etype == "no_connect" {
                let key = (comp.identity_key.clone(), pad.pad_number.clone());
                if let Some(net) = pcb_pad_to_net.get(&key) {
                    if !net.is_empty() && net != "<unconnected>" {
                        diags.push(Diagnostic {
                            stage: "equiv".into(),
                            code: "nc-pad-connected".into(),
                            severity: "error".into(),
                            location: None,
                            subject: Subject {
                                kind: "pad".into(),
                                name: Some(format!("{}.{}", comp.refdes, pad.pad_number)),
                                path: Some(format!("{}.{}", comp.path, pad.pad_number)),
                                id: Some(comp.identity_key.clone()),
                                ref_des: Some(comp.refdes.clone()),
                                pad: Some(pad.pad_number.clone()),
                            },
                            related: vec![],
                            expected: Some(serde_json::json!("<unconnected>")),
                            actual: Some(serde_json::json!(net)),
                            fix: Some(format!("Disconnect pad {}.{} from net '{}' on PCB", comp.refdes, pad.pad_number, net)),
                            message: format!("No-connect pad {}.{} is connected to net '{}' on PCB", comp.refdes, pad.pad_number, net),
                        });
                    }
                }
            }
        }
    }

    // 2. Map IR nets to pad sets
    for ir_net in &ir.nets {
        let mut ir_pad_set: HashSet<(String, String)> = HashSet::new();
        let mut ir_display_pads: BTreeSet<String> = BTreeSet::new();

        for pad in &ir_net.pads {
            let comp_opt = ir.components.iter().find(|c| c.path == pad.component_path || c.refdes == pad.component_ref);
            if let Some(comp) = comp_opt {
                ir_pad_set.insert((comp.identity_key.clone(), pad.pad_number.clone()));
                ir_display_pads.insert(format!("{}.{}", comp.refdes, pad.pad_number));
            }
        }

        if ir_pad_set.is_empty() {
            continue;
        }

        // Map each pad to its actual PCB net
        let mut pad_actual_net: BTreeMap<String, String> = BTreeMap::new();
        let mut net_counts: HashMap<String, usize> = HashMap::new();

        for pad in &ir_net.pads {
            let comp_opt = ir.components.iter().find(|c| c.path == pad.component_path || c.refdes == pad.component_ref);
            if let Some(comp) = comp_opt {
                let disp = format!("{}.{}", comp.refdes, pad.pad_number);
                let actual = pcb_pad_to_net
                    .get(&(comp.identity_key.clone(), pad.pad_number.clone()))
                    .cloned()
                    .unwrap_or_else(|| "<unconnected>".into());
                *net_counts.entry(actual.clone()).or_default() += 1;
                pad_actual_net.insert(disp, actual);
            }
        }

        // Determine primary PCB net: prefer one matching ir_net.name, else the one with most pads.
        // Break ties deterministically by net name.
        let primary_pcb_net = if net_counts.contains_key(&ir_net.name) {
            ir_net.name.clone()
        } else {
            let mut candidates: Vec<(&String, &usize)> = net_counts
                .iter()
                .filter(|(n, _)| n.as_str() != "<unconnected>")
                .collect();
            candidates.sort_by(|(n1, c1), (n2, c2)| c2.cmp(c1).then_with(|| n1.cmp(n2)));
            candidates
                .first()
                .map(|(n, _)| (*n).clone())
                .unwrap_or_else(|| "<unconnected>".into())
        };

        // Get actual pads on this primary PCB net
        let mut pcb_display_pads: BTreeSet<String> = BTreeSet::new();
        if primary_pcb_net != "<unconnected>" {
            if let Some(pads) = pcb_net_to_pads.get(&primary_pcb_net) {
                for (cid, pnum) in pads {
                    if let Some(r) = comp_ref_by_id.get(cid) {
                        pcb_display_pads.insert(format!("{}.{}", r, pnum));
                    }
                }
            }
        }

        let missing_pads: Vec<String> = ir_display_pads.difference(&pcb_display_pads).cloned().collect();
        let extra_pads: Vec<String> = pcb_display_pads.difference(&ir_display_pads).cloned().collect();

        if missing_pads.is_empty() && extra_pads.is_empty() {
            // Partition matches perfectly! Check net name
            if primary_pcb_net != ir_net.name {
                diags.push(Diagnostic {
                    stage: "equiv".into(),
                    code: "net-name-mismatch".into(),
                    severity: "error".into(),
                    location: None,
                    subject: Subject {
                        kind: "net".into(),
                        name: Some(ir_net.name.clone()),
                        path: None,
                        id: None,
                        ref_des: None,
                        pad: None,
                    },
                    related: vec![],
                    expected: Some(serde_json::json!({ "net": ir_net.name })),
                    actual: Some(serde_json::json!({ "net": primary_pcb_net })),
                    fix: None,
                    message: format!("Net partition matches, but net name differs (expected: {}, actual: {})", ir_net.name, primary_pcb_net),
                });
            }
        } else {
            // Partition mismatch!
            let mut related = Vec::new();
            for mp in &missing_pads {
                let actual = pad_actual_net.get(mp).cloned().unwrap_or_else(|| "<unconnected>".into());
                related.push(Related {
                    role: "missing-pad".into(),
                    name: Some(format!("{} (connected to: {})", mp, actual)),
                    location: None,
                });
            }
            for ep in &extra_pads {
                related.push(Related {
                    role: "extra-pad".into(),
                    name: Some(ep.clone()),
                    location: None,
                });
            }

            let mut msg_parts = Vec::new();
            if !missing_pads.is_empty() {
                msg_parts.push(format!("missing pads [{}]", missing_pads.join(", ")));
            }
            if !extra_pads.is_empty() {
                msg_parts.push(format!("extra pads [{}]", extra_pads.join(", ")));
            }

            diags.push(Diagnostic {
                stage: "equiv".into(),
                code: "net-partition-mismatch".into(),
                severity: "error".into(),
                location: None,
                subject: Subject {
                    kind: "net".into(),
                    name: Some(ir_net.name.clone()),
                    path: None,
                    id: None,
                    ref_des: None,
                    pad: None,
                },
                related,
                expected: Some(serde_json::json!({ "net": ir_net.name, "pads": ir_display_pads })),
                actual: Some(serde_json::json!({
                    "primary_pcb_net": primary_pcb_net,
                    "missing_pads": missing_pads,
                    "extra_pads": extra_pads,
                    "pad_connections": pad_actual_net,
                })),
                fix: None,
                message: format!("Pad partition for net '{}' does not match PCB: {}", ir_net.name, msg_parts.join("; ")),
            });
        }
    }

    // Check net-extra: PCB nets that contain matched pads but do not correspond to any IR net
    let ir_net_names: HashSet<&str> = ir.nets.iter().map(|n| n.name.as_str()).collect();
    let mut checked_pcb_nets: HashSet<String> = HashSet::new();
    let mut sorted_pcb_net_names: Vec<String> = pcb_net_to_pads.keys().cloned().collect();
    sorted_pcb_net_names.sort();

    for pcb_net_name in sorted_pcb_net_names {
        if pcb_net_name == "<unconnected>" || pcb_net_name.is_empty() {
            continue;
        }
        if !ir_net_names.contains(pcb_net_name.as_str()) {
            let pads = &pcb_net_to_pads[&pcb_net_name];
            let mut pad_list: Vec<String> = pads.iter()
                .filter_map(|(cid, pnum)| comp_ref_by_id.get(cid).map(|r| format!("{}.{}", r, pnum)))
                .collect();
            pad_list.sort();
            if !pad_list.is_empty() && checked_pcb_nets.insert(pcb_net_name.clone()) {
                diags.push(Diagnostic {
                    stage: "equiv".into(),
                    code: "net-extra".into(),
                    severity: "error".into(),
                    location: None,
                    subject: Subject {
                        kind: "net".into(),
                        name: Some(pcb_net_name.clone()),
                        path: None,
                        id: None,
                        ref_des: None,
                        pad: None,
                    },
                    related: vec![],
                    expected: None,
                    actual: Some(serde_json::json!({ "net": pcb_net_name, "pads": pad_list })),
                    fix: Some(format!("Remove extra net '{}' from PCB or add to circuit description", pcb_net_name)),
                    message: format!("Net '{}' on PCB does not exist in circuit description", pcb_net_name),
                });
            }
        }
    }

    LintReport::from_diagnostics(diags)
}
