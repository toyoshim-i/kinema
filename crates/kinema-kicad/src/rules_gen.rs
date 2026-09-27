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

pub fn generate_kicad_pro(ir: &FlatNetlistIR) -> String {
    let mut classes = vec![json!({
        "name": "Default",
        "clearance": 0.2,
        "track_width": 0.25
    })];

    let mut seen_classes = std::collections::HashSet::new();
    for net in &ir.nets {
        if let Some(nc) = &net.netclass {
            if seen_classes.insert(nc.clone()) {
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

                classes.push(json!({
                    "name": nc,
                    "clearance": 0.25,
                    "track_width": track_w
                }));
            }
        }
    }

    let pro_json = json!({
        "net_settings": {
            "classes": classes
        }
    });

    serde_json::to_string_pretty(&pro_json).unwrap_or_default()
}
