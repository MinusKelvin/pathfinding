use crate::util::GridDomain;
use crate::{Cell, Owner, SearchNode};

use super::NodePool;

pub struct GridPool {
    search_num: usize,
    width: i32,
    height: i32,
    grid: Box<[Cell<SearchNode<(i32, i32)>>]>,
}

impl GridPool {
    pub fn new(width: i32, height: i32) -> Self {
        assert!(width > 0 && height > 0, "width and height must be positive");
        let mut grid = Vec::with_capacity(width as usize * height as usize);
        for y in 0..height {
            for x in 0..width {
                grid.push(Cell::new(SearchNode {
                    search_num: 0,
                    expansions: 0,
                    pqueue_location: 0,
                    id: (x, y),
                    parent: None,
                    g: 0.0,
                    lb: 0.0,
                }));
            }
        }
        GridPool {
            search_num: 0,
            width,
            height,
            grid: grid.into_boxed_slice(),
        }
    }

    #[track_caller]
    pub fn get(&self, x: i32, y: i32, owner: &Owner) -> Option<&Cell<SearchNode<(i32, i32)>>> {
        self.bounds_check(x, y);
        let cell = unsafe {
            // SAFETY: bounds checked above
            self.grid.get_unchecked(self.locate(x, y))
        };
        if owner.ro(cell).search_num == self.search_num {
            Some(cell)
        } else {
            None
        }
    }

    #[inline(always)]
    #[track_caller]
    fn bounds_check(&self, x: i32, y: i32) {
        if !(0..self.width).contains(&x) || !(0..self.height).contains(&y) {
            panic!("Grid cell ({}, {}) is out of bounds.", x, y);
        }
    }

    #[inline(always)]
    fn locate(&self, x: i32, y: i32) -> usize {
        #[cfg(debug_assertions)]
        self.bounds_check(x, y);

        x as usize + y as usize * self.width as usize
    }
}

unsafe impl GridDomain for GridPool {
    fn width(&self) -> i32 {
        self.width
    }

    fn height(&self) -> i32 {
        self.height
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
                for cell in self.grid.iter() {
                    owner.rw(cell).search_num = 0;
                }
            }
        }
    }

    fn generate(&self, (x, y): (i32, i32), owner: &mut Owner) -> &Cell<SearchNode<(i32, i32)>> {
        self.bounds_check(x, y);
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
        let cell = self.grid.get_unchecked(self.locate(x, y));
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
