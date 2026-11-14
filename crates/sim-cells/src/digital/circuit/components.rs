use super::*;

pub(super) struct CircuitComponents {
    pub(super) gates: Vec<Vec<usize>>,
    pub(super) nets: Vec<Option<usize>>,
    pub(super) dffs: Vec<Option<usize>>,
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
    let (component_gates, roots) = assign_gate_components(&mut union, gates);
    let nets = assign_net_components(&mut union, &roots);
    let dffs = dffs.iter().map(|dff| nets[dff.output.raw()]).collect();
    CircuitComponents {
        gates: component_gates,
        nets,
        dffs,
    }
}

fn assign_gate_components(
    union: &mut UnionFind,
    gates: &[Gate],
) -> (Vec<Vec<usize>>, BTreeMap<usize, usize>) {
    let mut roots = BTreeMap::new();
    let mut components: Vec<Vec<usize>> = Vec::new();
    for (index, &gate) in gates.iter().enumerate() {
        let root = union.find(gate_output(gate).raw());
        let component = *roots.entry(root).or_insert_with(|| {
            components.push(Vec::new());
            components.len() - 1
        });
        components[component].push(index);
    }
    (components, roots)
}

fn assign_net_components(
    union: &mut UnionFind,
    roots: &BTreeMap<usize, usize>,
) -> Vec<Option<usize>> {
    (0..union.parent.len())
        .map(|net| roots.get(&union.find(net)).copied())
        .collect()
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

pub(super) fn gate_output(gate: Gate) -> NetId {
    match gate {
        Gate::Const(_, output)
        | Gate::Buf(_, output)
        | Gate::Not(_, output)
        | Gate::And(_, _, output)
        | Gate::Or(_, _, output)
        | Gate::Xor(_, _, output)
        | Gate::Nand(_, _, output)
        | Gate::Nor(_, _, output) => output,
        Gate::Mux { output, .. } => output,
    }
}
