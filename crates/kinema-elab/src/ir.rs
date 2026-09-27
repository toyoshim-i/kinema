use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentPad {
    pub component_path: String,
    pub component_ref: String,
    pub pad_number: String,
    pub port_name: String,
    pub etype: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlatComponent {
    pub path: String,
    pub module_name: String,
    pub identity_key: String,
    pub uuid: String,
    pub refdes: String,
    pub prefix: String,
    pub footprint: String,
    pub mpn: Option<String>,
    pub value: Option<String>,
    pub dnp: bool,
    pub pads: Vec<ComponentPad>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlatNet {
    pub name: String,
    pub pads: Vec<ComponentPad>,
    pub width: Option<String>,
    pub current: Option<String>,
    pub netclass: Option<String>,
    pub diffpair: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NearbyGroup {
    pub hub_wire: String,
    pub hub_path: String,
    pub members: Vec<String>,
    pub pads: Vec<ComponentPad>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JoinNode {
    pub instance_name: String,
    pub parent_wire: String,
    pub child_wire: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlatNetlistIR {
    pub top_module: String,
    pub components: Vec<FlatComponent>,
    pub nets: Vec<FlatNet>,
    pub nearby_groups: Vec<NearbyGroup>,
    pub join_nodes: Vec<JoinNode>,
}

impl FlatNetlistIR {
    pub fn validate(&self) -> Result<(), crate::elaborator::ElabError> {
        use std::collections::{HashMap, HashSet};

        // 1. Uniqueness check (一意性)
        let mut seen_refdes = HashSet::new();
        let mut seen_uuids = HashSet::new();
        let mut seen_paths = HashSet::new();

        for comp in &self.components {
            if !seen_refdes.insert(&comp.refdes) {
                return Err(crate::elaborator::ElabError::DuplicateRef(comp.refdes.clone()));
            }
            if !seen_uuids.insert(&comp.uuid) {
                return Err(crate::elaborator::ElabError::DuplicateUuid(comp.uuid.clone(), comp.refdes.clone()));
            }
            if !seen_paths.insert(&comp.path) {
                return Err(crate::elaborator::ElabError::DuplicatePath(comp.path.clone()));
            }

            let mut seen_pads = HashSet::new();
            for pad in &comp.pads {
                if !seen_pads.insert(&pad.pad_number) {
                    return Err(crate::elaborator::ElabError::DuplicatePadNumber(comp.refdes.clone(), pad.pad_number.clone()));
                }
            }
        }

        let mut seen_nets = HashSet::new();
        for net in &self.nets {
            if !seen_nets.insert(&net.name) {
                return Err(crate::elaborator::ElabError::DuplicateNet(net.name.clone()));
            }
        }

        // 2. Connectivity check (接続)
        let all_comp_pads: HashSet<(&str, &str)> = self
            .components
            .iter()
            .flat_map(|c| c.pads.iter().map(move |p| (c.refdes.as_str(), p.pad_number.as_str())))
            .collect();

        let mut pad_to_net: HashMap<(&str, &str), &str> = HashMap::new();

        for net in &self.nets {
            let mut net_pads = HashSet::new();
            for pad in &net.pads {
                if !all_comp_pads.contains(&(pad.component_ref.as_str(), pad.pad_number.as_str())) {
                    return Err(crate::elaborator::ElabError::DanglingPad(
                        net.name.clone(),
                        pad.component_ref.clone(),
                        pad.pad_number.clone(),
                    ));
                }
                if !net_pads.insert((pad.component_ref.as_str(), pad.pad_number.as_str())) {
                    return Err(crate::elaborator::ElabError::DuplicatePadInNet(
                        net.name.clone(),
                        pad.component_ref.clone(),
                        pad.pad_number.clone(),
                    ));
                }
                if let Some(prev_net) = pad_to_net.insert((pad.component_ref.as_str(), pad.pad_number.as_str()), &net.name) {
                    return Err(crate::elaborator::ElabError::PadMultiNet(
                        pad.component_ref.clone(),
                        pad.pad_number.clone(),
                        prev_net.to_string(),
                        net.name.clone(),
                    ));
                }
            }
        }

        // 3. Constraints check (制約)
        for net in &self.nets {
            if let Some(w) = &net.width {
                if !(w.ends_with("mm") || w.ends_with("mil")) {
                    return Err(crate::elaborator::ElabError::InvalidConstraint(
                        net.name.clone(),
                        format!("Width constraint '{}' must specify unit 'mm' or 'mil'", w),
                    ));
                }
                let num_str = w.trim_end_matches("mm").trim_end_matches("mil");
                if num_str.parse::<f64>().map_or(true, |v| v <= 0.0) {
                    return Err(crate::elaborator::ElabError::InvalidConstraint(
                        net.name.clone(),
                        format!("Width constraint '{}' must be a positive number", w),
                    ));
                }
            }
            if let Some(c) = &net.current {
                if !(c.ends_with("mA") || c.ends_with('A')) {
                    return Err(crate::elaborator::ElabError::InvalidConstraint(
                        net.name.clone(),
                        format!("Current constraint '{}' must specify unit 'A' or 'mA'", c),
                    ));
                }
                let num_str = if c.ends_with("mA") {
                    c.trim_end_matches("mA")
                } else {
                    c.trim_end_matches('A')
                };
                if num_str.parse::<f64>().map_or(true, |v| v <= 0.0) {
                    return Err(crate::elaborator::ElabError::InvalidConstraint(
                        net.name.clone(),
                        format!("Current constraint '{}' must be a positive number", c),
                    ));
                }
            }
        }

        Ok(())
    }
}
