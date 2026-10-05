use kinema_elab::ir::FlatNetlistIR;
use std::time::{SystemTime, UNIX_EPOCH};

fn escape_sexpr(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

fn current_date_string() -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let total_secs = duration.as_secs();
    let days = total_secs / 86400;

    let z = days as i64 + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1020 + doe / 1461 - doe / 146096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    format!("{:04}-{:02}-{:02}", y, m, d)
}

pub fn generate_kicad_netlist(ir: &FlatNetlistIR) -> String {
    let mut out = String::new();

    out.push_str("(export (version \"E\")\n");
    out.push_str("  (design\n");
    out.push_str("    (source \"kinema\")\n");
    out.push_str(&format!("    (date \"{}\")\n", current_date_string()));
    out.push_str("    (tool \"kinema (KiCad Netlist Generator)\")\n");
    out.push_str("  )\n");

    // Components section
    out.push_str("  (components\n");
    for comp in &ir.components {
        let refdes = escape_sexpr(&comp.refdes);
        let footprint = escape_sexpr(&comp.footprint);
        let uuid = escape_sexpr(&comp.uuid);
        let raw_val = comp.value.as_deref().unwrap_or(comp.mpn.as_deref().unwrap_or(&comp.module_name));
        let val = escape_sexpr(raw_val);

        out.push_str(&format!("    (comp (ref \"{}\")\n", refdes));
        out.push_str(&format!("      (value \"{}\")\n", val));
        out.push_str(&format!("      (footprint \"{}\")\n", footprint));
        out.push_str(&format!("      (tstamp \"{}\")\n", uuid));
        out.push_str(&format!("      (property (name \"Reference\") (value \"{}\"))\n", refdes));
        out.push_str(&format!("      (property (name \"Value\") (value \"{}\"))\n", val));
        out.push_str(&format!("      (property (name \"Footprint\") (value \"{}\"))\n", footprint));

        if let Some(mpn) = &comp.mpn {
            out.push_str(&format!("      (property (name \"mpn\") (value \"{}\"))\n", escape_sexpr(mpn)));
        }
        if comp.dnp {
            out.push_str("      (property (name \"dnp\") (value \"\"))\n");
        }
        for (prop_name, prop_val) in &comp.properties {
            if prop_name != "Reference" && prop_name != "Value" && prop_name != "Footprint" && prop_name != "mpn" && prop_name != "dnp" {
                out.push_str(&format!(
                    "      (property (name \"{}\") (value \"{}\"))\n",
                    escape_sexpr(prop_name),
                    escape_sexpr(prop_val)
                ));
            }
        }

        out.push_str("      (sheetpath (names \"/\") (tstamps \"/\"))\n");
        out.push_str(&format!("      (tstamps \"{}\")\n", uuid));
        out.push_str("    )\n");
    }
    out.push_str("  )\n");

    // Nets section
    out.push_str("  (nets\n");
    for (i, net) in ir.nets.iter().enumerate() {
        let code = i + 1;
        out.push_str(&format!("    (net (code \"{}\") (name \"{}\")\n", code, escape_sexpr(&net.name)));
        for pad in &net.pads {
            out.push_str(&format!(
                "      (node (ref \"{}\") (pin \"{}\") (pintype \"{}\"))\n",
                escape_sexpr(&pad.component_ref),
                escape_sexpr(&pad.pad_number),
                escape_sexpr(&pad.etype)
            ));
        }
        out.push_str("    )\n");
    }
    out.push_str("  )\n");

    out.push_str(")\n");
    out
}

