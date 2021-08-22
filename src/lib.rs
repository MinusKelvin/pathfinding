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
type Cell<T> = TLCell<SearchCellMarker, T>;
type Owner = TLCellOwner<SearchCellMarker>;

pub fn astar<VertexId>(
    pool: &mut impl NodePool<VertexId>,
    owner: &mut Owner,
    expansion_policy: &mut impl ExpansionPolicy<VertexId>,
    mut h: impl FnMut(VertexId) -> f64,
    source: VertexId,
    goal: VertexId,
) where VertexId: Copy + Eq {
    pool.reset();
    let mut queue = PriorityQueue::new();
    let mut edges = vec![];

    let source = pool.get_mut(source, owner);
    owner.rw(source).g = 0.0;
    owner.rw(source).lb = 0.0;

    queue.decrease_key(source, owner);

    while let Some(node) = queue.pop(owner) {
        let n = owner.rw(node);
        n.expansions += 1;
        if n.id == goal {
            break;
        }

        expansion_policy.expand(n, &mut edges);

        let parent_g = n.g;
        let parent_id = n.id;

        for edge in edges.drain(..) {
            let g = parent_g + edge.cost;
            let node = pool.get_mut(edge.destination, owner);
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
    fn get_mut(&self, id: VertexId, owner: &mut Owner) -> &Cell<SearchNode<VertexId>>;
}

pub trait ExpansionPolicy<VertexId> {
    fn expand(&mut self, node: &SearchNode<VertexId>, edges: &mut Vec<Edge<VertexId>>);
}
