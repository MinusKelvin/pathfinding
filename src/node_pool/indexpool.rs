use crate::{Cell, Owner, SearchNode};

use super::NodePool;

pub struct IndexPool {
    search_num: usize,
    pool: Vec<Cell<SearchNode<usize>>>,
}

impl IndexPool {
    pub fn new(size: usize) -> Self {
        let mut pool = Vec::with_capacity(size);
        for id in 0..size {
            pool.push(Cell::new(SearchNode {
                search_num: 0,
                id,
                expansions: 0,
                g: 0.0,
                lb: 0.0,
                parent: None,
                pqueue_location: 0,
            }));
        }
        IndexPool {
            search_num: 0,
            pool,
        }
    }
}

impl NodePool<usize> for IndexPool {
    fn reset(&mut self, owner: &mut Owner) {
        match self.search_num.checked_add(1) {
            Some(ok) => self.search_num = ok,
            None => {
                // on the off chance we do a search while there are still nodes with search nums
                // equal to the new search num after an overflow, it would be a *really* hard to
                // diagnose logic bug, so we nip it in the bud by resetting everything on overflow.
                self.search_num = 1;
                for node in &self.pool {
                    owner.rw(node).search_num = 0;
                }
            }
        }
    }

    fn generate(&self, id: usize, owner: &mut Owner) -> &Cell<SearchNode<usize>> {
        assert!(id < self.pool.len());
        unsafe {
            // SAFETY: bounds checked above
            self.generate_unchecked(id, owner)
        }
    }

    unsafe fn generate_unchecked(
        &self,
        id: usize,
        owner: &mut Owner,
    ) -> &Cell<SearchNode<usize>> {
        let cell = self.pool.get_unchecked(id);
        if owner.ro(cell).search_num == self.search_num {
            cell
        } else {
            let n = owner.rw(cell);
            n.lb = f64::INFINITY;
            n.g = f64::INFINITY;
            n.expansions = 0;
            n.search_num = self.search_num;
            n.parent = None;
            cell
        }
    }
}
