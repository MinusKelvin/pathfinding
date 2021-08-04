#![cfg_attr(feature = "nightly", feature(type_alias_impl_trait))]

use std::marker::PhantomData;

use traits::NodeBase;

pub mod algs;
pub mod domains;
pub mod pqueue;
pub mod traits;
pub mod util;

pub struct Searcher<VertexId, Node, N, S, E, Q> {
    pub node_pool: N,
    pub edge_gen: S,
    pub edge_nav: E,
    pub queue: Q,
    _marker: PhantomData<(VertexId, Node)>,
}

impl<VertexId, Node, N, S, E, Q> Searcher<VertexId, Node, N, S, E, Q>
where
    VertexId: Copy + Eq,
    Node: NodeBase,
    N: NodePool<VertexId, Node>,
    S: EdgeGenerator<VertexId, Node>,
    E: EdgeNavigator<VertexId, Node, S::Edge>,
    Q: Queue<VertexId, Node>,
{
    pub fn search(&mut self, source: VertexId, target: Option<VertexId>) {
        self.queue.clear();
        self.node_pool.reset();
        self.node_pool.generate(source).make_source();

        self.queue.push(source, &mut self.node_pool);

        while let Some(vertex) = self.queue.pop(&mut self.node_pool) {
            self.node_pool.generate(vertex).expand();
            if Some(vertex) == target {
                break;
            }

            for edge in self.edge_gen.edges(vertex, &mut self.node_pool) {
                if let Some(successor) = self.edge_nav.navigate(vertex, edge, &mut self.node_pool) {
                    self.queue.push(successor, &mut self.node_pool);
                }
            }
        }
    }
}

pub trait NodePool<V, N> {
    fn reset(&mut self);
    fn generate(&mut self, id: V) -> &mut N;
    fn get(&self, id: V) -> Option<&N>;
}

pub trait EdgeGenerator<V, N> {
    type Edge;
    fn edges(&mut self, of: V, node_pool: &mut impl NodePool<V, N>) -> &[Self::Edge];
}

pub trait EdgeNavigator<V, N, E> {
    fn navigate(&mut self, from: V, along: &E, node_pool: &mut impl NodePool<V, N>) -> Option<V>;
}

pub trait Queue<V, N> {
    fn push(&mut self, vertex: V, node_pool: &mut impl NodePool<V, N>);
    fn pop(&mut self, node_pool: &mut impl NodePool<V, N>) -> Option<V>;
    fn clear(&mut self);
}
