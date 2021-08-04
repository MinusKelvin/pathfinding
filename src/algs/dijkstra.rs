use std::marker::PhantomData;

use crate::pqueue::PriorityQueue;
use crate::traits::{Cost, Destination, GValue, NodeBase, Parent, QueueLocation};
use crate::{EdgeNavigator, NodePool, Searcher};

pub type DijkstraSearcher<VertexId, N, S, Node = DijkstraNode<VertexId>> = Searcher<
    VertexId,
    Node,
    N,
    S,
    DijkstraNavigator,
    PriorityQueue<VertexId, DijkstraOrdering<Node>>,
>;

impl<VertexId, Node, N, S> DijkstraSearcher<VertexId, N, S, Node>
where
    Node: GValue,
{
    pub fn new(node_pool: N, edge_gen: S) -> Self {
        DijkstraSearcher {
            node_pool,
            edge_gen,
            edge_nav: DijkstraNavigator,
            queue: PriorityQueue::new(dijkstra_ordering()),
            _marker: PhantomData,
        }
    }
}

pub struct DijkstraNavigator;

impl<V, N, E> EdgeNavigator<V, N, E> for DijkstraNavigator
where
    V: Copy,
    N: GValue + Parent<V>,
    E: Destination<V> + Cost,
{
    fn navigate(&mut self, from: V, along: &E, node_pool: &mut impl NodePool<V, N>) -> Option<V> {
        let destination = along.destination();
        let new_g = node_pool.get(from).unwrap().get_g() + along.cost();
        let node = node_pool.generate(destination);
        if new_g < node.get_g() {
            node.set_g(new_g);
            node.set_parent(from);
            Some(destination)
        } else {
            None
        }
    }
}

#[cfg(feature = "nightly")]
type DijkstraOrdering<Node> = impl Fn(&Node, &Node) -> bool;
#[cfg(not(feature = "nightly"))]
type DijkstraOrdering<Node> = fn(&Node, &Node) -> bool;

fn dijkstra_ordering<Node: GValue>() -> DijkstraOrdering<Node> {
    |a: &Node, b: &Node| a.get_g() <= b.get_g()
}

pub struct DijkstraNode<V> {
    parent: Option<V>,
    expansions: usize,
    location: usize,
    g: f64,
}

impl<V> NodeBase for DijkstraNode<V> {
    fn make_source(&mut self) {
        self.g = 0.0;
    }

    fn expand(&mut self) {
        self.expansions += 1;
    }

    fn get_expansions(&self) -> usize {
        self.expansions
    }
}

impl<V: Copy> Parent<V> for DijkstraNode<V> {
    fn set_parent(&mut self, p: V) {
        self.parent = Some(p);
    }

    fn get_parent(&self) -> Option<V> {
        self.parent
    }
}

impl<V> GValue for DijkstraNode<V> {
    fn get_g(&self) -> f64 {
        self.g
    }

    fn set_g(&mut self, g: f64) {
        self.g = g;
    }
}

impl<V> QueueLocation for DijkstraNode<V> {
    fn get_location(&self) -> usize {
        self.location
    }

    fn set_location(&mut self, index: usize) {
        self.location = index;
    }
}

impl<V> Default for DijkstraNode<V> {
    fn default() -> Self {
        DijkstraNode {
            expansions: 0,
            location: 0,
            parent: None,
            g: f64::INFINITY,
        }
    }
}
