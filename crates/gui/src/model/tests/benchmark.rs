use super::*;

#[test]
#[ignore = "performance benchmark"]
fn benchmark_large_document() {
    let library = library::builtin();
    let started = std::time::Instant::now();
    let document = large_document();
    run_large_queries(&document, library);
    println!("large document benchmark: {:?}", started.elapsed());
}

fn large_document() -> Document {
    let resistor = crate::library::find("resistor").expect("resistor");
    let mut source = Document::new();
    for row in 0..20 {
        for column in 0..25 {
            let position = GridPos::new(column * 4, row * 3);
            source.add(resistor, position, Rotation::Deg0, Mirror::None);
        }
    }
    let wires = (0..2000).map(benchmark_wire).collect();
    Document::restore(source.components().to_vec(), wires)
}

fn benchmark_wire(index: usize) -> Wire {
    let start = GridPos::new(((index % 10) * 4) as i32, ((index / 10) * 2) as i32);
    let end = start.offset(GridPos::new(2, 0));
    test_wire(index as u32, start, end, NetClass::Digital)
}

fn run_large_queries(document: &Document, library: &[ComponentDefinition]) {
    for index in 0..1000 {
        let point = [(index % 40) as f32, (index / 40) as f32 * 2.0];
        let _ = document.wire_at(point, 0.35);
    }
    let rect = super::GridRect::new(GridPos::new(0, 0), GridSize::new(50, 30));
    let _ = document.components_in_rect(library, rect);
}
