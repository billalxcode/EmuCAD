//! Schematic-to-netlist extraction for the analog solver.

use crate::library;
use crate::model::world_pin;
use crate::model::{ComponentDefinition, ComponentId, ComponentInstance, Document, GridPos, Wire};
use device::{
    ComponentRegistry, ComponentSpec, ModelClass, ParameterKind, ParameterSpec, PinRole,
    device_parameters, device_pin_roles,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::OnceLock;

/// Electrical netlist extracted from a schematic document.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Netlist {
    pub nodes: Vec<NetNode>,
    pub ground: usize,
    pub components: Vec<NetlistComponent>,
    pub mcu: Option<McuNetlist>,
}

/// Package pin to node mapping for one MCU.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct McuNetlist {
    pub label: String,
    pub pins: BTreeMap<u8, usize>,
}

impl Netlist {
    fn new(
        nodes: Vec<NetNode>,
        components: Vec<NetlistComponent>,
        mcu: Option<McuNetlist>,
    ) -> Self {
        Self {
            nodes,
            ground: 0,
            components,
            mcu,
        }
    }
}

/// One electrical node and its schematic points.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NetNode {
    pub name: String,
    pub points: Vec<GridPos>,
}

/// One component with its resolved catalog part and pin nodes.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NetlistComponent {
    pub id: ComponentId,
    pub label: String,
    pub part: &'static str,
    pub pins: Vec<usize>,
    pub parameters: BTreeMap<String, f64>,
}

/// One editable parameter and its effective default.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ParameterField {
    pub spec: ParameterSpec,
    pub default: f64,
    pub primary: bool,
}

impl ParameterField {
    /// Returns true when the parameter is a boolean switch.
    #[must_use]
    pub fn is_boolean(&self) -> bool {
        self.spec.kind == ParameterKind::Boolean
    }

    /// Returns true when the value is valid for this parameter.
    #[must_use]
    pub fn accepts(&self, value: f64) -> bool {
        if !self.spec.is_valid(value) {
            return false;
        }
        !self.is_boolean() || value == 0.0 || value == 1.0
    }
}

/// Returns all editable parameters for a solver-backed definition.
pub(crate) fn parameter_fields(definition: &ComponentDefinition) -> Vec<ParameterField> {
    let Some(catalog) = catalog_spec(definition) else {
        return Vec::new();
    };
    let Some(device) = catalog.device.as_ref() else {
        return Vec::new();
    };
    device_parameters(device)
        .iter()
        .map(|spec| ParameterField {
            spec: *spec,
            default: parameter_default(catalog, spec),
            primary: spec.name == catalog.parameter,
        })
        .collect()
}

/// Returns the fidelity label for a solver-backed definition.
pub(crate) fn fidelity_label(definition: &ComponentDefinition) -> Option<&'static str> {
    let catalog = catalog_spec(definition)?;
    Some(device::SimulationBinding::of(catalog)?.fidelity().label())
}

/// Returns catalog pin roles for a solver-backed definition.
pub(crate) fn pin_roles(definition: &ComponentDefinition) -> &'static [PinRole] {
    let Some(catalog) = catalog_spec(definition) else {
        return &[];
    };
    let Some(device) = catalog.device.as_ref() else {
        return &[];
    };
    device_pin_roles(device)
}

fn catalog_spec(definition: &ComponentDefinition) -> Option<&'static ComponentSpec> {
    if part_kind(definition) != PartKind::Device {
        return None;
    }
    let part = builtin_part(definition.id).unwrap_or(definition.id);
    part_spec(part)
}

fn parameter_default(catalog: &ComponentSpec, spec: &ParameterSpec) -> f64 {
    if spec.name == catalog.parameter {
        return catalog.default;
    }
    spec.default
}

static SPECS: OnceLock<BTreeMap<&'static str, &'static ComponentSpec>> = OnceLock::new();

fn part_spec(id: &str) -> Option<&'static ComponentSpec> {
    SPECS
        .get_or_init(|| {
            ComponentRegistry::new()
                .entries()
                .iter()
                .map(|spec| (spec.id, spec))
                .collect()
        })
        .get(id)
        .copied()
}

/// Netlist extraction failure.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum NetlistError {
    UnknownDefinition(String),
    Unsupported(String),
    PinCount {
        component: String,
        expected: usize,
        actual: usize,
    },
    MissingGround,
    MultipleGrounds(usize),
    FloatingPin {
        component: String,
        pin: String,
    },
    ReservedLabel(String),
    LabelConflict {
        first: String,
        second: String,
    },
    InvalidValue {
        component: String,
        value: String,
    },
    UnknownParameter {
        component: String,
        name: String,
    },
    ParameterOutOfRange {
        component: String,
        name: String,
        value: f64,
    },
    OverlappingPins {
        first: String,
        second: String,
    },
    WireStub {
        point: GridPos,
    },
    FloatingIsland {
        net: String,
    },
}

impl fmt::Display for NetlistError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message())
    }
}

impl std::error::Error for NetlistError {}

impl NetlistError {
    fn message(&self) -> String {
        match self {
            Self::UnknownDefinition(id) => format!("unknown component: {id}"),
            Self::Unsupported(name) => format!("no solver model: {name}"),
            Self::PinCount { .. } | Self::FloatingPin { .. } => self.pin_message(),
            Self::MissingGround => "schematic has no GND component".to_string(),
            Self::MultipleGrounds(count) => {
                format!("schematic has {count} unconnected ground nets")
            }
            Self::ReservedLabel(_) | Self::LabelConflict { .. } => self.label_message(),
            Self::InvalidValue { component, value } => {
                format!("{component} has invalid value: {value}")
            }
            Self::UnknownParameter { .. } | Self::ParameterOutOfRange { .. } => {
                self.parameter_message()
            }
            Self::OverlappingPins { .. } => self.layout_message(),
            Self::WireStub { .. } | Self::FloatingIsland { .. } => self.net_message(),
        }
    }

    fn layout_message(&self) -> String {
        match self {
            Self::OverlappingPins { first, second } => {
                format!("pins overlap: {first} and {second}")
            }
            _ => String::new(),
        }
    }

    fn net_message(&self) -> String {
        match self {
            Self::WireStub { point } => {
                format!("wire endpoint has no connection at {}", point_text(*point))
            }
            Self::FloatingIsland { net } => {
                format!("net {net} has no source or ground")
            }
            _ => String::new(),
        }
    }

    fn parameter_message(&self) -> String {
        match self {
            Self::UnknownParameter { component, name } => {
                format!("{component} has unknown parameter: {name}")
            }
            Self::ParameterOutOfRange {
                component,
                name,
                value,
            } => format!("{component} parameter {name} is out of range: {value}"),
            _ => String::new(),
        }
    }

    fn pin_message(&self) -> String {
        match self {
            Self::PinCount {
                component,
                expected,
                actual,
            } => format!("{component} requires {expected} pins, got {actual}"),
            Self::FloatingPin { component, pin } => {
                format!("{component} pin {pin} is floating")
            }
            _ => String::new(),
        }
    }

    fn label_message(&self) -> String {
        match self {
            Self::ReservedLabel(label) => format!("net label {label} is reserved"),
            Self::LabelConflict { first, second } => {
                format!("net has conflicting labels {first} and {second}")
            }
            _ => String::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PartKind {
    Device,
    Vcc,
    Gnd,
    Mcu,
}

struct PlacedPin {
    name: &'static str,
    number: &'static str,
    point: GridPos,
}

struct PlacedPart {
    id: ComponentId,
    label: String,
    part: &'static str,
    kind: PartKind,
    pins: Vec<PlacedPin>,
    parameters: BTreeMap<String, f64>,
}

impl PlacedPart {
    fn marker(
        instance: &ComponentInstance,
        definition: &ComponentDefinition,
        kind: PartKind,
    ) -> Self {
        Self {
            id: instance.id,
            label: instance.label.clone(),
            part: definition.id,
            kind,
            pins: placed_pins(instance, definition),
            parameters: BTreeMap::new(),
        }
    }

    fn device(
        instance: &ComponentInstance,
        definition: &ComponentDefinition,
        part: &'static str,
        parameters: BTreeMap<String, f64>,
    ) -> Self {
        Self {
            id: instance.id,
            label: instance.label.clone(),
            part,
            kind: PartKind::Device,
            pins: placed_pins(instance, definition),
            parameters,
        }
    }
}

/// Extracts an electrical netlist from a schematic document.
pub(crate) fn extract(document: &Document) -> Result<Netlist, NetlistError> {
    let parts = collect_parts(document)?;
    validate_pin_positions(&parts)?;
    validate_wire_endpoints(document, &parts)?;
    let points = collect_points(document, &parts);
    let index = PointIndex::new(points);
    let mut set = DisjointSet::new(index.len());
    union_wires(document, &index, &mut set);
    union_labels(document, &index, &mut set);
    union_vcc(&parts, &index, &mut set);
    let ground = ground_root(&parts, &index, &mut set)?;
    let table = build_nodes(document, &parts, &index, &mut set, ground)?;
    validate_connections(&parts, &index, &mut set)?;
    validate_driver_islands(&parts, &index, &mut set, &table)?;
    let components = build_components(&parts, &index, &mut set, &table);
    let mcu = build_mcu(&parts, &index, &mut set, &table);
    Ok(Netlist::new(table.nodes, components, mcu))
}

fn point_text(point: GridPos) -> String {
    format!("({}, {})", point.x, point.y)
}

fn validate_pin_positions(parts: &[PlacedPart]) -> Result<(), NetlistError> {
    let mut seen: BTreeMap<GridPos, String> = BTreeMap::new();
    for part in parts {
        for pin in &part.pins {
            let name = format!("{}.{}", part.label, pin.name);
            if let Some(existing) = seen.insert(pin.point, name.clone()) {
                return Err(NetlistError::OverlappingPins {
                    first: existing,
                    second: name,
                });
            }
        }
    }
    Ok(())
}

fn validate_wire_endpoints(document: &Document, parts: &[PlacedPart]) -> Result<(), NetlistError> {
    let pins: BTreeSet<GridPos> = parts
        .iter()
        .flat_map(|part| part.pins.iter().map(|pin| pin.point))
        .collect();
    let junctions: BTreeSet<GridPos> = document.junctions().into_iter().collect();
    let mut endpoints: BTreeMap<GridPos, usize> = BTreeMap::new();
    for wire in document.wires() {
        count_endpoint(&mut endpoints, wire.points.first());
        count_endpoint(&mut endpoints, wire.points.last());
    }
    for (point, count) in endpoints {
        if count >= 2 || pins.contains(&point) || junctions.contains(&point) {
            continue;
        }
        return Err(NetlistError::WireStub { point });
    }
    Ok(())
}

fn count_endpoint(endpoints: &mut BTreeMap<GridPos, usize>, point: Option<&GridPos>) {
    if let Some(point) = point {
        *endpoints.entry(*point).or_default() += 1;
    }
}

fn is_source(part: &str) -> bool {
    part.contains("voltage_source") || part.contains("current_source") || part.contains("battery")
}

fn validate_driver_islands(
    parts: &[PlacedPart],
    index: &PointIndex,
    set: &mut DisjointSet,
    table: &NodeTable,
) -> Result<(), NetlistError> {
    let mut islands = DisjointSet::new(table.nodes.len());
    let mut driven = vec![false; table.nodes.len()];
    driven[0] = true;
    for part in parts {
        let nets = part_nets(part, index, set, table);
        union_nets(&mut islands, &nets);
        mark_driven(part, &nets, &mut driven);
    }
    floating_island(&mut islands, &driven, &table.nodes)
}

fn mark_driven(part: &PlacedPart, nets: &[usize], driven: &mut [bool]) {
    if part.kind != PartKind::Gnd && part.kind != PartKind::Mcu && !is_source(part.part) {
        return;
    }
    for &net in nets {
        driven[net] = true;
    }
}

fn floating_island(
    islands: &mut DisjointSet,
    driven: &[bool],
    nodes: &[NetNode],
) -> Result<(), NetlistError> {
    let mut roots = vec![false; nodes.len()];
    for (net, is_driven) in driven.iter().enumerate() {
        if *is_driven {
            roots[islands.find(net)] = true;
        }
    }
    for (index, node) in nodes.iter().enumerate() {
        if !roots[islands.find(index)] {
            return Err(NetlistError::FloatingIsland {
                net: node.name.clone(),
            });
        }
    }
    Ok(())
}

fn part_nets(
    part: &PlacedPart,
    index: &PointIndex,
    set: &mut DisjointSet,
    table: &NodeTable,
) -> Vec<usize> {
    part.pins
        .iter()
        .filter(|pin| index.contains(pin.point))
        .map(|pin| {
            let root = set.find(index.get(pin.point));
            *table.by_root.get(&root).expect("pin root has node")
        })
        .collect()
}

fn union_nets(islands: &mut DisjointSet, nets: &[usize]) {
    let Some((&first, rest)) = nets.split_first() else {
        return;
    };
    for &net in rest {
        islands.union(first, net);
    }
}

fn collect_parts(document: &Document) -> Result<Vec<PlacedPart>, NetlistError> {
    document.components().iter().map(collect_part).collect()
}

fn collect_part(instance: &ComponentInstance) -> Result<PlacedPart, NetlistError> {
    let definition = library::find(instance.definition)
        .ok_or_else(|| NetlistError::UnknownDefinition(instance.definition.to_string()))?;
    let kind = part_kind(definition);
    if kind != PartKind::Device {
        return Ok(PlacedPart::marker(instance, definition, kind));
    }
    let spec = device_spec(definition)?;
    require_pin_count(instance, definition, spec)?;
    let parameters = parameters_for(instance, spec)?;
    Ok(PlacedPart::device(
        instance, definition, spec.id, parameters,
    ))
}

fn part_kind(definition: &ComponentDefinition) -> PartKind {
    match definition.id {
        "vcc" => PartKind::Vcc,
        "gnd" => PartKind::Gnd,
        "atmega328p" => PartKind::Mcu,
        _ => PartKind::Device,
    }
}

fn device_spec(definition: &ComponentDefinition) -> Result<&'static ComponentSpec, NetlistError> {
    let part = builtin_part(definition.id).unwrap_or(definition.id);
    let spec = part_spec(part)
        .ok_or_else(|| NetlistError::UnknownDefinition(definition.id.to_string()))?;
    if spec.device.is_none() {
        return Err(NetlistError::Unsupported(definition.name.to_string()));
    }
    Ok(spec)
}

fn builtin_part(id: &str) -> Option<&'static str> {
    match id {
        "resistor" => Some("passive.resistor.ideal_resistor"),
        "led" => Some("diode.led.red_led"),
        "button" => Some("switch.mechanical.pushbutton"),
        _ => None,
    }
}

fn require_pin_count(
    instance: &ComponentInstance,
    definition: &ComponentDefinition,
    spec: &ComponentSpec,
) -> Result<(), NetlistError> {
    let actual = definition.pins.len();
    let expected = usize::from(spec.pins);
    if actual == expected {
        return Ok(());
    }
    Err(NetlistError::PinCount {
        component: instance.label.clone(),
        expected,
        actual,
    })
}

fn parameters_for(
    instance: &ComponentInstance,
    catalog: &ComponentSpec,
) -> Result<BTreeMap<String, f64>, NetlistError> {
    validate_overrides(instance, catalog)?;
    let mut parameters = default_parameters(catalog);
    parameters.extend(
        instance
            .parameters
            .iter()
            .map(|(name, value)| (name.clone(), *value)),
    );
    if instance.parameters.contains_key(catalog.parameter) {
        return Ok(parameters);
    }
    apply_value_string(instance, catalog, parameters)
}

fn apply_value_string(
    instance: &ComponentInstance,
    catalog: &ComponentSpec,
    mut parameters: BTreeMap<String, f64>,
) -> Result<BTreeMap<String, f64>, NetlistError> {
    match parse_value(&instance.value) {
        Some(value) => {
            parameters.insert(catalog.parameter.to_string(), value);
            Ok(parameters)
        }
        None if value_is_optional(instance) => Ok(parameters),
        None => Err(NetlistError::InvalidValue {
            component: instance.label.clone(),
            value: instance.value.clone(),
        }),
    }
}

fn default_parameters(catalog: &ComponentSpec) -> BTreeMap<String, f64> {
    let Some(device) = catalog.device.as_ref() else {
        return catalog.default_parameters();
    };
    let mut parameters: BTreeMap<String, f64> = device_parameters(device)
        .iter()
        .map(|spec| (spec.name.to_string(), parameter_default(catalog, spec)))
        .collect();
    if catalog.class == ModelClass::LogicGate {
        parameters.insert("gate".to_string(), f64::from(catalog.variant));
    }
    parameters
}

fn validate_overrides(
    instance: &ComponentInstance,
    catalog: &ComponentSpec,
) -> Result<(), NetlistError> {
    let Some(device) = catalog.device.as_ref() else {
        return Ok(());
    };
    for (name, value) in &instance.parameters {
        let Some(spec) = parameter_spec(device, name) else {
            return Err(unknown_parameter(instance, name));
        };
        if !spec.is_valid(*value) || !boolean_value_valid(spec, *value) {
            return Err(out_of_range(instance, name, *value));
        }
    }
    Ok(())
}

fn parameter_spec(device: &kernel::DeviceKind, name: &str) -> Option<&'static ParameterSpec> {
    device_parameters(device)
        .iter()
        .find(|spec| spec.name == name)
}

fn boolean_value_valid(spec: &ParameterSpec, value: f64) -> bool {
    if spec.kind != ParameterKind::Boolean {
        return true;
    }
    value == 0.0 || value == 1.0
}

fn unknown_parameter(instance: &ComponentInstance, name: &str) -> NetlistError {
    NetlistError::UnknownParameter {
        component: instance.label.clone(),
        name: name.to_string(),
    }
}

fn out_of_range(instance: &ComponentInstance, name: &str, value: f64) -> NetlistError {
    NetlistError::ParameterOutOfRange {
        component: instance.label.clone(),
        name: name.to_string(),
        value,
    }
}

fn value_is_optional(instance: &ComponentInstance) -> bool {
    instance.value.trim().is_empty() || builtin_part(instance.definition).is_some()
}

fn parse_value(text: &str) -> Option<f64> {
    let token = text.split_whitespace().next()?;
    let value = token.parse::<f64>().ok()?;
    value.is_finite().then_some(value)
}

fn placed_pins(instance: &ComponentInstance, definition: &ComponentDefinition) -> Vec<PlacedPin> {
    definition
        .pins
        .iter()
        .map(|pin| PlacedPin {
            name: pin.name,
            number: pin.number,
            point: world_pin(instance, definition, pin),
        })
        .collect()
}

struct PointIndex {
    points: Vec<GridPos>,
    lookup: BTreeMap<GridPos, usize>,
}

impl PointIndex {
    fn new(points: BTreeSet<GridPos>) -> Self {
        let points: Vec<_> = points.into_iter().collect();
        let lookup = points
            .iter()
            .enumerate()
            .map(|(index, point)| (*point, index))
            .collect();
        Self { points, lookup }
    }

    fn get(&self, point: GridPos) -> usize {
        *self.lookup.get(&point).expect("netlist point indexed")
    }

    fn contains(&self, point: GridPos) -> bool {
        self.lookup.contains_key(&point)
    }

    fn len(&self) -> usize {
        self.points.len()
    }
}

struct DisjointSet {
    parent: Vec<usize>,
    rank: Vec<u8>,
}

impl DisjointSet {
    fn new(len: usize) -> Self {
        Self {
            parent: (0..len).collect(),
            rank: vec![0; len],
        }
    }

    fn find(&mut self, index: usize) -> usize {
        let parent = self.parent[index];
        if parent == index {
            return index;
        }
        let root = self.find(parent);
        self.parent[index] = root;
        root
    }

    fn union(&mut self, first: usize, second: usize) {
        let first_root = self.find(first);
        let second_root = self.find(second);
        if first_root == second_root {
            return;
        }
        if self.rank[first_root] < self.rank[second_root] {
            self.parent[first_root] = second_root;
            return;
        }
        self.parent[second_root] = first_root;
        if self.rank[first_root] == self.rank[second_root] {
            self.rank[first_root] += 1;
        }
    }
}

fn collect_points(document: &Document, parts: &[PlacedPart]) -> BTreeSet<GridPos> {
    let mut points = BTreeSet::new();
    for wire in document.wires() {
        points.extend(wire.points.iter().copied());
    }
    let junctions: BTreeSet<GridPos> = document.junctions().into_iter().collect();
    for part in parts {
        for pin in &part.pins {
            let connected = points.contains(&pin.point) || junctions.contains(&pin.point);
            if part.kind == PartKind::Mcu && !connected {
                continue;
            }
            points.insert(pin.point);
        }
    }
    points
}

fn union_wires(document: &Document, index: &PointIndex, set: &mut DisjointSet) {
    for wire in document.wires() {
        union_polyline(&wire.points, index, set);
    }
}

fn union_polyline(points: &[GridPos], index: &PointIndex, set: &mut DisjointSet) {
    let Some((&first, rest)) = points.split_first() else {
        return;
    };
    let first_index = index.get(first);
    for point in rest {
        set.union(first_index, index.get(*point));
    }
}

fn union_labels(document: &Document, index: &PointIndex, set: &mut DisjointSet) {
    let mut labels: BTreeMap<&str, usize> = BTreeMap::new();
    for wire in document.wires() {
        let Some(label) = wire_label(wire) else {
            continue;
        };
        let Some(&first) = wire.points.first() else {
            continue;
        };
        let point = index.get(first);
        if let Some(&existing) = labels.get(label) {
            set.union(existing, point);
        } else {
            labels.insert(label, point);
        }
    }
}

fn wire_label(wire: &Wire) -> Option<&str> {
    wire.label.as_deref().filter(|text| !text.is_empty())
}

fn union_vcc(parts: &[PlacedPart], index: &PointIndex, set: &mut DisjointSet) {
    let mut first = None;
    for pin in vcc_pins(parts) {
        let point = index.get(pin.point);
        if let Some(root) = first {
            set.union(root, point);
        } else {
            first = Some(point);
        }
    }
}

fn vcc_pins(parts: &[PlacedPart]) -> impl Iterator<Item = &PlacedPin> {
    parts
        .iter()
        .filter(|part| part.kind == PartKind::Vcc)
        .flat_map(|part| part.pins.iter())
}

fn ground_root(
    parts: &[PlacedPart],
    index: &PointIndex,
    set: &mut DisjointSet,
) -> Result<usize, NetlistError> {
    let roots: BTreeSet<_> = parts
        .iter()
        .filter(|part| part.kind == PartKind::Gnd)
        .flat_map(|part| part.pins.iter())
        .map(|pin| set.find(index.get(pin.point)))
        .collect();
    match roots.len() {
        0 => Err(NetlistError::MissingGround),
        1 => Ok(*roots.iter().next().expect("one ground root")),
        count => Err(NetlistError::MultipleGrounds(count)),
    }
}

struct NodeTable {
    nodes: Vec<NetNode>,
    by_root: BTreeMap<usize, usize>,
}

fn build_nodes(
    document: &Document,
    parts: &[PlacedPart],
    index: &PointIndex,
    set: &mut DisjointSet,
    ground: usize,
) -> Result<NodeTable, NetlistError> {
    let points = points_by_root(index, set);
    let reps = representatives(&points);
    let labels = node_labels(document, parts, index, set)?;
    let roots = ordered_roots(&reps, ground);
    Ok(assemble_nodes(ground, &roots, &labels, &points))
}

fn node_labels(
    document: &Document,
    parts: &[PlacedPart],
    index: &PointIndex,
    set: &mut DisjointSet,
) -> Result<BTreeMap<usize, String>, NetlistError> {
    let mut labels = root_labels(document, index, set)?;
    apply_vcc_names(parts, index, set, &mut labels)?;
    Ok(labels)
}

fn assemble_nodes(
    ground: usize,
    roots: &[usize],
    labels: &BTreeMap<usize, String>,
    points: &BTreeMap<usize, Vec<GridPos>>,
) -> NodeTable {
    let mut by_root = BTreeMap::new();
    by_root.insert(ground, 0);
    let mut nodes = vec![node("0", ground, points)];
    for &root in roots {
        let name = labels
            .get(&root)
            .cloned()
            .unwrap_or_else(|| format!("n{}", nodes.len()));
        nodes.push(node(&name, root, points));
        by_root.insert(root, nodes.len() - 1);
    }
    NodeTable { nodes, by_root }
}

fn points_by_root(index: &PointIndex, set: &mut DisjointSet) -> BTreeMap<usize, Vec<GridPos>> {
    let mut points: BTreeMap<usize, Vec<GridPos>> = BTreeMap::new();
    for &point in &index.points {
        let root = set.find(index.get(point));
        points.entry(root).or_default().push(point);
    }
    points
}

fn representatives(points: &BTreeMap<usize, Vec<GridPos>>) -> BTreeMap<usize, GridPos> {
    points
        .iter()
        .filter_map(|(&root, points)| points.first().map(|point| (root, *point)))
        .collect()
}

fn ordered_roots(reps: &BTreeMap<usize, GridPos>, ground: usize) -> Vec<usize> {
    let mut roots: Vec<_> = reps.iter().map(|(&root, &point)| (point, root)).collect();
    roots.sort();
    roots
        .into_iter()
        .map(|(_, root)| root)
        .filter(|&root| root != ground)
        .collect()
}

fn node(name: &str, root: usize, points: &BTreeMap<usize, Vec<GridPos>>) -> NetNode {
    NetNode {
        name: name.to_string(),
        points: points.get(&root).cloned().unwrap_or_default(),
    }
}

fn root_labels(
    document: &Document,
    index: &PointIndex,
    set: &mut DisjointSet,
) -> Result<BTreeMap<usize, String>, NetlistError> {
    let mut labels = BTreeMap::new();
    for wire in document.wires() {
        let Some(label) = wire_label(wire) else {
            continue;
        };
        let Some(&first) = wire.points.first() else {
            continue;
        };
        let root = set.find(index.get(first));
        insert_label(&mut labels, root, label)?;
    }
    Ok(labels)
}

fn apply_vcc_names(
    parts: &[PlacedPart],
    index: &PointIndex,
    set: &mut DisjointSet,
    labels: &mut BTreeMap<usize, String>,
) -> Result<(), NetlistError> {
    for pin in vcc_pins(parts) {
        let root = set.find(index.get(pin.point));
        insert_label(labels, root, "VCC")?;
    }
    Ok(())
}

fn insert_label(
    labels: &mut BTreeMap<usize, String>,
    root: usize,
    label: &str,
) -> Result<(), NetlistError> {
    if label == "0" || label == "GND" {
        return Err(NetlistError::ReservedLabel(label.to_string()));
    }
    match labels.get(&root) {
        Some(existing) if existing != label => Err(NetlistError::LabelConflict {
            first: existing.clone(),
            second: label.to_string(),
        }),
        Some(_) => Ok(()),
        None => {
            labels.insert(root, label.to_string());
            Ok(())
        }
    }
}

fn validate_connections(
    parts: &[PlacedPart],
    index: &PointIndex,
    set: &mut DisjointSet,
) -> Result<(), NetlistError> {
    let counts = pin_counts(parts, index, set);
    for part in parts.iter().filter(|part| part.kind == PartKind::Device) {
        for pin in &part.pins {
            let root = set.find(index.get(pin.point));
            if counts.get(&root).copied().unwrap_or(0) < 2 {
                return Err(NetlistError::FloatingPin {
                    component: part.label.clone(),
                    pin: pin.name.to_string(),
                });
            }
        }
    }
    Ok(())
}

fn pin_counts(
    parts: &[PlacedPart],
    index: &PointIndex,
    set: &mut DisjointSet,
) -> BTreeMap<usize, usize> {
    let mut counts = BTreeMap::new();
    for pin in parts
        .iter()
        .flat_map(|part| part.pins.iter())
        .filter(|pin| index.contains(pin.point))
    {
        let root = set.find(index.get(pin.point));
        *counts.entry(root).or_default() += 1;
    }
    counts
}

fn build_components(
    parts: &[PlacedPart],
    index: &PointIndex,
    set: &mut DisjointSet,
    table: &NodeTable,
) -> Vec<NetlistComponent> {
    parts
        .iter()
        .filter(|part| part.kind == PartKind::Device)
        .map(|part| NetlistComponent {
            id: part.id,
            label: part.label.clone(),
            part: part.part,
            pins: component_pins(part, index, set, table),
            parameters: part.parameters.clone(),
        })
        .collect()
}

fn component_pins(
    part: &PlacedPart,
    index: &PointIndex,
    set: &mut DisjointSet,
    table: &NodeTable,
) -> Vec<usize> {
    part.pins
        .iter()
        .filter(|pin| index.contains(pin.point))
        .map(|pin| {
            let root = set.find(index.get(pin.point));
            *table.by_root.get(&root).expect("pin root has node")
        })
        .collect()
}

fn build_mcu(
    parts: &[PlacedPart],
    index: &PointIndex,
    set: &mut DisjointSet,
    table: &NodeTable,
) -> Option<McuNetlist> {
    let part = parts.iter().find(|part| part.kind == PartKind::Mcu)?;
    let pins = part
        .pins
        .iter()
        .filter_map(|pin| mcu_pin_node(pin, index, set, table))
        .collect();
    Some(McuNetlist {
        label: part.label.clone(),
        pins,
    })
}

fn mcu_pin_node(
    pin: &PlacedPin,
    index: &PointIndex,
    set: &mut DisjointSet,
    table: &NodeTable,
) -> Option<(u8, usize)> {
    if !index.contains(pin.point) {
        return None;
    }
    let number = pin.number.parse().ok()?;
    let root = set.find(index.get(pin.point));
    let node = *table.by_root.get(&root)?;
    Some((number, node))
}

mod circuit;
#[cfg(test)]
mod mixed_session;
#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) use circuit::build_circuit_with_sources;
pub(crate) use circuit::{build_circuit, device_currents, node_voltages, solve_dc};
#[cfg(test)]
pub(crate) use mixed_session::{MixedPinPlan, build_mixed_session};
