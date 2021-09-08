use crate::domains::DirectedGraph;
use crate::util::IndexDomain;

use super::ExpansionPolicy;

pub struct OutgoingEdges<'a, V>(&'a DirectedGraph<V>);

impl<'a, V> OutgoingEdges<'a, V> {
    pub fn new(graph: &'a DirectedGraph<V>) -> Self {
        OutgoingEdges(graph)
    }
}

impl<V> ExpansionPolicy<usize> for OutgoingEdges<'_, V> {
    unsafe fn expand_unchecked(
        &mut self,
        node: &crate::SearchNode<usize>,
        edges: &mut Vec<crate::Edge<usize>>,
    ) {
        edges.extend_from_slice(self.0.outgoing_edges_unchecked(node.id));
    }

    fn expand(&mut self, node: &crate::SearchNode<usize>, edges: &mut Vec<crate::Edge<usize>>) {
        assert!(node.id < self.0.len());
        unsafe {
            // SAFETY: Bounds checked above
            self.expand_unchecked(node, edges)
        }
    }
}

// SAFETY: DirectedGraph always contains valid edges, so all edges are in-bounds.
unsafe impl<V> IndexDomain for OutgoingEdges<'_, V> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

pub struct IncomingEdges<'a, V>(&'a DirectedGraph<V>);

impl<'a, V> IncomingEdges<'a, V> {
    pub fn new(graph: &'a DirectedGraph<V>) -> Self {
        IncomingEdges(graph)
    }
}

impl<V> ExpansionPolicy<usize> for IncomingEdges<'_, V> {
    unsafe fn expand_unchecked(
        &mut self,
        node: &crate::SearchNode<usize>,
        edges: &mut Vec<crate::Edge<usize>>,
    ) {
        edges.extend_from_slice(self.0.incoming_edges_unchecked(node.id));
    }

    fn expand(&mut self, node: &crate::SearchNode<usize>, edges: &mut Vec<crate::Edge<usize>>) {
        assert!(node.id < self.0.len());
        unsafe {
            // SAFETY: Bounds checked above
            self.expand_unchecked(node, edges)
        }
    }
}

// SAFETY: DirectedGraph always contains valid edges, so all edges are in-bounds.
unsafe impl<V> IndexDomain for IncomingEdges<'_, V> {
    fn len(&self) -> usize {
        self.0.len()
    }
}
