use super::{AnalogHandle, Event};
use crate::library;
use crate::model::{Document, GridPos, Mirror, NetClass, Rotation};
use crate::transient::TransientSettings;

pub(crate) fn test_document() -> Document {
    let mut document = Document::default();
    let resistor = library::find("passive.resistor.ideal_resistor").expect("resistor");
    let ground = library::find("gnd").expect("gnd");
    document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    document.add(ground, GridPos::new(0, 2), Rotation::Deg0, Mirror::None);
    add_wire(&mut document, &[(0, 1), (0, 2), (1, 2)]);
    add_wire(&mut document, &[(4, 1), (4, -2), (6, -2), (6, 2), (1, 2)]);
    document
}

fn add_wire(document: &mut Document, points: &[(i32, i32)]) {
    let points: Vec<_> = points.iter().map(|&(x, y)| GridPos::new(x, y)).collect();
    let id = document.add_wire(library::builtin(), points.clone(), NetClass::Power);
    assert!(id.is_some(), "wire {points:?}");
    id.expect("wire");
}

#[test]
fn dc_worker_returns_result() {
    let mut worker = AnalogHandle::spawn();
    let request = worker.request_dc(test_document());
    let Event::Dc {
        request: completed,
        result,
    } = worker.wait()
    else {
        panic!("unexpected event");
    };
    assert_eq!(completed, request);
    let result = result.expect("dc");
    assert!(!result.voltages.is_empty());
    assert!(result.iterations > 0);
}

#[test]
fn transient_worker_returns_trace() {
    let mut worker = AnalogHandle::spawn();
    let settings = TransientSettings {
        dt: 1e-6,
        steps: 2,
        use_dc_op: true,
        ..TransientSettings::default()
    };
    let request = worker.request_transient(test_document(), settings);
    let Event::Transient {
        request: completed,
        result,
    } = worker.wait()
    else {
        panic!("unexpected event");
    };
    assert_eq!(completed, request);
    let trace = result.expect("trace");
    assert_eq!(trace.values.len(), 2);
}
