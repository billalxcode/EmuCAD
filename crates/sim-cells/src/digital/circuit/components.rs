use super::topology::component_order;
use super::*;

pub(super) struct CircuitComponents {
    pub(super) gates: Vec<Vec<usize>>,
    pub(super) orders: Vec<Vec<usize>>,
    pub(super) cyclic: Vec<bool>,
    pub(super) fanout: Vec<Vec<usize>>,
    pub(super) gate_component: Vec<usize>,
    pub(super) gate_rank: Vec<u32>,
}

struct UnionFind {
    parent: Vec<usize>,
}

impl UnionFind {
    fn new(size: usize) -> Self {
        Self {
            parent: (0..size).collect(),
        }
    }

    fn find(&mut self, mut index: usize) -> usize {
        while self.parent[index] != index {
            self.parent[index] = self.parent[self.parent[index]];
            index = self.parent[index];
        }
        index
    }

    fn union(&mut self, first: usize, second: usize) {
        let first = self.find(first);
        let second = self.find(second);
        if first != second {
            self.parent[second] = first;
        }
    }
}

pub(super) fn build_components(values: usize, gates: &[Gate], dffs: &[Dff]) -> CircuitComponents {
    let mut union = UnionFind::new(values);
    for &gate in gates {
        union_gate(&mut union, gate);
    }
    for dff in dffs {
        union.union(dff.data.raw(), dff.output.raw());
    }
    let component_gates = assign_gate_components(&mut union, gates);
    component_data(component_gates, gates, values)
}

fn component_data(
    component_gates: Vec<Vec<usize>>,
    gates: &[Gate],
    values: usize,
) -> CircuitComponents {
    let (orders, cyclic) = component_layouts(&component_gates, gates, values);
    let fanout = build_fanout(gates, values);
    let gate_component = map_gate_components(&component_gates, gates.len());
    let gate_rank = rank_gates(&orders, gates.len());
    CircuitComponents {
        gates: component_gates,
        orders,
        cyclic,
        fanout,
        gate_component,
        gate_rank,
    }
}

fn rank_gates(orders: &[Vec<usize>], gates: usize) -> Vec<u32> {
    let mut ranks = vec![0u32; gates];
    for order in orders {
        for (rank, &gate) in order.iter().enumerate() {
            ranks[gate] = rank as u32;
        }
    }
    ranks
}

fn build_fanout(gates: &[Gate], values: usize) -> Vec<Vec<usize>> {
    let mut fanout = vec![Vec::new(); values];
    for (index, &gate) in gates.iter().enumerate() {
        for input in gate.inputs() {
            fanout[input.raw()].push(index);
        }
    }
    fanout
}

fn map_gate_components(components: &[Vec<usize>], gates: usize) -> Vec<usize> {
    let mut map = vec![0usize; gates];
    for (component, gate_list) in components.iter().enumerate() {
        for &gate in gate_list {
            map[gate] = component;
        }
    }
    map
}

fn component_layouts(
    components: &[Vec<usize>],
    gates: &[Gate],
    values: usize,
) -> (Vec<Vec<usize>>, Vec<bool>) {
    let mut orders = Vec::with_capacity(components.len());
    let mut cyclic = Vec::with_capacity(components.len());
    for component in components {
        let layout = component_order(component, gates, values);
        orders.push(layout.order);
        cyclic.push(layout.cyclic);
    }
    (orders, cyclic)
}

fn assign_gate_components(union: &mut UnionFind, gates: &[Gate]) -> Vec<Vec<usize>> {
    let mut roots = BTreeMap::new();
    let mut components: Vec<Vec<usize>> = Vec::new();
    for (index, &gate) in gates.iter().enumerate() {
        let root = union.find(gate.output().raw());
        let component = *roots.entry(root).or_insert_with(|| {
            components.push(Vec::new());
            components.len() - 1
        });
        components[component].push(index);
    }
    components
}

fn union_gate(union: &mut UnionFind, gate: Gate) {
    let nets = gate_nets(gate);
    union_nets(union, &nets);
}

fn gate_nets(gate: Gate) -> Vec<NetId> {
    match gate {
        Gate::Const(_, _) => Vec::new(),
        Gate::Buf(first, output) | Gate::Not(first, output) => vec![first, output],
        Gate::And(first, second, output)
        | Gate::Or(first, second, output)
        | Gate::Xor(first, second, output)
        | Gate::Nand(first, second, output)
        | Gate::Nor(first, second, output) => vec![first, second, output],
        Gate::Mux {
            first,
            second,
            select,
            output,
        } => vec![first, second, select, output],
    }
}

fn union_nets(union: &mut UnionFind, nets: &[NetId]) {
    for pair in nets.windows(2) {
        union.union(pair[0].raw(), pair[1].raw());
    }
}
