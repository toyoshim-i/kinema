use kinema_elab::ir::FlatNetlistIR;

pub fn generate_kicad_netlist(ir: &FlatNetlistIR) -> String {
    let mut out = String::new();

    out.push_str("(export (version \"E\")\n");
    out.push_str("  (design\n");
    out.push_str("    (source \"kinema\")\n");
    out.push_str("    (date \"2026-09-27\")\n");
    out.push_str("    (tool \"kinema (KiCad Netlist Generator)\")\n");
    out.push_str("  )\n");

    // Components section
    out.push_str("  (components\n");
    for comp in &ir.components {
        out.push_str(&format!("    (comp (ref \"{}\")\n", comp.refdes));
        let val = comp.value.as_deref().unwrap_or(comp.mpn.as_deref().unwrap_or(&comp.module_name));
        out.push_str(&format!("      (value \"{}\")\n", val));
        out.push_str(&format!("      (footprint \"{}\")\n", comp.footprint));
        out.push_str(&format!("      (tstamp \"{}\")\n", comp.uuid));
        out.push_str(&format!("      (property (name \"Reference\") (value \"{}\"))\n", comp.refdes));
        out.push_str(&format!("      (property (name \"Value\") (value \"{}\"))\n", val));
        out.push_str(&format!("      (property (name \"Footprint\") (value \"{}\"))\n", comp.footprint));

        if let Some(mpn) = &comp.mpn {
            out.push_str(&format!("      (property (name \"mpn\") (value \"{}\"))\n", mpn));
        }
        if comp.dnp {
            out.push_str("      (property (name \"dnp\") (value \"\"))\n");
        }

        out.push_str("      (sheetpath (names \"/\") (tstamps \"/\"))\n");
        out.push_str(&format!("      (tstamps \"{}\")\n", comp.uuid));
        out.push_str("    )\n");
    }
    out.push_str("  )\n");

    // Nets section
    out.push_str("  (nets\n");
    for (i, net) in ir.nets.iter().enumerate() {
        let code = i + 1;
        out.push_str(&format!("    (net (code \"{}\") (name \"{}\")\n", code, net.name));
        for pad in &net.pads {
            out.push_str(&format!(
                "      (node (ref \"{}\") (pin \"{}\") (pintype \"{}\"))\n",
                pad.component_ref, pad.pad_number, pad.etype
            ));
        }
        out.push_str("    )\n");
    }
    out.push_str("  )\n");

    out.push_str(")\n");
    out
}
