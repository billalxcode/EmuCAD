use super::*;

pub(super) fn new_instance(
    id: ComponentId,
    definition: &ComponentDefinition,
    position: GridPos,
    rotation: Rotation,
    mirror: Mirror,
    label: String,
) -> ComponentInstance {
    ComponentInstance {
        id,
        definition: definition.id,
        position,
        rotation,
        mirror,
        label,
        value: definition.default_value.to_string(),
        parameters: BTreeMap::new(),
    }
}

/// Returns the body size after rotation.
#[must_use]
pub fn body_size(definition: &ComponentDefinition, rotation: Rotation) -> GridSize {
    match rotation {
        Rotation::Deg0 | Rotation::Deg180 => definition.size,
        Rotation::Deg90 | Rotation::Deg270 => definition.size.swapped(),
    }
}

/// Returns a pin position in world grid coordinates.
#[must_use]
pub fn world_pin(
    instance: &ComponentInstance,
    definition: &ComponentDefinition,
    pin: &PinDefinition,
) -> GridPos {
    instance.position.offset(transform_point(
        definition,
        pin.offset,
        instance.rotation,
        instance.mirror,
    ))
}

/// Returns the outward direction of a pin in world grid units.
#[must_use]
pub fn pin_direction(
    definition: &ComponentDefinition,
    pin: &PinDefinition,
    rotation: Rotation,
    mirror: Mirror,
) -> GridPos {
    let delta = outward_delta(definition, pin.offset);
    let mirrored = mirror.apply(delta);
    rotation.apply(mirrored)
}

pub(super) fn outward_delta(definition: &ComponentDefinition, offset: GridPos) -> GridPos {
    if offset.x == 0 {
        GridPos::new(-1, 0)
    } else if offset.x == definition.size.width {
        GridPos::new(1, 0)
    } else if offset.y == 0 {
        GridPos::new(0, -1)
    } else {
        GridPos::new(0, 1)
    }
}

pub(super) fn transform_point(
    definition: &ComponentDefinition,
    point: GridPos,
    rotation: Rotation,
    mirror: Mirror,
) -> GridPos {
    let min = transform_min(definition, rotation, mirror);
    let raw = transform_raw(definition, point, rotation, mirror);
    raw.subtract(min)
}

pub(super) fn transform_min(
    definition: &ComponentDefinition,
    rotation: Rotation,
    mirror: Mirror,
) -> GridPos {
    let width = definition.size.width;
    let height = definition.size.height;
    let corners = [
        GridPos::new(0, 0),
        GridPos::new(width, 0),
        GridPos::new(0, height),
        GridPos::new(width, height),
    ];
    corners
        .into_iter()
        .map(|corner| transform_raw(definition, corner, rotation, mirror))
        .reduce(min_grid)
        .unwrap_or_default()
}

fn min_grid(first: GridPos, second: GridPos) -> GridPos {
    GridPos::new(first.x.min(second.x), first.y.min(second.y))
}

pub(super) fn transform_raw(
    definition: &ComponentDefinition,
    point: GridPos,
    rotation: Rotation,
    mirror: Mirror,
) -> GridPos {
    let center = GridPos::new(definition.size.width / 2, definition.size.height / 2);
    let delta = point.subtract(center);
    let mirrored = mirror.apply(delta);
    let rotated = rotation.apply(mirrored);
    center.offset(rotated)
}

/// Looks up a definition by ID.
#[must_use]
pub fn find_definition<'a>(
    library: &'a [ComponentDefinition],
    id: &str,
) -> Option<&'a ComponentDefinition> {
    library.iter().find(|definition| definition.id == id)
}

pub(super) fn is_orthogonal(start: GridPos, end: GridPos) -> bool {
    start.x == end.x || start.y == end.y
}

pub(super) fn wire_edges(points: &[GridPos]) -> Option<BTreeSet<GridEdge>> {
    if points.len() < 2 {
        return None;
    }
    let mut edges = BTreeSet::new();
    for segment in points.windows(2) {
        let start = segment[0];
        let end = segment[1];
        if start == end || !is_orthogonal(start, end) {
            return None;
        }
        for edge in segment_edges(start, end) {
            if !edges.insert(edge) {
                return None;
            }
        }
    }
    Some(edges)
}

pub(super) fn segment_edges(start: GridPos, end: GridPos) -> Vec<GridEdge> {
    let length = segment_length(start, end);
    (0..length)
        .map(|step| {
            GridEdge::new(
                segment_step(start, end, step),
                segment_step(start, end, step + 1),
            )
        })
        .collect()
}

pub(crate) fn extend_bounds(min: &mut Option<GridPos>, max: &mut Option<GridPos>, point: GridPos) {
    let low = min.get_or_insert(point);
    low.x = low.x.min(point.x);
    low.y = low.y.min(point.y);
    let high = max.get_or_insert(point);
    high.x = high.x.max(point.x);
    high.y = high.y.max(point.y);
}

pub(super) fn crossing_point(
    first: GridPos,
    second: GridPos,
    other_first: GridPos,
    other_second: GridPos,
) -> Option<GridPos> {
    let point = crossing_candidate(first, second, other_first, other_second)?;
    if !on_segment(first, second, point) || !on_segment(other_first, other_second, point) {
        return None;
    }
    let first_endpoint = point == first || point == second;
    let second_endpoint = point == other_first || point == other_second;
    if first_endpoint && second_endpoint {
        return None;
    }
    Some(point)
}

pub(super) fn crossing_candidate(
    first: GridPos,
    second: GridPos,
    other_first: GridPos,
    other_second: GridPos,
) -> Option<GridPos> {
    if first.y == second.y && other_first.x == other_second.x {
        return Some(GridPos::new(other_first.x, first.y));
    }
    if first.x == second.x && other_first.y == other_second.y {
        return Some(GridPos::new(first.x, other_first.y));
    }
    None
}

pub(super) fn on_segment(start: GridPos, end: GridPos, point: GridPos) -> bool {
    point.x >= start.x.min(end.x)
        && point.x <= start.x.max(end.x)
        && point.y >= start.y.min(end.y)
        && point.y <= start.y.max(end.y)
}

pub(super) fn segment_distance(point: [f32; 2], start: GridPos, end: GridPos) -> f32 {
    let x = point[0];
    let y = point[1];
    if start.y == end.y {
        let min = start.x.min(end.x) as f32;
        let max = start.x.max(end.x) as f32;
        let nearest = x.clamp(min, max);
        return ((x - nearest).powi(2) + (y - start.y as f32).powi(2)).sqrt();
    }
    let min = start.y.min(end.y) as f32;
    let max = start.y.max(end.y) as f32;
    let nearest = y.clamp(min, max);
    ((x - start.x as f32).powi(2) + (y - nearest).powi(2)).sqrt()
}

pub(super) fn segment_length(start: GridPos, end: GridPos) -> i32 {
    (end.x - start.x).abs() + (end.y - start.y).abs()
}

pub(super) fn segment_step(start: GridPos, end: GridPos, step: i32) -> GridPos {
    let dx = (end.x - start.x).signum();
    let dy = (end.y - start.y).signum();
    GridPos::new(start.x + dx * step, start.y + dy * step)
}
