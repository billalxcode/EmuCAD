use super::*;

impl Document {
    pub fn component(&self, id: ComponentId) -> Option<&ComponentInstance> {
        self.components.iter().find(|component| component.id == id)
    }

    /// Finds one component by ID for editing.
    pub fn component_mut(&mut self, id: ComponentId) -> Option<&mut ComponentInstance> {
        self.components
            .iter_mut()
            .find(|component| component.id == id)
    }

    /// Adds a component and generates its reference label.
    pub fn add(
        &mut self,
        definition: &ComponentDefinition,
        position: GridPos,
        rotation: Rotation,
        mirror: Mirror,
    ) -> ComponentId {
        self.add_with_value(
            definition,
            position,
            rotation,
            mirror,
            definition.default_value,
        )
    }

    /// Adds a component with an explicit value and a generated label.
    pub fn add_with_value(
        &mut self,
        definition: &ComponentDefinition,
        position: GridPos,
        rotation: Rotation,
        mirror: Mirror,
        value: &str,
    ) -> ComponentId {
        let id = self.allocate_id();
        let label = format!(
            "{}{}",
            definition.reference_prefix,
            self.next_reference(definition.reference_prefix)
        );
        let mut instance = new_instance(id, definition, position, rotation, mirror, label);
        instance.value = value.to_string();
        self.components.push(instance);
        id
    }

    /// Removes a component.
    pub fn remove(&mut self, id: ComponentId) {
        self.components.retain(|component| component.id != id);
    }

    /// Tests whether a definition can be placed without overlap.
    #[must_use]
    pub fn can_place(
        &self,
        library: &[ComponentDefinition],
        definition: &ComponentDefinition,
        position: GridPos,
        rotation: Rotation,
        _mirror: Mirror,
    ) -> bool {
        !self.overlaps(library, definition, position, rotation)
    }

    /// Moves several components atomically when every target is free.
    pub fn move_components(
        &mut self,
        library: &[ComponentDefinition],
        moves: &[(ComponentId, GridPos)],
    ) -> bool {
        let ids: BTreeSet<_> = moves.iter().map(|(id, _)| *id).collect();
        if ids.len() != moves.len() {
            return false;
        }
        let mut candidate = self.clone();
        if !candidate.apply_moves(moves) {
            return false;
        }
        if !ids.iter().all(|id| candidate.position_valid(library, *id)) {
            return false;
        }
        *self = candidate;
        true
    }

    pub(super) fn apply_moves(&mut self, moves: &[(ComponentId, GridPos)]) -> bool {
        for &(id, position) in moves {
            let Some(instance) = self.component_mut(id) else {
                return false;
            };
            instance.position = position;
        }
        true
    }

    pub(super) fn position_valid(&self, library: &[ComponentDefinition], id: ComponentId) -> bool {
        let Some(instance) = self.component(id) else {
            return false;
        };
        let Some(definition) = find_definition(library, instance.definition) else {
            return false;
        };
        let bounds = instance.bounds(definition);
        self.components
            .iter()
            .filter(|other| other.id != id)
            .all(|other| {
                let Some(other_definition) = find_definition(library, other.definition) else {
                    return true;
                };
                !bounds.intersects(other.bounds(other_definition))
            })
    }

    /// Rotates several components atomically when every result is free.
    pub fn rotate_components(
        &mut self,
        library: &[ComponentDefinition],
        ids: &[ComponentId],
    ) -> bool {
        self.update_transforms(library, ids, |instance| {
            instance.rotation = instance.rotation.next();
        })
    }

    /// Mirrors several components atomically when every result is free.
    pub fn mirror_components(
        &mut self,
        library: &[ComponentDefinition],
        ids: &[ComponentId],
    ) -> bool {
        self.update_transforms(library, ids, |instance| {
            instance.mirror = instance.mirror.next();
        })
    }

    pub(super) fn update_transforms(
        &mut self,
        library: &[ComponentDefinition],
        ids: &[ComponentId],
        update: impl Fn(&mut ComponentInstance),
    ) -> bool {
        let unique: BTreeSet<_> = ids.iter().copied().collect();
        if unique.len() != ids.len() {
            return false;
        }
        let Some(candidate) = self.transformed_candidate(library, ids, &unique, update) else {
            return false;
        };
        *self = candidate;
        true
    }

    pub(super) fn transformed_candidate(
        &self,
        library: &[ComponentDefinition],
        ids: &[ComponentId],
        unique: &BTreeSet<ComponentId>,
        update: impl Fn(&mut ComponentInstance),
    ) -> Option<Document> {
        let mut candidate = self.clone();
        if !candidate.apply_transforms(ids, update) {
            return None;
        }
        if !unique
            .iter()
            .all(|id| candidate.position_valid(library, *id))
        {
            return None;
        }
        Some(candidate)
    }

    pub(super) fn apply_transforms(
        &mut self,
        ids: &[ComponentId],
        update: impl Fn(&mut ComponentInstance),
    ) -> bool {
        for &id in ids {
            let Some(instance) = self.component_mut(id) else {
                return false;
            };
            update(instance);
        }
        true
    }

    /// Returns true when a wire endpoint touches one of the component pins.
    #[must_use]
    pub fn component_has_wire(&self, library: &[ComponentDefinition], id: ComponentId) -> bool {
        let Some(instance) = self.component(id) else {
            return false;
        };
        let Some(definition) = find_definition(library, instance.definition) else {
            return false;
        };
        definition.pins.iter().any(|pin| {
            let point = world_pin(instance, definition, pin);
            self.wires.iter().any(|wire| wire.points.contains(&point))
        })
    }

    /// Returns the topmost component at a grid point.
    #[must_use]
    pub fn hit_test(&self, library: &[ComponentDefinition], point: GridPos) -> Option<ComponentId> {
        self.components.iter().rev().find_map(|instance| {
            let definition = find_definition(library, instance.definition)?;
            instance
                .bounds(definition)
                .contains(point)
                .then_some(instance.id)
        })
    }

    pub(super) fn overlaps(
        &self,
        library: &[ComponentDefinition],
        definition: &ComponentDefinition,
        position: GridPos,
        rotation: Rotation,
    ) -> bool {
        let candidate = GridRect::new(position, body_size(definition, rotation));
        self.components.iter().any(|instance| {
            let Some(existing) = find_definition(library, instance.definition) else {
                return false;
            };
            candidate.intersects(instance.bounds(existing))
        })
    }

    pub(super) fn allocate_id(&mut self) -> ComponentId {
        let id = ComponentId(self.next_id);
        self.next_id += 1;
        id
    }

    pub(super) fn validate_wire(
        &self,
        library: &[ComponentDefinition],
        points: &[GridPos],
        ignore: Option<WireId>,
    ) -> bool {
        if !self.valid_wire_ends(library, points, ignore) {
            return false;
        }
        let Some(edges) = wire_edges(points) else {
            return false;
        };
        if !self.free_edges(&edges, ignore) {
            return false;
        }
        if self.crosses_wire(points, ignore) {
            return false;
        }
        !self.path_blocked(library, points)
    }

    pub(super) fn valid_wire_ends(
        &self,
        library: &[ComponentDefinition],
        points: &[GridPos],
        ignore: Option<WireId>,
    ) -> bool {
        let (Some(&first), Some(&last)) = (points.first(), points.last()) else {
            return false;
        };
        points.len() >= 2
            && self.is_connection_point_excluding(library, first, ignore)
            && self.is_connection_point_excluding(library, last, ignore)
    }

    pub(super) fn free_edges(&self, edges: &BTreeSet<GridEdge>, ignore: Option<WireId>) -> bool {
        !edges.iter().any(|edge| self.edge_occupied(*edge, ignore))
    }

    pub(super) fn crosses_wire(&self, points: &[GridPos], ignore: Option<WireId>) -> bool {
        points
            .windows(2)
            .any(|segment| self.segment_crosses_wire(segment[0], segment[1], ignore))
    }

    pub(super) fn edge_occupied(&self, edge: GridEdge, ignore: Option<WireId>) -> bool {
        self.wires.iter().any(|wire| {
            if Some(wire.id) == ignore {
                return false;
            }
            wire.points
                .windows(2)
                .any(|segment| segment_edges(segment[0], segment[1]).contains(&edge))
        })
    }

    pub(super) fn segment_crosses_wire(
        &self,
        start: GridPos,
        end: GridPos,
        ignore: Option<WireId>,
    ) -> bool {
        self.wires.iter().any(|wire| {
            if Some(wire.id) == ignore {
                return false;
            }
            wire.points
                .windows(2)
                .any(|segment| crossing_point(start, end, segment[0], segment[1]).is_some())
        })
    }

    pub(super) fn path_blocked(&self, library: &[ComponentDefinition], points: &[GridPos]) -> bool {
        points
            .windows(2)
            .any(|segment| self.segment_blocked(library, segment[0], segment[1]))
    }

    pub(super) fn segment_blocked(
        &self,
        library: &[ComponentDefinition],
        start: GridPos,
        end: GridPos,
    ) -> bool {
        let length = segment_length(start, end);
        (1..length).any(|step| self.point_in_body(library, segment_step(start, end, step)))
    }

    pub(super) fn point_in_body(&self, library: &[ComponentDefinition], point: GridPos) -> bool {
        self.components.iter().any(|instance| {
            let Some(definition) = find_definition(library, instance.definition) else {
                return false;
            };
            instance.bounds(definition).strictly_contains(point)
        })
    }

    pub(super) fn has_pin_at(&self, library: &[ComponentDefinition], point: GridPos) -> bool {
        self.components.iter().any(|instance| {
            let Some(definition) = find_definition(library, instance.definition) else {
                return false;
            };
            definition
                .pins
                .iter()
                .any(|pin| world_pin(instance, definition, pin) == point)
        })
    }

    pub(super) fn has_wire_vertex(&self, point: GridPos, ignore: Option<WireId>) -> bool {
        self.wires
            .iter()
            .any(|wire| Some(wire.id) != ignore && wire.points.contains(&point))
    }

    pub(super) fn next_reference(&self, prefix: &str) -> usize {
        self.components
            .iter()
            .filter(|component| component.label.starts_with(prefix))
            .count()
            + 1
    }
}
