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
