use crate::util::GridDomain;
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

unsafe impl GridDomain for GridPool {
    fn width(&self) -> i32 {
        self.grid.width()
    }

    fn height(&self) -> i32 {
        self.grid.height()
    }
}

impl NodePool<(i32, i32)> for GridPool {
    fn reset(&mut self, owner: &mut Owner) {
        match self.search_num.checked_add(1) {
            Some(ok) => self.search_num = ok,
            None => {
                // on the off chance we do a search while there are still nodes with search nums
                // equal to the new search num after an overflow, it would be a *really* hard to
                // diagnose logic bug, so we nip it in the bud by resetting everything on overflow.
                self.search_num = 1;
                for y in 0..self.grid.height() {
                    for x in 0..self.grid.width() {
                        owner.rw(self.grid.get(x, y)).search_num = 0;
                    }
                }
            }
        }
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