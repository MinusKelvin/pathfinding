use std::collections::VecDeque;
use std::marker::PhantomData;

use crate::traits::{Destination, NodeBase, Parent};
use crate::{EdgeNavigator, NodePool, Queue, Searcher};

pub type BreadthFirstSearcher<VertexId, N, S, Node = BasicNode<VertexId>> =
    Searcher<VertexId, Node, N, S, BlindNavigator, BfsQueue<VertexId>>;

impl<VertexId, Node, N, S> BreadthFirstSearcher<VertexId, N, S, Node> {
    pub fn new(node_pool: N, edge_gen: S) -> Self {
        BreadthFirstSearcher {
            node_pool,
            edge_gen,
            edge_nav: BlindNavigator,
            queue: BfsQueue::default(),
            _marker: PhantomData,
        }
    }
}

pub struct BfsQueue<V> {
    queue: VecDeque<V>,
}

impl<V: Copy + Eq, N: NodeBase> Queue<V, N> for BfsQueue<V> {
    fn push(&mut self, vertex: V, _node_pool: &mut impl NodePool<V, N>) {
        self.queue.push_back(vertex);
    }

    fn pop(&mut self, node_pool: &mut impl NodePool<V, N>) -> Option<V> {
        loop {
            let v = self.queue.pop_front()?;
            if node_pool.generate(v).get_expansions() == 0 {
                return Some(v);
            }
        }
    }

    fn clear(&mut self) {
        self.queue.clear();
    }
}

impl<V> Default for BfsQueue<V> {
    fn default() -> Self {
        BfsQueue {
            queue: VecDeque::new(),
        }
    }
}

pub type DepthFirstSearcher<VertexId, N, S, Node = BasicNode<VertexId>> =
    Searcher<VertexId, Node, N, S, BlindNavigator, DfsQueue<VertexId>>;

impl<VertexId, Node, N, S> DepthFirstSearcher<VertexId, N, S, Node> {
    pub fn new(node_pool: N, edge_gen: S) -> Self {
        DepthFirstSearcher {
            node_pool,
            edge_gen,
            edge_nav: BlindNavigator,
            queue: DfsQueue::default(),
            _marker: PhantomData,
        }
    }
}

pub struct DfsQueue<V> {
    stack: Vec<V>,
}

impl<V: Copy + Eq, N: NodeBase> Queue<V, N> for DfsQueue<V> {
    fn push(&mut self, vertex: V, _node_pool: &mut impl NodePool<V, N>) {
        self.stack.push(vertex);
    }

    fn pop(&mut self, node_pool: &mut impl NodePool<V, N>) -> Option<V> {
        loop {
            let v = self.stack.pop()?;
            if node_pool.generate(v).get_expansions() == 0 {
                return Some(v);
            }
        }
    }

    fn clear(&mut self) {
        self.stack.clear();
    }
}

impl<V> Default for DfsQueue<V> {
    fn default() -> Self {
        DfsQueue { stack: vec![] }
    }
}

pub struct BlindNavigator;

impl<V: Copy, N: Parent<V> + NodeBase, E: Destination<V>> EdgeNavigator<V, N, E>
    for BlindNavigator
{
    fn navigate(&mut self, from: V, along: &E, node_pool: &mut impl NodePool<V, N>) -> Option<V> {
        let destination = along.destination();
        let target = node_pool.generate(destination);
        if target.get_expansions() != 0 {
            return None;
        }
        target.set_parent(from);
        Some(destination)
    }
}

pub struct BasicNode<V> {
    parent: Option<V>,
    expansions: usize,
}

impl<V> NodeBase for BasicNode<V> {
    fn make_source(&mut self) {}

    fn expand(&mut self) {
        self.expansions += 1;
    }

    fn get_expansions(&self) -> usize {
        self.expansions
    }
}

impl<V: Copy> Parent<V> for BasicNode<V> {
    fn set_parent(&mut self, p: V) {
        self.parent = Some(p);
    }

    fn get_parent(&self) -> Option<V> {
        self.parent
    }
}

impl<V> Default for BasicNode<V> {
    fn default() -> Self {
        BasicNode {
            expansions: 0,
            parent: None,
        }
    }
}
