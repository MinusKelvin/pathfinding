use pqueue::PriorityQueue;
use qcell::{TLCell, TLCellOwner};

pub mod bitgrid;
pub mod pqueue;
pub mod util;
pub mod weighted_grid;

#[derive(Debug, Copy, Clone)]
pub struct SearchNode<VertexId> {
    search_num: usize,
    pqueue_location: usize,
    pub expansions: usize,
    pub id: VertexId,
    pub parent: Option<VertexId>,
    pub g: f64,
    pub lb: f64,
}

#[derive(Debug, Copy, Clone)]
pub struct Edge<VertexId> {
    pub destination: VertexId,
    pub cost: f64,
}

pub enum SearchCellMarker {}
pub type Cell<T> = TLCell<SearchCellMarker, T>;
pub type Owner = TLCellOwner<SearchCellMarker>;

pub fn astar<VertexId>(
    pool: &mut impl NodePool<VertexId>,
    owner: &mut Owner,
    expansion_policy: &mut impl ExpansionPolicy<VertexId>,
    h: impl FnMut(VertexId) -> f64,
    source: VertexId,
    goal: VertexId,
) where
    VertexId: Copy + Eq,
{
    unsafe {
        // SAFETY: Since SafeNodePool and SafeExpansionPolicy always do bounds checks, so all vertex
        //         IDs are in-bounds for the purposes of safety.
        astar_unchecked(
            &mut SafeNodePool(pool),
            owner,
            &mut SafeExpansionPolicy(expansion_policy),
            h,
            source,
            goal,
        )
    }
}

/// SAFETY: The caller must ensure that the following invariants hold:
/// - `source` must be in-bounds of the expansion policy.
/// - `expansion_policy` must always produce edges whose destinations are in-bounds of the
///   expansion policy.
/// - If a vertex ID is in-bounds of the expansion policy, then it must be in-bounds of the node
///   pool.
pub unsafe fn astar_unchecked<VertexId>(
    pool: &mut impl NodePool<VertexId>,
    owner: &mut Owner,
    expansion_policy: &mut impl ExpansionPolicy<VertexId>,
    mut h: impl FnMut(VertexId) -> f64,
    source: VertexId,
    goal: VertexId,
) where
    VertexId: Copy + Eq,
{
    pool.reset();
    let mut queue = PriorityQueue::new();
    let mut edges = vec![];

    let source = pool.generate_unchecked(source, owner);
    owner.rw(source).g = 0.0;
    owner.rw(source).lb = 0.0;

    queue.decrease_key(source, owner);

    while let Some(node) = queue.pop(owner) {
        let n = owner.rw(node);
        n.expansions += 1;
        if n.id == goal {
            break;
        }

        expansion_policy.expand_unchecked(n, &mut edges);

        let parent_g = n.g;
        let parent_id = n.id;

        for edge in edges.drain(..) {
            let g = parent_g + edge.cost;
            let node = pool.generate_unchecked(edge.destination, owner);
            let n = owner.rw(node);
            if g < n.g {
                n.g = g;
                n.lb = g + h(n.id);
                n.parent = Some(parent_id);
                queue.decrease_key(node, owner);
            }
        }
    }
}

pub trait NodePool<VertexId> {
    fn reset(&mut self);
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

struct SafeNodePool<'a, N>(&'a mut N);
impl<V, N: NodePool<V>> NodePool<V> for SafeNodePool<'_, N> {
    fn reset(&mut self) {
        self.0.reset()
    }

    fn generate(&self, id: V, owner: &mut Owner) -> &Cell<SearchNode<V>> {
        self.0.generate(id, owner)
    }
}

struct SafeExpansionPolicy<'a, E>(&'a mut E);
impl<V, E: ExpansionPolicy<V>> ExpansionPolicy<V> for SafeExpansionPolicy<'_, E> {
    fn expand(&mut self, node: &SearchNode<V>, edges: &mut Vec<Edge<V>>) {
        self.0.expand(node, edges)
    }
}
