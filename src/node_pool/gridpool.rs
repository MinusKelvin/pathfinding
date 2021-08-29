use crate::{Cell, Owner, SearchNode};
use crate::domains::WeightedGrid;

use super::NodePool;

pub struct GridPool {
    search_num: usize,
    grid: WeightedGrid<Cell<SearchNode<(i32, i32)>>>,
}

impl GridPool {
    pub fn new(width: i32, height: i32) -> Self {
        GridPool {
            search_num: 0,
            grid: WeightedGrid::new(width, height, |x, y| {
                Cell::new(SearchNode {
                    search_num: 0,
                    expansions: 0,
                    pqueue_location: 0,
                    id: (x, y),
                    parent: None,
                    g: 0.0,
                    lb: 0.0,
                })
            }),
        }
    }

    pub fn get(&self, x: i32, y: i32, owner: &Owner) -> Option<&Cell<SearchNode<(i32, i32)>>> {
        let cell = self.grid.get(x, y);
        if owner.ro(cell).search_num == self.search_num {
            Some(cell)
        } else {
            None
        }
    }
}

impl NodePool<(i32, i32)> for GridPool {
    fn reset(&mut self) {
        self.search_num += 1;
    }

    fn generate(&self, (x, y): (i32, i32), owner: &mut Owner) -> &Cell<SearchNode<(i32, i32)>> {
        self.grid.get(x, y);
        unsafe {
            // SAFETY: Bounds checked above.
            self.generate_unchecked((x, y), owner)
        }
    }

    unsafe fn generate_unchecked(
        &self,
        (x, y): (i32, i32),
        owner: &mut Owner,
    ) -> &Cell<SearchNode<(i32, i32)>> {
        let cell = self.grid.get_unchecked(x, y);
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