use crate::{Edge, SearchNode};

pub mod bitgrid;
pub mod weighted_grid;

pub trait ExpansionPolicy<VertexId> {
    fn expand(&mut self, node: &SearchNode<VertexId>, edges: &mut Vec<Edge<VertexId>>);

    /// SAFETY: The caller must ensure that the supplied vertex ID is in-bounds for this expansion
    ///         policy.
    unsafe fn expand_unchecked(
        &mut self,
        node: &SearchNode<VertexId>,
        edges: &mut Vec<Edge<VertexId>>,
    ) {
        self.expand(node, edges)
    }
}
