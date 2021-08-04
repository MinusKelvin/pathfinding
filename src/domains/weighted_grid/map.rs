pub struct WeightedGrid<V> {
    width: i32,
    height: i32,
    cells: Box<[V]>,
}

impl<V> WeightedGrid<V> {
    pub fn new(width: i32, height: i32, mut init: impl FnMut(i32, i32) -> V) -> Self {
        let mut cells = vec![];
        for y in -1..height + 1 {
            for x in -1..width + 1 {
                cells.push(init(x, y));
            }
        }
        WeightedGrid {
            width,
            height,
            cells: cells.into_boxed_slice(),
        }
    }

    #[inline(always)]
    pub fn get(&self, x: i32, y: i32) -> &V {
        self.bounds_check(x, y);
        unsafe { self.get_unchecked(x, y) }
    }

    #[inline(always)]
    pub fn get_mut(&mut self, x: i32, y: i32) -> &mut V {
        self.bounds_check(x, y);
        unsafe { self.get_unchecked_mut(x, y) }
    }

    /// SAFETY: `x` must be in `-1..width+1`, `y` must be in `-1..height+1`.
    #[inline(always)]
    pub unsafe fn get_unchecked(&self, x: i32, y: i32) -> &V {
        self.cells.get_unchecked(self.locate(x, y))
    }

    /// SAFETY: `x` must be in `-1..width+1`, `y` must be in `-1..height+1`.
    #[inline(always)]
    pub unsafe fn get_unchecked_mut(&mut self, x: i32, y: i32) -> &mut V {
        self.cells.get_unchecked_mut(self.locate(x, y))
    }

    #[inline(always)]
    fn locate(&self, x: i32, y: i32) -> usize {
        #[cfg(debug_assertions)]
        self.bounds_check(x, y);

        let padded_y = (y + 1) as usize;
        let padded_width = self.width as usize + 1;
        let padded_x = (x + 1) as usize;
        let id = padded_y * padded_width + padded_x;

        debug_assert!(id < padded_width * (self.height as usize + 2) + 1);

        id
    }

    #[track_caller]
    #[inline(always)]
    fn bounds_check(&self, x: i32, y: i32) {
        if !(-1..self.width + 1).contains(&x) || !(-1..self.height + 1).contains(&y) {
            panic!("Grid cell ({}, {}) is out of bounds.", x, y);
        }
    }
}
