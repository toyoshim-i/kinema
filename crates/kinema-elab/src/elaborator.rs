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
    #[error("Duplicate reference designator '{0}'")]
    DuplicateRef(String),
    #[error("Duplicate component UUID '{0}' on component '{1}'")]
    DuplicateUuid(String, String),
    #[error("Duplicate component path '{0}'")]
    DuplicatePath(String),
    #[error("Duplicate net '{0}' in flat netlist")]
    DuplicateNet(String),
    #[error("Duplicate pad number '{1}' on component '{0}'")]
    DuplicatePadNumber(String, String),
    #[error("Dangling pad '{1}.{2}' referenced in net '{0}'")]
    DanglingPad(String, String, String),
    #[error("Duplicate pad '{1}.{2}' in net '{0}'")]
    DuplicatePadInNet(String, String, String),
    #[error("Pad '{0}.{1}' connected to multiple nets: '{2}' and '{3}'")]
    PadMultiNet(String, String, String, String),
    #[error("Invalid constraint on net '{0}': {1}")]
    InvalidConstraint(String, String),
    #[error("Elaboration failed: {0}")]
    Generic(String),
}

pub type NetAttrTuple = (Option<String>, Option<String>, Option<String>, Option<String>);

pub struct Elaborator {
    modules: HashMap<String, ModuleDef>,
    used_ref_numbers: HashMap<String, HashSet<u32>>,
    assigned_refdes: HashSet<String>,
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
            assigned_refdes: HashSet::new(),
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

        // Pre-scan all explicit refs across the module tree to reserve them
        self.prescan_refdes(&top_module);

        let mut components = Vec::new();
        let mut join_nodes = Vec::new();
        let mut raw_connections: Vec<(String, ComponentPad)> = Vec::new(); // (net_name, pad)
        let mut net_attrs: HashMap<String, NetAttrTuple> = HashMap::new(); // width, current, netclass, diffpair
        let mut hub_wires: HashSet<String> = HashSet::new();

        let project_ns = Uuid::new_v5(&Uuid::NAMESPACE_OID, top_name.as_bytes());

        self.elaborate_submodule(
            "",
            &top_module,
            &HashMap::new(),
            &HashMap::new(),
            &project_ns,
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

        // Group net attributes by canonical net, inheriting child and base wire attributes.
        // Canonical net's own attributes take precedence over child wire attributes.
        let mut canonical_attrs: HashMap<String, NetAttrTuple> = HashMap::new();
        let mut sorted_wires: Vec<_> = net_attrs.keys().cloned().collect();
        sorted_wires.sort();
        for wire in sorted_wires {
            let attrs = &net_attrs[&wire];
            let canonical = resolve_canonical_net(&wire);
            let is_canonical = wire == canonical;
            let entry = canonical_attrs.entry(canonical).or_insert((None, None, None, None));
            if (is_canonical && attrs.0.is_some()) || entry.0.is_none() { entry.0 = attrs.0.clone(); }
            if (is_canonical && attrs.1.is_some()) || entry.1.is_none() { entry.1 = attrs.1.clone(); }
            if (is_canonical && attrs.2.is_some()) || entry.2.is_none() { entry.2 = attrs.2.clone(); }
            if (is_canonical && attrs.3.is_some()) || entry.3.is_none() { entry.3 = attrs.3.clone(); }
        }

        let mut flat_nets = Vec::new();
        for (net_name, pads) in nets_map {
            let mut width = None;
            let mut current = None;
            let mut netclass = None;
            let mut diffpair = None;

            let mut merge_attr = |w: Option<String>, c: Option<String>, n: Option<String>, d: Option<String>| {
                if width.is_none() && w.is_some() { width = w; }
                if current.is_none() && c.is_some() { current = c; }
                if netclass.is_none() && n.is_some() { netclass = n; }
                if diffpair.is_none() && d.is_some() { diffpair = d; }
            };

            if let Some((w, c, n, d)) = canonical_attrs.get(&net_name) {
                merge_attr(w.clone(), c.clone(), n.clone(), d.clone());
            }
            if let Some(idx_pos) = net_name.rfind('[') {
                let base = &net_name[..idx_pos];
                let canonical_base = resolve_canonical_net(base);
                if let Some((w, c, n, d)) = canonical_attrs.get(&canonical_base) {
                    merge_attr(w.clone(), c.clone(), n.clone(), d.clone());
                }
            }

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

        let ir = FlatNetlistIR {
            top_module: top_name,
            components,
            nets: flat_nets,
            nearby_groups,
            join_nodes,
        };
        ir.validate()?;
        Ok(ir)
    }

    #[allow(clippy::too_many_arguments)]
    fn elaborate_submodule(
        &mut self,
        path_prefix: &str,
        module: &ModuleDef,
        port_bindings: &HashMap<String, Vec<String>>,
        param_bindings: &HashMap<String, String>,
        project_ns: &Uuid,
        components: &mut Vec<FlatComponent>,
        join_nodes: &mut Vec<JoinNode>,
        raw_connections: &mut Vec<(String, ComponentPad)>,
        net_attrs: &mut HashMap<String, NetAttrTuple>,
        hub_wires: &mut HashSet<String>,
    ) -> Result<(), ElabError> {
        let resolve_ref_bits = |r: &RefExpr| -> Vec<String> {
            let mut result = Vec::new();
            if let Some(bound_bits) = port_bindings.get(&r.ident) {
                if let Some(idx) = &r.index {
                    match idx {
                        RefIndex::Single(i) => {
                            let port_range = module.ports.iter().find(|p| p.name == r.ident).and_then(|p| p.range.as_ref());
                            let msb = port_range.map(|rng| rng.msb).unwrap_or(0);
                            let lsb = port_range.map(|rng| rng.lsb).unwrap_or(0);
                            if *i >= lsb && *i <= msb {
                                let offset = (msb - *i) as usize;
                                if offset < bound_bits.len() {
                                    result.push(bound_bits[offset].clone());
                                } else if let Some(first) = bound_bits.first() {
                                    result.push(first.clone());
                                }
                            } else if let Some(first) = bound_bits.first() {
                                result.push(first.clone());
                            }
                        }
                        RefIndex::Range(msb, lsb) => {
                            let port_range = module.ports.iter().find(|p| p.name == r.ident).and_then(|p| p.range.as_ref());
                            let p_msb = port_range.map(|rng| rng.msb).unwrap_or(*msb);
                            for bit in (*lsb..=*msb).rev() {
                                let offset = (p_msb - bit) as usize;
                                if offset < bound_bits.len() {
                                    result.push(bound_bits[offset].clone());
                                }
                            }
                        }
                    }
                } else {
                    result.extend(bound_bits.clone());
                }
            } else {
                let qname = if path_prefix.is_empty() {
                    r.ident.clone()
                } else {
                    format!("{}.{}", path_prefix, r.ident)
                };

                if let Some(idx) = &r.index {
                    match idx {
                        RefIndex::Single(i) => {
                            result.push(format!("{}[{}]", qname, i));
                        }
                        RefIndex::Range(msb, lsb) => {
                            for bit in (*lsb..=*msb).rev() {
                                result.push(format!("{}[{}]", qname, bit));
                            }
                        }
                    }
                } else {
                    let wire_range = module
                        .items
                        .iter()
                        .find_map(|item| {
                            if let Item::Wire(w) = item {
                                if w.name == r.ident {
                                    return w.range.as_ref();
                                }
                            }
                            None
                        })
                        .or_else(|| {
                            module.ports.iter().find(|p| p.name == r.ident).and_then(|p| p.range.as_ref())
                        });

                    if let Some(range) = wire_range {
                        for bit in (range.lsb..=range.msb).rev() {
                            result.push(format!("{}[{}]", qname, bit));
                        }
                    } else {
                        result.push(qname);
                    }
                }
            }
            result
        };

        let resolve_expr_bits = |expr: &Expr| -> Vec<String> {
            match expr {
                Expr::Ref(r) => resolve_ref_bits(r),
                Expr::Concat(list) => {
                    let mut bits = Vec::new();
                    for r in list {
                        bits.extend(resolve_ref_bits(r));
                    }
                    bits
                }
            }
        };

        // Record wire and port attributes and hubs
        let record_attr = |qname: &str, attrs: &[Attr], hubs: &mut HashSet<String>, n_attrs: &mut HashMap<String, NetAttrTuple>| {
            let mut is_hub = false;
            let mut width = None;
            let mut current = None;
            let mut netclass = None;
            let mut diffpair = None;

            for attr in attrs {
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
                hubs.insert(qname.to_string());
            }
            if width.is_some() || current.is_some() || netclass.is_some() || diffpair.is_some() {
                n_attrs.insert(qname.to_string(), (width, current, netclass, diffpair));
            }
        };

        for port in &module.ports {
            let qname = if path_prefix.is_empty() {
                port.name.clone()
            } else {
                format!("{}.{}", path_prefix, port.name)
            };
            record_attr(&qname, &port.attrs, hub_wires, net_attrs);
        }

        for item in &module.items {
            if let Item::Wire(wire) = item {
                let qname = if path_prefix.is_empty() {
                    wire.name.clone()
                } else {
                    format!("{}.{}", path_prefix, wire.name)
                };
                record_attr(&qname, &wire.attrs, hub_wires, net_attrs);
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
                        let wire_opt = conn.expr.as_ref().and_then(|e| {
                            let bits = resolve_expr_bits(e);
                            bits.into_iter().next()
                        });
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

                // Compute parameter bindings: defaults + overrides
                let mut sub_params = HashMap::new();
                for p in &target_mod.params {
                    if !p.value.is_empty() {
                        sub_params.insert(p.name.clone(), p.value.clone());
                    }
                }
                for p in &inst.param_overrides {
                    let val = if let Some(parent_val) = param_bindings.get(&p.value) {
                        parent_val.clone()
                    } else {
                        p.value.clone()
                    };
                    sub_params.insert(p.name.clone(), val);
                }

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

                    let value = sub_params.get("value").cloned();

                    // Determine refdes
                    let refdes = if let Some(r) = ref_override {
                        if let Some((pfx, num)) = Self::split_prefix_digits(&r) {
                            self.used_ref_numbers.entry(pfx).or_default().insert(num);
                        }
                        r
                    } else if Self::matches_prefix_digits(&inst.instance_name, &prefix) {
                        if let Some((pfx, num)) = Self::split_prefix_digits(&inst.instance_name) {
                            self.used_ref_numbers.entry(pfx).or_default().insert(num);
                        }
                        inst.instance_name.clone()
                    } else {
                        self.allocate_next_ref(&prefix)
                    };

                    if !self.assigned_refdes.insert(refdes.clone()) {
                        return Err(ElabError::DuplicateRef(refdes));
                    }

                    // Deterministic identity key & UUID v5 scoped by project namespace
                    let identity_key = id.clone().unwrap_or_else(|| inst_path.clone());
                    let uuid = Uuid::new_v5(project_ns, identity_key.as_bytes()).to_string();

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

                        let conn_expr = inst.connections.iter().find(|c| c.port_name == port.name).and_then(|c| c.expr.as_ref());
                        let connected_bits = conn_expr.map(&resolve_expr_bits).unwrap_or_default();

                        if let Some(spec) = pad_spec {
                            let pad_nums: Vec<&str> = spec.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
                            let is_vector_port = port.range.is_some();
                            let port_width = port.range.as_ref().map(|r| r.msb - r.lsb + 1).unwrap_or(1) as usize;

                            for (idx, pad_trimmed) in pad_nums.iter().enumerate() {
                                let pad = ComponentPad {
                                    component_path: inst_path.clone(),
                                    component_ref: refdes.clone(),
                                    pad_number: pad_trimmed.to_string(),
                                    port_name: port.name.clone(),
                                    etype: etype.clone(),
                                };

                                component_pads.push(pad.clone());

                                let net_opt = if is_vector_port && pad_nums.len() == port_width {
                                    connected_bits.get(idx).cloned()
                                } else {
                                    connected_bits.first().cloned()
                                };

                                if let Some(net) = net_opt {
                                    raw_connections.push((net, pad));
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
                    let mut sub_port_bindings: HashMap<String, Vec<String>> = HashMap::new();
                    for conn in &inst.connections {
                        if let Some(expr) = &conn.expr {
                            let bits = resolve_expr_bits(expr);
                            sub_port_bindings.insert(conn.port_name.clone(), bits);
                        }
                    }

                    self.elaborate_submodule(
                        &inst_path,
                        &target_mod,
                        &sub_port_bindings,
                        &sub_params,
                        project_ns,
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

    fn split_prefix_digits(name: &str) -> Option<(String, u32)> {
        let mut split_idx = name.len();
        for (i, c) in name.char_indices().rev() {
            if c.is_ascii_digit() {
                split_idx = i;
            } else {
                break;
            }
        }
        if split_idx < name.len() && split_idx > 0 {
            let prefix = &name[..split_idx];
            let num_str = &name[split_idx..];
            if let Ok(num) = num_str.parse::<u32>() {
                return Some((prefix.to_string(), num));
            }
        }
        None
    }

    fn prescan_refdes(&mut self, module: &ModuleDef) {
        for item in &module.items {
            if let Item::Instance(inst) = item {
                if inst.module_name == "join" {
                    continue;
                }
                for attr in &inst.attrs {
                    if attr.key == "ref" {
                        if let Some(r) = &attr.value {
                            if let Some((pfx, num)) = Self::split_prefix_digits(r) {
                                self.used_ref_numbers.entry(pfx).or_default().insert(num);
                            }
                        }
                    }
                }
                if let Some((pfx, num)) = Self::split_prefix_digits(&inst.instance_name) {
                    self.used_ref_numbers.entry(pfx).or_default().insert(num);
                }

                if let Some(target_mod) = self.modules.get(&inst.module_name).cloned() {
                    if !target_mod.items.is_empty() {
                        self.prescan_refdes(&target_mod);
                    }
                }
            }
        }
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
