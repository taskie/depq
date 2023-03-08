use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    hash::Hash,
};

#[derive(Debug, Clone)]
pub(crate) struct Edge<T: Clone>(pub(crate) T, pub(crate) T);

impl<T: Clone> Edge<T> {
    pub(crate) fn invert(&self) -> Self {
        Edge(self.1.clone(), self.0.clone())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Graph<T: Clone + Eq + Hash> {
    pub(crate) values: Vec<T>,
    pub(crate) value_to_index: HashMap<T, usize>,
    pub(crate) deps: HashMap<usize, Vec<usize>>,
}

impl<T: Clone + Eq + Hash> Default for Graph<T> {
    fn default() -> Self {
        Graph {
            values: Vec::new(),
            value_to_index: HashMap::new(),
            deps: HashMap::new(),
        }
    }
}

impl<T: Clone + Eq + Hash> FromIterator<Edge<T>> for Graph<T> {
    fn from_iter<I: IntoIterator<Item = Edge<T>>>(iter: I) -> Self {
        let mut graph = Graph::<T>::default();
        let mut seen: HashMap<T, usize> = HashMap::new();
        let mut values: Vec<T> = vec![];
        let mut f = |k: &T| match seen.entry(k.clone()) {
            Entry::Occupied(o) => *o.get(),
            Entry::Vacant(v) => {
                let i = values.len();
                v.insert(i);
                values.push(k.clone());
                i
            }
        };
        for t in iter {
            let from = f(&t.0);
            let to = f(&t.1);
            match graph.deps.entry(from) {
                Entry::Occupied(mut o) => o.get_mut().push(to),
                Entry::Vacant(v) => {
                    v.insert(vec![to]);
                }
            };
        }
        graph.values = values;
        graph.value_to_index = seen;
        graph
    }
}

impl<T: Clone + Eq + Hash> Graph<T> {
    pub(crate) fn find_roots(&self) -> Vec<usize> {
        let mut seen: HashSet<usize> = HashSet::new();
        for (_k, vs) in self.deps.iter() {
            seen.extend(vs.iter());
        }
        let root_set: HashSet<usize> = (0..self.values.len())
            .filter(|i| !seen.contains(i))
            .collect();
        let mut roots: Vec<usize> = root_set.into_iter().collect();
        roots.sort();
        roots
    }

    pub(crate) fn to_edges(&self) -> Vec<Edge<T>> {
        let mut edges = vec![];
        for (from, k) in self.values.iter().enumerate() {
            let Some(tos) = self.deps.get(&from) else { continue; };
            for to in tos {
                edges.push(Edge(k.clone(), self.values[*to].clone()))
            }
        }
        edges
    }

    pub(crate) fn remap(&self, values: Vec<T>) -> Graph<T> {
        let value_to_index: HashMap<T, usize> = values
            .iter()
            .enumerate()
            .map(|(i, v)| (v.clone(), i))
            .collect();
        let deps = self
            .deps
            .iter()
            .map(|(k, vs)| {
                (
                    *value_to_index.get(&self.values[*k]).unwrap(),
                    vs.iter()
                        .map(|v| *value_to_index.get(&self.values[*v]).unwrap())
                        .collect(),
                )
            })
            .collect();
        Graph {
            values,
            value_to_index,
            deps,
        }
    }

    pub(crate) fn invert(&self) -> Self {
        Self::from_iter(self.to_edges().iter().map(|e| e.invert()))
    }
}
