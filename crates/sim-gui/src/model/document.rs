use super::*;

impl Document {
    /// Creates an empty document.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a document from restored components and wires.
    #[must_use]
    pub fn restore(components: Vec<ComponentInstance>, wires: Vec<Wire>) -> Self {
        let mut wires = wires;
        wires.sort_by_key(|wire| wire.id);
        let next_id = components
            .iter()
            .map(|component| component.id.0 + 1)
            .max()
            .unwrap_or(0);
        let next_wire_id = wires.iter().map(|wire| wire.id.0 + 1).max().unwrap_or(0);
        let wire_index = SegmentIndex::rebuild(&wires);
        Self {
            components,
            wires,
            wire_index,
            next_id,
            next_wire_id,
        }
    }

    /// Returns all components.
    #[must_use]
    pub fn components(&self) -> &[ComponentInstance] {
        &self.components
    }

    /// Returns all wires.
    #[must_use]
    pub fn wires(&self) -> &[Wire] {
        &self.wires
    }

    /// Adds a validated wire polyline.
    pub fn add_wire(
        &mut self,
        library: &[ComponentDefinition],
        points: Vec<GridPos>,
        class: NetClass,
    ) -> Option<WireId> {
        if !self.validate_wire(library, &points, None) {
            return None;
        }
        let id = WireId(self.next_wire_id);
        self.next_wire_id += 1;
        let wire = new_wire(id, points, class);
        self.wire_index.insert(&wire);
        self.wires.push(wire);
        Some(id)
    }

    /// Replaces a wire polyline when the result is valid.
    pub fn update_wire_points(
        &mut self,
        library: &[ComponentDefinition],
        id: WireId,
        points: Vec<GridPos>,
    ) -> bool {
        if !self.validate_wire(library, &points, Some(id)) {
            return false;
        }
        let Some(wire) = self.wires.iter_mut().find(|wire| wire.id == id) else {
            return false;
        };
        self.wire_index.remove(wire);
        wire.points = points;
        self.wire_index.insert(wire);
        true
    }

    /// Tests whether a single segment can be added.
    #[must_use]
    pub fn can_add_segment(
        &self,
        library: &[ComponentDefinition],
        start: GridPos,
        end: GridPos,
    ) -> bool {
        if !is_orthogonal(start, end) || start == end {
            return false;
        }
        if self.segment_blocked(library, start, end) {
            return false;
        }
        let Some(edges) = wire_edges(&[start, end]) else {
            return false;
        };
        !edges.iter().any(|edge| self.edge_occupied(*edge, None))
            && !self.segment_crosses_wire(start, end, None)
    }

    /// Returns true when a point is a pin or an existing wire vertex.
    #[must_use]
    pub fn is_connection_point(&self, library: &[ComponentDefinition], point: GridPos) -> bool {
        self.is_connection_point_excluding(library, point, None)
    }

    pub(super) fn is_connection_point_excluding(
        &self,
        library: &[ComponentDefinition],
        point: GridPos,
        ignore: Option<WireId>,
    ) -> bool {
        self.has_pin_at(library, point) || self.has_wire_vertex(point, ignore)
    }

    /// Removes a wire.
    pub fn remove_wire(&mut self, id: WireId) {
        if let Some(wire) = self.wires.iter().find(|wire| wire.id == id) {
            self.wire_index.remove(wire);
        }
        self.wires.retain(|wire| wire.id != id);
    }

    /// Changes a wire net class.
    pub fn set_wire_class(&mut self, id: WireId, class: NetClass) -> bool {
        let Some(wire) = self.wires.iter_mut().find(|wire| wire.id == id) else {
            return false;
        };
        if wire.class == class {
            return false;
        }
        wire.class = class;
        true
    }

    /// Sets or clears a wire color override.
    pub fn set_wire_color(&mut self, id: WireId, color: Option<[u8; 3]>) -> bool {
        let Some(wire) = self.wires.iter_mut().find(|wire| wire.id == id) else {
            return false;
        };
        if wire.color_override == color {
            return false;
        }
        wire.color_override = color;
        true
    }

    /// Sets or clears a wire net label.
    pub fn set_wire_label(&mut self, id: WireId, label: Option<String>) -> bool {
        let Some(wire) = self.wires.iter_mut().find(|wire| wire.id == id) else {
            return false;
        };
        if wire.label == label {
            return false;
        }
        wire.label = label;
        true
    }

    /// Returns the nearest wire within a world-space tolerance.
    #[must_use]
    pub fn wire_at(&self, point: [f32; 2], tolerance: f32) -> Option<WireId> {
        let mut best: Option<(f32, WireId)> = None;
        for segment in self.wire_index.candidates(point) {
            let distance = segment_distance(point, segment.start, segment.end);
            if distance > tolerance {
                continue;
            }
            if best.is_none_or(|(current, _)| distance < current) {
                best = Some((distance, segment.wire));
            }
        }
        best.map(|(_, id)| id)
    }

    /// Returns components fully contained in a selection rectangle.
    #[must_use]
    pub fn components_in_rect(
        &self,
        library: &[ComponentDefinition],
        rect: GridRect,
    ) -> Vec<ComponentId> {
        self.components
            .iter()
            .filter_map(|instance| {
                let definition = find_definition(library, instance.definition)?;
                let bounds = instance.bounds(definition);
                let max = GridPos::new(bounds.max_x(), bounds.max_y());
                (rect.contains(bounds.min) && rect.contains(max)).then_some(instance.id)
            })
            .collect()
    }

    /// Returns wires fully contained in a selection rectangle.
    #[must_use]
    pub fn wires_in_rect(&self, rect: GridRect) -> Vec<WireId> {
        self.wires
            .iter()
            .filter(|wire| wire.points.iter().all(|point| rect.contains(*point)))
            .map(|wire| wire.id)
            .collect()
    }

    /// Returns all wires transitively connected to a wire.
    #[must_use]
    pub fn net_wires(&self, id: WireId) -> BTreeSet<WireId> {
        let mut net = BTreeSet::new();
        if !self.wires.iter().any(|wire| wire.id == id) {
            return net;
        }
        net.insert(id);
        while self.expand_net(&mut net) {}
        net
    }

    pub(super) fn expand_net(&self, net: &mut BTreeSet<WireId>) -> bool {
        let points = self.net_points(net);
        let before = net.len();
        for wire in &self.wires {
            if wire.points.iter().any(|point| points.contains(point)) {
                net.insert(wire.id);
            }
        }
        net.len() != before
    }

    pub(super) fn net_points(&self, net: &BTreeSet<WireId>) -> BTreeSet<GridPos> {
        self.wires
            .iter()
            .filter(|wire| net.contains(&wire.id))
            .flat_map(|wire| wire.points.iter().copied())
            .collect()
    }

    /// Returns a wire connected to a pin at a grid point.
    #[must_use]
    pub fn pin_wire_at(&self, library: &[ComponentDefinition], point: GridPos) -> Option<WireId> {
        let world = self.snap_pin(library, point)?;
        self.wires
            .iter()
            .find(|wire| wire.points.contains(&world))
            .map(|wire| wire.id)
    }

    /// Maps a visible pin dot or pin point to the model connection point.
    #[must_use]
    pub fn snap_pin(&self, library: &[ComponentDefinition], point: GridPos) -> Option<GridPos> {
        self.components.iter().find_map(|instance| {
            let definition = find_definition(library, instance.definition)?;
            definition.pins.iter().find_map(|pin| {
                let world = world_pin(instance, definition, pin);
                let direction = pin_direction(definition, pin, instance.rotation, instance.mirror);
                (point == world || point == world.offset(direction)).then_some(world)
            })
        })
    }

    /// Returns the outer bounds of all components and wires.
    #[must_use]
    pub fn bounds(&self, library: &[ComponentDefinition]) -> Option<(GridPos, GridPos)> {
        let mut min = None;
        let mut max = None;
        self.include_component_bounds(library, &mut min, &mut max);
        self.include_wire_bounds(&mut min, &mut max);
        Some((min?, max?))
    }

    pub(super) fn include_component_bounds(
        &self,
        library: &[ComponentDefinition],
        min: &mut Option<GridPos>,
        max: &mut Option<GridPos>,
    ) {
        for instance in &self.components {
            let Some(definition) = find_definition(library, instance.definition) else {
                continue;
            };
            let bounds = instance.bounds(definition);
            extend_bounds(min, max, bounds.min);
            extend_bounds(min, max, GridPos::new(bounds.max_x(), bounds.max_y()));
        }
    }

    pub(super) fn include_wire_bounds(&self, min: &mut Option<GridPos>, max: &mut Option<GridPos>) {
        for wire in &self.wires {
            for point in &wire.points {
                extend_bounds(min, max, *point);
            }
        }
    }

    /// Returns the nearest segment of one wire within a tolerance.
    #[must_use]
    pub fn wire_segment_at(&self, id: WireId, point: [f32; 2], tolerance: f32) -> Option<usize> {
        let mut best: Option<(f32, usize)> = None;
        for segment in self.wire_index.candidates(point) {
            if segment.wire != id {
                continue;
            }
            let distance = segment_distance(point, segment.start, segment.end);
            if distance > tolerance {
                continue;
            }
            if best.is_none_or(|(current, _)| distance < current) {
                best = Some((distance, segment.index));
            }
        }
        best.map(|(_, index)| index)
    }

    /// Returns grid points where three or more segments meet.
    #[must_use]
    pub fn junctions(&self) -> Vec<GridPos> {
        let mut counts: BTreeMap<GridPos, usize> = BTreeMap::new();
        for wire in &self.wires {
            for segment in wire.points.windows(2) {
                *counts.entry(segment[0]).or_default() += 1;
                *counts.entry(segment[1]).or_default() += 1;
            }
        }
        counts
            .into_iter()
            .filter_map(|(point, count)| (count >= 3).then_some(point))
            .collect()
    }
}
