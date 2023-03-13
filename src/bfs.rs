use std::{collections::VecDeque, hash::Hash};

use crate::graph::Graph;

pub(crate) fn bfs<T: Clone + Eq + Ord + Hash, F: FnMut(usize, usize, Option<usize>) -> bool>(
    graph: &Graph<T>,
    is: &[usize],
    mut f: F,
) {
    let mut queue: VecDeque<(usize, usize, Option<usize>)> = is
        .iter()
        .filter(|i| **i < graph.values.len())
        .map(|i| (0, *i, None))
        .collect();
    loop {
        let Some(x) = queue.pop_front() else { break; };
        if !f(x.0, x.1, x.2) {
            continue;
        }
        let Some(nexts) = graph.deps.get(&x.1) else { continue; };
        queue.extend(nexts.iter().map(|next| (x.0 + 1, *next, Some(x.1))))
    }
}

pub(crate) fn bfs_path<T: Clone + Eq + Ord + Hash, F: FnMut(&[usize]) -> bool>(
    graph: &Graph<T>,
    is: &[usize],
    mut f: F,
) {
    let mut queue: VecDeque<(Vec<usize>, usize)> = is
        .iter()
        .filter(|i| **i < graph.values.len())
        .map(|i| (vec![], *i))
        .collect();
    loop {
        let Some(x) = queue.pop_front() else { break; };
        let mut new_path = x.0.clone();
        new_path.push(x.1);
        if !f(new_path.as_slice()) {
            continue;
        }
        let Some(nexts) = graph.deps.get(&x.1) else { continue; };
        queue.extend(nexts.iter().map(|next| (new_path.clone(), *next)))
    }
}
