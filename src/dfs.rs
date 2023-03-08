use std::hash::Hash;

use crate::graph::Graph;

pub(crate) fn dfs<T: Clone + Eq + Hash, F: FnMut(usize, usize, Option<usize>) -> bool>(
    graph: &Graph<T>,
    is: &[usize],
    mut f: F,
) {
    let mut stack: Vec<(usize, usize, Option<usize>)> = is
        .iter()
        .filter(|i| **i < graph.values.len())
        .map(|i| (0, *i, None))
        .collect();
    stack.reverse();
    loop {
        let Some(x) = stack.pop() else { break; };
        if !f(x.0, x.1, x.2) {
            continue;
        }
        let Some(nexts) = graph.deps.get(&x.1) else { continue; };
        stack.extend(nexts.iter().map(|next| (x.0 + 1, *next, Some(x.1))).rev())
    }
}
