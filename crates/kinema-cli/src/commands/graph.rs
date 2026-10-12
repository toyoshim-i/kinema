use super::common::{format_report, load_source_files, resolve_project_sources};
use super::CommandResult;
use kinema_elab::{elaborate_sources, FlatNetlistIR};
use std::collections::HashMap;

pub fn run_graph(files: &[String], _json: bool) -> CommandResult {
    let (paths, _) = resolve_project_sources(files);
    let source_files = match load_source_files(&paths) {
        Ok(f) => f,
        Err(report) => {
            let output = format_report(&report, agent_man::RenderMode::Json);
            let data = serde_json::to_value(&report).ok();
            let mut res = CommandResult::error(output);
            res.data = data;
            return res;
        }
    };

    match elaborate_sources(&source_files) {
        Ok(ir) => {
            let yosys_json = generate_yosys_json(&ir);
            let json_str = format!("{}\n", serde_json::to_string_pretty(&yosys_json).unwrap_or_default());
            let mut res = CommandResult::success(json_str);
            res.data = Some(yosys_json);
            res
        }
        Err(e) => CommandResult::error(format!("Elaboration error: {}\n", e)),
    }
}

fn generate_yosys_json(ir: &FlatNetlistIR) -> serde_json::Value {
    let mut modules = serde_json::Map::new();
    let mut cells = serde_json::Map::new();
    let mut netnames = serde_json::Map::new();

    let mut pad_to_bit: HashMap<(String, String), usize> = HashMap::new();

    for (idx, net) in ir.nets.iter().enumerate() {
        let bit_id = idx + 2;
        netnames.insert(
            net.name.clone(),
            serde_json::json!({
                "hide_name": 0,
                "bits": [bit_id],
                "attributes": {}
            }),
        );
        for pad in &net.pads {
            pad_to_bit.insert((pad.component_ref.clone(), pad.port_name.clone()), bit_id);
        }
    }

    for comp in &ir.components {
        let mut conn = serde_json::Map::new();
        let mut port_dirs = serde_json::Map::new();

        for pad in &comp.pads {
            if let Some(&bit_id) = pad_to_bit.get(&(comp.refdes.clone(), pad.port_name.clone())) {
                conn.insert(pad.port_name.clone(), serde_json::json!([bit_id]));
            } else {
                conn.insert(pad.port_name.clone(), serde_json::json!([]));
            }
            port_dirs.insert(pad.port_name.clone(), serde_json::json!(&pad.etype));
        }

        cells.insert(
            comp.refdes.clone(),
            serde_json::json!({
                "type": comp.module_name,
                "port_directions": port_dirs,
                "connections": conn
            }),
        );
    }

    modules.insert(
        ir.top_module.clone(),
        serde_json::json!({
            "cells": cells,
            "netnames": netnames
        }),
    );

    serde_json::json!({
        "creator": "kinema",
        "modules": modules
    })
}
