use crate::model::{GridPos, GridSize, PinDefinition, PinDirection};

/// Returns the grid size and pin anchors for one base symbol.
pub(crate) fn layout(base: &str, count: u8) -> (GridSize, Vec<PinDefinition>) {
    match (base, count) {
        ("button" | "switch", 2) => two(
            4,
            4,
            (0, 2, PinDirection::Left),
            (4, 2, PinDirection::Right),
        ),
        ("switch_spdt", 3) => three(
            4,
            4,
            (0, 2, PinDirection::Left),
            (4, 1, PinDirection::Right),
            (4, 3, PinDirection::Right),
        ),
        _ => generic(count),
    }
}

fn two(
    width: i32,
    height: i32,
    first: (i32, i32, PinDirection),
    second: (i32, i32, PinDirection),
) -> (GridSize, Vec<PinDefinition>) {
    (
        GridSize::new(width, height),
        vec![
            pin(1, first.0, first.1, first.2),
            pin(2, second.0, second.1, second.2),
        ],
    )
}

fn three(
    width: i32,
    height: i32,
    first: (i32, i32, PinDirection),
    second: (i32, i32, PinDirection),
    third: (i32, i32, PinDirection),
) -> (GridSize, Vec<PinDefinition>) {
    (
        GridSize::new(width, height),
        vec![
            pin(1, first.0, first.1, first.2),
            pin(2, second.0, second.1, second.2),
            pin(3, third.0, third.1, third.2),
        ],
    )
}

fn generic(count: u8) -> (GridSize, Vec<PinDefinition>) {
    match count {
        1 => (GridSize::new(2, 2), vec![pin(1, 1, 0, PinDirection::Up)]),
        2 => two(
            4,
            2,
            (0, 1, PinDirection::Left),
            (4, 1, PinDirection::Right),
        ),
        3 => three(
            4,
            4,
            (0, 1, PinDirection::Left),
            (4, 2, PinDirection::Right),
            (0, 3, PinDirection::Left),
        ),
        4 => four(),
        _ => grid(count),
    }
}

fn four() -> (GridSize, Vec<PinDefinition>) {
    (
        GridSize::new(4, 4),
        vec![
            pin(1, 0, 1, PinDirection::Left),
            pin(2, 4, 1, PinDirection::Right),
            pin(3, 0, 3, PinDirection::Left),
            pin(4, 4, 3, PinDirection::Right),
        ],
    )
}

fn grid(count: u8) -> (GridSize, Vec<PinDefinition>) {
    let left = usize::from(count).div_ceil(2);
    let right = usize::from(count) / 2;
    let height = (left * 2 + 2) as i32;
    let mut pins = Vec::with_capacity(usize::from(count));
    for row in 0..left {
        let index = (row + 1) as u8;
        let y = (row * 2 + 2) as i32;
        pins.push(pin(index, 0, y, PinDirection::Left));
    }
    for row in 0..right {
        let index = (left + row + 1) as u8;
        let y = (row * 2 + 2) as i32;
        pins.push(pin(index, 8, y, PinDirection::Right));
    }
    (GridSize::new(8, height), pins)
}

fn pin(index: u8, x: i32, y: i32, direction: PinDirection) -> PinDefinition {
    let label = pin_label(index);
    PinDefinition {
        name: label,
        number: label,
        offset: GridPos::new(x, y),
        direction,
    }
}

fn pin_label(index: u8) -> &'static str {
    const LABELS: [&str; 8] = ["1", "2", "3", "4", "5", "6", "7", "8"];
    LABELS
        .get(usize::from(index).saturating_sub(1))
        .copied()
        .unwrap_or_else(|| Box::leak(index.to_string().into_boxed_str()))
}

#[cfg(test)]
mod tests {
    use super::layout;
    use crate::model::{GridPos, PinDirection};

    #[test]
    fn spdt_layout_matches_svg_sides() {
        let (size, pins) = layout("switch_spdt", 3);
        assert_eq!((size.width, size.height), (4, 4));
        assert_eq!(pins[0].offset, GridPos::new(0, 2));
        assert_eq!(pins[1].offset, GridPos::new(4, 1));
        assert_eq!(pins[2].offset, GridPos::new(4, 3));
        assert_eq!(pins[0].direction, PinDirection::Left);
        assert_eq!(pins[1].direction, PinDirection::Right);
        assert_eq!(pins[2].direction, PinDirection::Right);
    }
}
