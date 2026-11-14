use super::*;
use std::collections::VecDeque;

pub(super) struct Layout {
    pub order: Vec<usize>,
    pub cyclic: bool,
}

pub(super) fn component_order(gates: &[usize], all: &[Gate], nets: usize) -> Layout {
    let (_producer, mut indegree, edges) = graph(gates, all, nets);
    let order = topological(gates, &edges, &mut indegree);
    let cyclic = order.len() != gates.len();
    let order = if cyclic { gates.to_vec() } else { order };
    Layout { order, cyclic }
}

fn graph(
    gates: &[usize],
    all: &[Gate],
    nets: usize,
) -> (Vec<Option<usize>>, Vec<usize>, Vec<Vec<usize>>) {
    let producer = producers(gates, all, nets);
    let (indegree, edges) = dependency_edges(gates, all, &producer);
    (producer, indegree, edges)
}

fn producers(gates: &[usize], all: &[Gate], nets: usize) -> Vec<Option<usize>> {
    let mut producer = vec![None; nets];
    for &gate in gates {
        producer[all[gate].output().raw()] = Some(gate);
    }
    producer
}

fn dependency_edges(
    gates: &[usize],
    all: &[Gate],
    producer: &[Option<usize>],
) -> (Vec<usize>, Vec<Vec<usize>>) {
    let mut indegree = vec![0usize; all.len()];
    let mut edges = vec![Vec::new(); all.len()];
    for &gate in gates {
        for input in all[gate].inputs() {
            let Some(source) = producer[input.raw()] else {
                continue;
            };
            indegree[gate] += 1;
            edges[source].push(gate);
        }
    }
    (indegree, edges)
}

fn topological(gates: &[usize], edges: &[Vec<usize>], indegree: &mut [usize]) -> Vec<usize> {
    let mut queue: VecDeque<usize> = gates
        .iter()
        .copied()
        .filter(|gate| indegree[*gate] == 0)
        .collect();
    let mut order = Vec::with_capacity(gates.len());
    while let Some(gate) = queue.pop_front() {
        order.push(gate);
        for &next in &edges[gate] {
            indegree[next] -= 1;
            if indegree[next] == 0 {
                queue.push_back(next);
            }
        }
    }
    order
}
