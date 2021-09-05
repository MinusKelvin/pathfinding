use crate::{Cell, Owner, SearchNode};

mod gridpool;
pub use self::gridpool::GridPool;
mod hashpool;
pub use self::hashpool::HashPool;
mod indexpool;
pub use self::indexpool::IndexPool;

pub trait NodePool<VertexId> {
    fn reset(&mut self, owner: &mut Owner);
    fn generate(&self, id: VertexId, owner: &mut Owner) -> &Cell<SearchNode<VertexId>>;

    /// SAFETY: The caller must ensure that the supplied vertex ID is in-bounds for this node pool.
    unsafe fn generate_unchecked(
        &self,
        id: VertexId,
        owner: &mut Owner,
    ) -> &Cell<SearchNode<VertexId>> {
        self.generate(id, owner)
    }
}
