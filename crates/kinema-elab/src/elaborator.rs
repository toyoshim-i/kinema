use crate::ir::*;
use kinema_syntax::ast::*;
use std::collections::{HashMap, HashSet};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ElabError {
    #[error("Top module count must be exactly 1, found {0}")]
    TopModule(usize),
    #[error("Module '{0}' not found")]
    ModuleNotFound(String),
    #[error("Join cycle detected involving wire '{0}'")]
    JoinCycle(String),
    #[error("Elaboration failed: {0}")]
    Generic(String),
}

pub type NetAttrTuple = (Option<String>, Option<String>, Option<String>, Option<String>);

pub struct Elaborator {
    modules: HashMap<String, ModuleDef>,
    used_ref_numbers: HashMap<String, HashSet<u32>>,
}

impl Elaborator {
    pub fn new(source_files: &[SourceFile]) -> Self {
        let mut modules = HashMap::new();
        for file in source_files {
            for m in &file.modules {
                modules.insert(m.name.clone(), m.clone());
            }
        }
        Self {
            modules,
            used_ref_numbers: HashMap::new(),
        }
    }

    pub fn find_top_module(&self) -> Result<String, ElabError> {
        let mut instantiated = HashSet::new();

        for m in self.modules.values() {
            for item in &m.items {
                if let Item::Instance(inst) = item {
                    if inst.module_name != "join" {
                        instantiated.insert(inst.module_name.clone());
                    }
                }
            }
        }

        let non_leaf_uninstantiated: Vec<String> = self
            .modules
            .values()
            .filter(|m| m.name != "join" && !m.items.is_empty())
            .map(|m| m.name.clone())
            .filter(|name| !instantiated.contains(name))
            .collect();

        if non_leaf_uninstantiated.len() == 1 {
            Ok(non_leaf_uninstantiated[0].clone())
        } else {
            Err(ElabError::TopModule(non_leaf_uninstantiated.len()))
        }
    }

    pub fn elaborate(&mut self) -> Result<FlatNetlistIR, ElabError> {
        let top_name = self.find_top_module()?;
        let top_module = self.modules.get(&top_name).cloned().ok_or_else(|| {
            ElabError::ModuleNotFound(top_name.clone())
        })?;

        let mut components = Vec::new();
        let mut join_nodes = Vec::new();
        let mut raw_connections: Vec<(String, ComponentPad)> = Vec::new(); // (net_name, pad)
        let mut net_attrs: HashMap<String, NetAttrTuple> = HashMap::new(); // width, current, netclass, diffpair
        let mut hub_wires: HashSet<String> = HashSet::new();

        self.elaborate_submodule(
            "",
            &top_module,
            &HashMap::new(),
            &HashMap::new(),
            &mut components,
            &mut join_nodes,
            &mut raw_connections,
            &mut net_attrs,
            &mut hub_wires,
        )?;

        // Build Join Tree and resolve canonical net names
        // Child wire -> Parent wire
        let mut parent_map: HashMap<String, String> = HashMap::new();
        for join in &join_nodes {
            let child = join.child_wire.clone();
            let parent = join.parent_wire.clone();

            // Cycle check
            let mut curr = parent.clone();
            while let Some(next) = parent_map.get(&curr) {
                if next == &child {
                    return Err(ElabError::JoinCycle(child));
                }
                curr = next.clone();
            }

            parent_map.insert(child, parent);
        }

        let resolve_canonical_net = |wire: &str| -> String {
            let mut curr = wire.to_string();
            let mut visited = HashSet::new();
            visited.insert(curr.clone());
            while let Some(parent) = parent_map.get(&curr) {
                if visited.contains(parent) {
                    break;
                }
                curr = parent.clone();
                visited.insert(curr.clone());
            }
            curr
        };

        // Group pads by canonical net
        let mut nets_map: HashMap<String, Vec<ComponentPad>> = HashMap::new();
        for (wire, pad) in &raw_connections {
            let canonical = resolve_canonical_net(wire);
            nets_map.entry(canonical).or_default().push(pad.clone());
        }

        let mut flat_nets = Vec::new();
        for (net_name, pads) in nets_map {
            let (width, current, netclass, diffpair) = net_attrs
                .get(&net_name)
                .cloned()
                .unwrap_or((None, None, None, None));
            flat_nets.push(FlatNet {
                name: net_name,
                pads,
                width,
                current,
                netclass,
                diffpair,
            });
        }
        flat_nets.sort_by(|a, b| a.name.cmp(&b.name));

        // Extract Nearby Groups
        let mut nearby_groups = Vec::new();
        for hub in &hub_wires {
            let mut members = Vec::new();
            let mut pads = Vec::new();

            // Find all child wires joined to this hub
            for join in &join_nodes {
                if &join.parent_wire == hub {
                    members.push(join.child_wire.clone());
                }
            }

            // Collect pads attached to members
            for (wire, pad) in &raw_connections {
                if members.contains(wire) {
                    pads.push(pad.clone());
                }
            }

            nearby_groups.push(NearbyGroup {
                hub_wire: hub.clone(),
                hub_path: top_name.clone(),
                members,
                pads,
            });
        }

        Ok(FlatNetlistIR {
            top_module: top_name,
            components,
            nets: flat_nets,
            nearby_groups,
            join_nodes,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn elaborate_submodule(
        &mut self,
        path_prefix: &str,
        module: &ModuleDef,
        port_bindings: &HashMap<String, String>,
        _param_bindings: &HashMap<String, String>,
        components: &mut Vec<FlatComponent>,
        join_nodes: &mut Vec<JoinNode>,
        raw_connections: &mut Vec<(String, ComponentPad)>,
        net_attrs: &mut HashMap<String, NetAttrTuple>,
        hub_wires: &mut HashSet<String>,
    ) -> Result<(), ElabError> {
        let qual_wire = |wire: &str| -> String {
            if let Some(bound) = port_bindings.get(wire) {
                bound.clone()
            } else if path_prefix.is_empty() {
                wire.to_string()
            } else {
                format!("{}.{}", path_prefix, wire)
            }
        };

        // Record wire attributes and hubs
        for item in &module.items {
            if let Item::Wire(wire) = item {
                let qname = qual_wire(&wire.name);
                let mut is_hub = false;
                let mut width = None;
                let mut current = None;
                let mut netclass = None;
                let mut diffpair = None;

                for attr in &wire.attrs {
                    match attr.key.as_str() {
                        "nearby" => is_hub = true,
                        "width" => width = attr.value.clone(),
                        "current" => current = attr.value.clone(),
                        "netclass" => netclass = attr.value.clone(),
                        "diffpair" => diffpair = attr.value.clone(),
                        _ => {}
                    }
                }

                if is_hub {
                    hub_wires.insert(qname.clone());
                }
                if width.is_some() || current.is_some() || netclass.is_some() || diffpair.is_some() {
                    net_attrs.insert(qname, (width, current, netclass, diffpair));
                }
            }
        }

        for item in &module.items {
            if let Item::Instance(inst) = item {
                let inst_path = if path_prefix.is_empty() {
                    inst.instance_name.clone()
                } else {
                    format!("{}.{}", path_prefix, inst.instance_name)
                };

                if inst.module_name == "join" {
                    // join pseudo component
                    let mut parent_wire = None;
                    let mut child_wire = None;
                    for conn in &inst.connections {
                        let wire_opt = match &conn.expr {
                            Some(Expr::Ref(r)) => Some(qual_wire(&r.ident)),
                            _ => None,
                        };
                        if conn.port_name == "P" {
                            parent_wire = wire_opt;
                        } else if conn.port_name == "C" {
                            child_wire = wire_opt;
                        }
                    }

                    if let (Some(p), Some(c)) = (parent_wire, child_wire) {
                        join_nodes.push(JoinNode {
                            instance_name: inst.instance_name.clone(),
                            parent_wire: p,
                            child_wire: c,
                        });
                    }
                    continue;
                }

                let target_mod = self
                    .modules
                    .get(&inst.module_name)
                    .cloned()
                    .ok_or_else(|| ElabError::ModuleNotFound(inst.module_name.clone()))?;

                let is_leaf = target_mod.items.is_empty();

                if is_leaf {
                    // Extract leaf properties
                    let mut prefix = "U".to_string();
                    let mut footprint = None;
                    let mut mpn = None;
                    let mut id = None;
                    let mut ref_override = None;
                    let mut dnp = false;

                    // Leaf module attrs
                    for attr in &target_mod.attrs {
                        match attr.key.as_str() {
                            "prefix" => {
                                if let Some(v) = &attr.value {
                                    prefix = v.clone();
                                }
                            }
                            "footprint" => footprint = attr.value.clone(),
                            "mpn" => mpn = attr.value.clone(),
                            _ => {}
                        }
                    }

                    // Instance attrs override
                    for attr in &inst.attrs {
                        match attr.key.as_str() {
                            "footprint" => footprint = attr.value.clone(),
                            "mpn" => mpn = attr.value.clone(),
                            "id" => id = attr.value.clone(),
                            "ref" => ref_override = attr.value.clone(),
                            "dnp" => dnp = true,
                            _ => {}
                        }
                    }

                    // Param overrides (e.g. value)
                    let mut value = None;
                    for p in &inst.param_overrides {
                        if p.name == "value" {
                            value = Some(p.value.clone());
                        }
                    }

                    // Determine refdes
                    let refdes = if let Some(r) = ref_override {
                        r
                    } else if Self::matches_prefix_digits(&inst.instance_name, &prefix) {
                        inst.instance_name.clone()
                    } else {
                        self.allocate_next_ref(&prefix)
                    };

                    // Deterministic identity key & UUID v5
                    let identity_key = id.clone().unwrap_or_else(|| inst_path.clone());
                    let uuid = Uuid::new_v5(&Uuid::NAMESPACE_OID, identity_key.as_bytes()).to_string();

                    // Collect pads
                    let mut component_pads = Vec::new();

                    for port in &target_mod.ports {
                        let mut pad_spec = None;
                        let mut etype = "passive".to_string();

                        for attr in &port.attrs {
                            if attr.key == "pad" {
                                pad_spec = attr.value.clone();
                            } else if attr.key == "etype" {
                                if let Some(v) = &attr.value {
                                    etype = v.clone();
                                }
                            }
                        }

                        let connected_net = inst.connections.iter().find(|c| c.port_name == port.name).and_then(|c| {
                            c.expr.as_ref().and_then(|e| match e {
                                Expr::Ref(r) => Some(qual_wire(&r.ident)),
                                _ => None,
                            })
                        });

                        if let Some(spec) = pad_spec {
                            for pad_num in spec.split(',') {
                                let pad_trimmed = pad_num.trim();
                                if pad_trimmed.is_empty() {
                                    continue;
                                }

                                let pad = ComponentPad {
                                    component_path: inst_path.clone(),
                                    component_ref: refdes.clone(),
                                    pad_number: pad_trimmed.to_string(),
                                    port_name: port.name.clone(),
                                    etype: etype.clone(),
                                };

                                component_pads.push(pad.clone());

                                if let Some(net) = &connected_net {
                                    raw_connections.push((net.clone(), pad));
                                }
                            }
                        }
                    }

                    components.push(FlatComponent {
                        path: inst_path,
                        module_name: inst.module_name.clone(),
                        identity_key,
                        uuid,
                        refdes,
                        prefix,
                        footprint: footprint.unwrap_or_default(),
                        mpn,
                        value,
                        dnp,
                        pads: component_pads,
                    });
                } else {
                    // Hierarchical submodule elaboration
                    let mut sub_port_bindings = HashMap::new();
                    for conn in &inst.connections {
                        if let Some(Expr::Ref(r)) = &conn.expr {
                            sub_port_bindings.insert(conn.port_name.clone(), qual_wire(&r.ident));
                        }
                    }

                    self.elaborate_submodule(
                        &inst_path,
                        &target_mod,
                        &sub_port_bindings,
                        &HashMap::new(),
                        components,
                        join_nodes,
                        raw_connections,
                        net_attrs,
                        hub_wires,
                    )?;
                }
            }
        }

        Ok(())
    }

    fn matches_prefix_digits(name: &str, prefix: &str) -> bool {
        if let Some(rest) = name.strip_prefix(prefix) {
            !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit())
        } else {
            false
        }
    }

    fn allocate_next_ref(&mut self, prefix: &str) -> String {
        let set = self.used_ref_numbers.entry(prefix.to_string()).or_default();
        let mut n = 1;
        while set.contains(&n) {
            n += 1;
        }
        set.insert(n);
        format!("{}{}", prefix, n)
    }
}
