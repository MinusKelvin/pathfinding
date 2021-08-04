use crate::traits::QueueLocation;
use crate::{NodePool, Queue};

pub struct PriorityQueue<V, F> {
    heap: Vec<V>,
    is_le: F,
}

impl<V, F> PriorityQueue<V, F> {
    pub fn new(is_le: F) -> Self {
        PriorityQueue {
            heap: vec![],
            is_le,
        }
    }
}

impl<V, N, F> Queue<V, N> for PriorityQueue<V, F>
where
    V: Copy + Eq,
    N: QueueLocation,
    F: Comparator<N>,
{
    fn push(&mut self, vertex: V, node_pool: &mut impl NodePool<V, N>) {
        if !self.contains(vertex, node_pool) {
            let index = self.heap.len();
            self.heap.push(vertex);
            node_pool.generate(vertex).set_location(index);
            self.heapify_up(index, node_pool);
            return;
        }

        let node = node_pool.get(vertex).unwrap();
        let index = node.get_location();
        if index == 0 {
            self.heapify_down(0, node_pool);
            return;
        }

        let parent = node_pool.get(self.heap[(index - 1) / 2]).unwrap();
        if self.is_le.is_le(parent, node) {
            self.heapify_down(index, node_pool);
        } else {
            self.heapify_up(index, node_pool);
        }
    }

    fn pop(&mut self, node_pool: &mut impl NodePool<V, N>) -> Option<V> {
        match self.heap.len() {
            0 => None,
            1 => self.heap.pop(),
            _ => {
                let k = self.heap.swap_remove(0);
                node_pool.generate(self.heap[0]).set_location(0);
                self.heapify_down(0, node_pool);
                Some(k)
            }
        }
    }

    fn clear(&mut self) {
        self.heap.clear();
    }
}

impl<V: Copy + Eq, F> PriorityQueue<V, F> {
    fn contains<N>(&self, v: V, data: &mut impl NodePool<V, N>) -> bool
    where
        N: QueueLocation,
    {
        data.get(v)
            .and_then(|n| self.heap.get(n.get_location()))
            .map(|&a| a == v)
            .unwrap_or(false)
    }

    #[inline(always)]
    fn le<N>(&mut self, i: usize, j: usize, data: &mut impl NodePool<V, N>) -> bool
    where
        N: QueueLocation,
        F: Comparator<N>,
    {
        self.is_le.is_le(
            data.get(self.heap[i]).unwrap(),
            data.get(self.heap[j]).unwrap(),
        )
    }

    fn heapify_up<N>(&mut self, mut i: usize, data: &mut impl NodePool<V, N>)
    where
        N: QueueLocation,
        F: Comparator<N>,
    {
        while i != 0 {
            let parent = (i - 1) / 2;
            if self.le(parent, i, data) {
                break;
            }

            self.heap.swap(i, parent);
            data.generate(self.heap[i]).set_location(i);
            data.generate(self.heap[parent]).set_location(parent);

            i = parent;
        }
    }

    fn heapify_down<N>(&mut self, mut i: usize, data: &mut impl NodePool<V, N>)
    where
        N: QueueLocation,
        F: Comparator<N>,
    {
        loop {
            let c1 = i * 2 + 1;
            if c1 >= self.heap.len() {
                break;
            }
            let c2 = c1 + 1;

            let smaller_child = if c2 >= self.heap.len() || self.le(c1, c2, data) {
                c1
            } else {
                c2
            };

            if self.le(i, smaller_child, data) {
                break;
            }

            self.heap.swap(i, smaller_child);
            data.generate(self.heap[i]).set_location(i);
            data.generate(self.heap[smaller_child])
                .set_location(smaller_child);

            i = smaller_child;
        }
    }
}

pub trait Comparator<Node> {
    fn is_le(&mut self, a: &Node, b: &Node) -> bool;
}

impl<N, F: Fn(&N, &N) -> bool> Comparator<N> for F {
    fn is_le(&mut self, a: &N, b: &N) -> bool {
        self(a, b)
    }
}
