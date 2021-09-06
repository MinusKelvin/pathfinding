use crate::Edge;

pub struct DirectedGraph<V> {
    vertices: Vec<Vertex<V>>,
    edges: usize,
}

#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
struct Vertex<V> {
    incoming: Vec<Edge<usize>>,
    outgoing: Vec<Edge<usize>>,
    data: V,
}

impl<V> DirectedGraph<V> {
    pub fn new() -> Self {
        DirectedGraph {
            edges: 0,
            vertices: vec![],
        }
    }

    pub fn add_vertex(&mut self, data: V) -> usize {
        let id = self.vertices.len();
        self.vertices.push(Vertex {
            incoming: vec![],
            outgoing: vec![],
            data,
        });
        id
    }

    /// note: if edge is not present, runtime is linear in the number of edges on the relevant
    ///       vertices. if edge is present, runtime is logarithmic in the number of edges on the
    ///       relevant vertices.
    pub fn add_edge(&mut self, from: usize, to: usize, cost: f64) {
        assert!(
            from < self.vertices.len() && to < self.vertices.len(),
            "from and to vertices must exist"
        );

        let outgoing = &mut self.vertices[from].outgoing;
        match outgoing.binary_search_by_key(&to, |e| e.destination) {
            Ok(i) => outgoing[i].cost = cost,
            Err(i) => {
                outgoing.insert(
                    i,
                    Edge {
                        destination: to,
                        cost,
                    },
                );
                self.edges += 1;
            }
        }

        let incoming = &mut self.vertices[from].incoming;
        match incoming.binary_search_by_key(&from, |e| e.destination) {
            Ok(i) => incoming[i].cost = cost,
            Err(i) => incoming.insert(
                i,
                Edge {
                    destination: from,
                    cost,
                },
            ),
        }
    }

    /// bulk loading method
    pub fn try_add_edges(&mut self, edges: &[(usize, usize, f64)]) -> Result<(), &'static str> {
        let mut result = Ok(());
        for &(from, to, cost) in edges {
            if from >= self.vertices.len() || to >= self.vertices.len() {
                result = Err("Edge vertices don't exist");
                break;
            }
            self.vertices[from].outgoing.push(Edge {
                destination: to,
                cost,
            });
            self.vertices[to].outgoing.push(Edge {
                destination: from,
                cost,
            });
        }

        self.edges = 0;
        for vertex in &mut self.vertices {
            vertex.incoming.sort_by_key(|e| e.destination);
            vertex.incoming.dedup_by(|a, b| {
                // in the case of duplicate edges, we want to keep the *last* edge in the list.
                // since Vec::dedup keeps the first value instead of the last, we need to swap the
                // values when they're in the same bucket to get that behavior
                let same_bucket = a.destination == b.destination;
                if same_bucket {
                    std::mem::swap(a, b);
                }
                same_bucket
            });
            vertex.outgoing.sort_by_key(|e| e.destination);
            vertex.outgoing.dedup_by(|a, b| {
                let same_bucket = a.destination == b.destination;
                if same_bucket {
                    std::mem::swap(a, b);
                }
                same_bucket
            });

            self.edges += vertex.outgoing.len();
        }

        result
    }

    pub fn vertex_data(&self, vertex: usize) -> &V {
        &self.vertices[vertex].data
    }

    pub fn outgoing_edges(&self, vertex: usize) -> &[Edge<usize>] {
        &self.vertices[vertex].outgoing
    }

    pub fn incoming_edges(&self, vertex: usize) -> &[Edge<usize>] {
        &self.vertices[vertex].incoming
    }

    pub unsafe fn vertex_data_unchecked(&self, vertex: usize) -> &V {
        &self.vertices.get_unchecked(vertex).data
    }

    pub unsafe fn outgoing_edges_unchecked(&self, vertex: usize) -> &[Edge<usize>] {
        &self.vertices.get_unchecked(vertex).outgoing
    }

    pub unsafe fn incoming_edges_unchecked(&self, vertex: usize) -> &[Edge<usize>] {
        &self.vertices.get_unchecked(vertex).incoming
    }

    pub fn len(&self) -> usize {
        self.vertices.len()
    }

    pub fn total_edges(&self) -> usize {
        self.edges
    }

    /// note: runtime is logarithmic in the number of edges on the from vertex.
    pub fn find_edge(&self, from: usize, to: usize) -> Option<&Edge<usize>> {
        self.vertices[from]
            .outgoing
            .binary_search_by_key(&to, |e| e.destination)
            .ok()
            .map(|i| &self.vertices[from].outgoing[i])
    }
}

#[cfg(feature = "serde")]
mod serde {
    use serde::ser::{SerializeSeq, SerializeStruct};
    use serde::{Deserialize, Serialize};

    impl<V: Serialize> Serialize for super::DirectedGraph<V> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let mut s = serializer.serialize_struct("DirectedGraph", 2)?;
            s.serialize_field(
                "vertices",
                &StreamingSequence {
                    count: self.vertices.len(),
                    iter: || self.vertices.iter().map(|v| &v.data),
                },
            )?;
            s.serialize_field(
                "edges",
                &StreamingSequence {
                    count: self.edges,
                    iter: || {
                        self.vertices.iter().enumerate().flat_map(|(from, v)| {
                            v.outgoing
                                .iter()
                                .map(move |e| (from, e.destination, e.cost))
                        })
                    },
                },
            )?;
            s.end()
        }
    }

    struct StreamingSequence<F> {
        count: usize,
        iter: F,
    }

    impl<F, I, T> Serialize for StreamingSequence<F>
    where
        T: Serialize,
        F: Fn() -> I,
        I: Iterator<Item = T>,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let mut s = serializer.serialize_seq(Some(self.count))?;
            for v in (self.iter)() {
                s.serialize_element(&v)?;
            }
            s.end()
        }
    }

    #[derive(Serialize, Deserialize)]
    struct DirectedGraphTransport<V> {
        vertices: Vec<V>,
        edges: Vec<(usize, usize, f64)>,
    }

    impl<'de, V: Deserialize<'de>> Deserialize<'de> for super::DirectedGraph<V> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let v = DirectedGraphTransport::deserialize(deserializer)?;
            let mut graph = super::DirectedGraph::new();
            for data in v.vertices {
                graph.add_vertex(data);
            }
            if let Err(e) = graph.try_add_edges(&v.edges) {
                return Err(serde::de::Error::custom(e));
            }
            Ok(graph)
        }
    }
}
