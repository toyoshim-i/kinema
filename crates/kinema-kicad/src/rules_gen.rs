use kinema_elab::ir::FlatNetlistIR;
use serde_json::json;

pub fn generate_kicad_dru(ir: &FlatNetlistIR) -> String {
    let mut out = String::new();
    out.push_str("(version 1)\n\n");

    for net in &ir.nets {
        if let Some(w) = &net.width {
            let rule_name = format!("width_{}", net.name);
            out.push_str(&format!("(rule \"{}\"\n", rule_name));
            out.push_str(&format!("  (constraint track_width (min {}))\n", w));
            out.push_str(&format!("  (condition \"Matchnet('{}')\")\n", net.name));
            out.push_str(")\n\n");
        }
    }

    out
}

pub fn merge_kicad_pro(ir: &FlatNetlistIR, existing_json: Option<&str>) -> Result<String, String> {
    let mut root: serde_json::Value = if let Some(content) = existing_json {
        serde_json::from_str(content).map_err(|e| e.to_string())?
    } else {
        json!({
            "meta": {
                "filename": format!("{}.kicad_pro", ir.top_module),
                "version": 1
            }
        })
    };

    if !root.is_object() {
        root = json!({});
    }

    let root_obj = root.as_object_mut().unwrap();
    let net_settings = root_obj.entry("net_settings").or_insert_with(|| json!({}));
    if !net_settings.is_object() {
        *net_settings = json!({});
    }

    // 1. Classes
    let mut classes_vec: Vec<serde_json::Value> = net_settings
        .get("classes")
        .and_then(|c| c.as_array())
        .cloned()
        .unwrap_or_else(|| {
            vec![json!({
                "name": "Default",
                "clearance": 0.2,
                "track_width": 0.25
            })]
        });

    let mut class_map: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for (i, c) in classes_vec.iter().enumerate() {
        if let Some(name) = c.get("name").and_then(|n| n.as_str()) {
            class_map.insert(name.to_string(), i);
        }
    }

    for net in &ir.nets {
        if let Some(nc) = &net.netclass {
            let track_w = net
                .width
                .as_ref()
                .and_then(|w| {
                    if w.ends_with("mm") {
                        w.trim_end_matches("mm").parse::<f64>().ok()
                    } else {
                        None
                    }
                })
                .unwrap_or(0.5);

            let new_class = json!({
                "name": nc,
                "clearance": 0.25,
                "track_width": track_w
            });

            if let Some(&idx) = class_map.get(nc) {
                classes_vec[idx] = new_class;
            } else {
                class_map.insert(nc.clone(), classes_vec.len());
                classes_vec.push(new_class);
            }
        }
    }
    net_settings["classes"] = serde_json::Value::Array(classes_vec);

    // 2. Netclass patterns
    let mut patterns_vec: Vec<serde_json::Value> = net_settings
        .get("netclass_patterns")
        .and_then(|p| p.as_array())
        .cloned()
        .unwrap_or_default();

    let mut pattern_map: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for (i, p) in patterns_vec.iter().enumerate() {
        if let Some(pat) = p.get("pattern").and_then(|pt| pt.as_str()) {
            pattern_map.insert(pat.to_string(), i);
        }
    }

    for net in &ir.nets {
        if let Some(nc) = &net.netclass {
            let entry = json!({
                "netclass": nc,
                "pattern": net.name
            });
            if let Some(&idx) = pattern_map.get(&net.name) {
                patterns_vec[idx] = entry;
            } else {
                pattern_map.insert(net.name.clone(), patterns_vec.len());
                patterns_vec.push(entry);
            }
        }
    }
    net_settings["netclass_patterns"] = serde_json::Value::Array(patterns_vec);

    serde_json::to_string_pretty(&root).map_err(|e| e.to_string())
}

pub fn generate_kicad_pro(ir: &FlatNetlistIR) -> String {
    merge_kicad_pro(ir, None).unwrap_or_default()
}
