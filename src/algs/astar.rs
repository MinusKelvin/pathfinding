use std::marker::PhantomData;

use crate::pqueue::PriorityQueue;
use crate::traits::{Cost, Destination, GValue, LowerBound, NodeBase, Parent, QueueLocation};
use crate::{EdgeNavigator, NodePool, Searcher};

pub type AStarSearcher<VertexId, N, S, H, Node = AStarNode<VertexId>> =
    Searcher<VertexId, Node, N, S, AStarNavigator<H>, PriorityQueue<VertexId, AStarOrdering<Node>>>;

impl<VertexId, Node, N, S, H> AStarSearcher<VertexId, N, S, H, Node>
where
    Node: GValue + LowerBound,
{
    pub fn new(node_pool: N, edge_gen: S, heuristic: H) -> Self {
        AStarSearcher {
            node_pool,
            edge_gen,
            edge_nav: AStarNavigator { heuristic },
            queue: PriorityQueue::new(astar_ordering()),
            _marker: PhantomData,
        }
    }
}

pub struct AStarNavigator<H> {
    pub heuristic: H,
}

impl<V, N, E, H> EdgeNavigator<V, N, E> for AStarNavigator<H>
where
    V: Copy,
    N: GValue + LowerBound + Parent<V>,
    E: Destination<V> + Cost,
    H: FnMut(V) -> f64,
{
    fn navigate(&mut self, from: V, along: &E, node_pool: &mut impl NodePool<V, N>) -> Option<V> {
        let destination = along.destination();
        let new_g = node_pool.get(from).unwrap().get_g() + along.cost();
        let node = node_pool.generate(destination);
        if new_g < node.get_g() {
            node.set_g(new_g);
            node.set_lb(new_g + (self.heuristic)(destination));
            node.set_parent(from);
            Some(destination)
        } else {
            None
        }
    }
}

#[cfg(feature = "nightly")]
type AStarOrdering<Node> = impl Fn(&Node, &Node) -> bool;
#[cfg(not(feature = "nightly"))]
type AStarOrdering<Node> = fn(&Node, &Node) -> bool;

fn astar_ordering<Node: GValue + LowerBound>() -> AStarOrdering<Node> {
    |a: &Node, b: &Node| {
        if a.get_lb() < b.get_lb() {
            true
        } else if a.get_lb() > b.get_lb() {
            false
        } else {
            a.get_g() >= b.get_g()
        }
    }
}

pub struct AStarNode<V> {
    parent: Option<V>,
    expansions: usize,
    location: usize,
    g: f64,
    lb: f64,
}

impl<V> NodeBase for AStarNode<V> {
    fn make_source(&mut self) {
        self.g = 0.0;
        self.lb = 0.0;
    }

    fn expand(&mut self) {
        self.expansions += 1;
    }

    fn get_expansions(&self) -> usize {
        self.expansions
    }
}

impl<V: Copy> Parent<V> for AStarNode<V> {
    fn set_parent(&mut self, p: V) {
        self.parent = Some(p);
    }

    fn get_parent(&self) -> Option<V> {
        self.parent
    }
}

impl<V> GValue for AStarNode<V> {
    fn get_g(&self) -> f64 {
        self.g
    }

    fn set_g(&mut self, g: f64) {
        self.g = g;
    }
}

impl<V> LowerBound for AStarNode<V> {
    fn get_lb(&self) -> f64 {
        self.lb
    }

    fn set_lb(&mut self, lb: f64) {
        self.lb = lb;
    }
}

impl<V> QueueLocation for AStarNode<V> {
    fn get_location(&self) -> usize {
        self.location
    }

    fn set_location(&mut self, index: usize) {
        self.location = index;
    }
}

impl<V> Default for AStarNode<V> {
    fn default() -> Self {
        AStarNode {
            expansions: 0,
            location: 0,
            parent: None,
            g: f64::INFINITY,
            lb: std::f64::INFINITY,
        }
    }
}
