use crate::{Cell, Owner, SearchNode};

pub struct PriorityQueue<'a> {
    heap: Vec<&'a Cell<SearchNode>>,
}

impl<'a> PriorityQueue<'a> {
    pub fn new() -> Self {
        PriorityQueue { heap: vec![] }
    }

    pub fn decrease_key(&mut self, node: &'a Cell<SearchNode>, owner: &mut Owner) {
        if !self.contains(node, owner) {
            let index = self.heap.len();
            self.heap.push(node);
            owner.rw(node).pqueue_location = index;
            self.heapify_up(index, owner);
            return;
        }

        let index = owner.ro(node).pqueue_location;
        self.heapify_up(index, owner);
    }

    pub fn pop(&mut self, owner: &mut Owner) -> Option<&'a Cell<SearchNode>> {
        match self.heap.len() {
            0 => None,
            1 => self.heap.pop(),
            _ => {
                let k = self.heap.swap_remove(0);
                owner.rw(self.heap[0]).pqueue_location = 0;
                self.heapify_down(0, owner);
                Some(k)
            }
        }
    }

    fn contains(&self, node: &'a Cell<SearchNode>, owner: &mut Owner) -> bool {
        self.heap
            .get(owner.ro(node).pqueue_location)
            .map_or(false, |&occupant| std::ptr::eq(node, occupant))
    }

    #[inline(always)]
    fn le(&mut self, i: usize, j: usize, owner: &Owner) -> bool {
        let a = owner.ro(self.heap[i]);
        let b = owner.ro(self.heap[j]);
        if a.lb < b.lb {
            true
        } else if a.lb > b.lb {
            false
        } else {
            a.g >= b.g
        }
    }

    fn heapify_up(&mut self, mut i: usize, owner: &mut Owner) {
        while i != 0 {
            let parent = (i - 1) / 2;
            if self.le(parent, i, owner) {
                break;
            }

            self.heap.swap(i, parent);
            owner.rw(self.heap[i]).pqueue_location = i;
            owner.rw(self.heap[parent]).pqueue_location = parent;

            i = parent;
        }
    }

    fn heapify_down(&mut self, mut i: usize, owner: &mut Owner) {
        assert!(i < self.heap.len());
        loop {
            let c1 = i * 2 + 1;
            if c1 >= self.heap.len() {
                break;
            }
            let c2 = c1 + 1;

            // no, seriously, using || was causing bad codegen without the bounds checks
            let smaller_child = if c2 >= self.heap.len() {
                c1
            } else if self.le(c1, c2, owner) {
                c1
            } else {
                c2
            };

            if i >= self.heap.len() || smaller_child >= self.heap.len() {
                // SAFETY: We check that c1 is in-bounds. If c2 is not in-bounds, then smaller_child
                //         is chosen to be c1, otherwise in either case smaller_child is in-bounds.
                //         On the first iteration, the assert checks that i is in-bounds. On
                //         subsequent iterations, i inherits being in-bounds from smaller_child.
                // Hence, this point can never be reached.
                unsafe { std::hint::unreachable_unchecked() }
            }
            if self.le(i, smaller_child, owner) {
                break;
            }

            self.heap.swap(i, smaller_child);
            owner.rw(self.heap[i]).pqueue_location = i;
            owner.rw(self.heap[smaller_child]).pqueue_location = smaller_child;

            i = smaller_child;
        }
    }
}
