use std::fmt::Debug;
use std::{collections::HashSet, hash::Hash};

use crate::graph::Graph;

pub(crate) fn tsort<T: Clone + Debug + Eq + Ord + Hash, F: FnMut(usize)>(
    graph: &Graph<T>,
    mut f: F,
) -> Result<(), HashSet<usize>> {
    eprintln!("values: {:?}", graph.values);
    let mut deps = graph.deps.clone();
    let mut rdeps = graph.invert().remap(graph.values.clone()).deps;
    let mut stack = graph.find_roots();
    stack.reverse();
    stack.retain(|v| !rdeps.contains_key(v));
    loop {
        let Some(n) = stack.pop() else { break; };
        f(n);
        let Some(ts) = deps.remove(&n) else { continue; };
        for t in ts.iter().rev() {
            let Some(fs) = rdeps.get_mut(t) else { continue; };
            fs.retain(|&v| v != n);
            if fs.is_empty() {
                rdeps.remove(t);
                stack.push(*t);
            }
        }
    }
    if deps.is_empty() {
        Ok(())
    } else {
        Err(deps.keys().copied().collect())
    }
}
