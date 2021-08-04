use enumset::EnumSet;

use crate::util::Direction;

pub struct BitGrid {
    width: i32,
    height: i32,
    cells: Box<[u8]>,
}

impl BitGrid {
    pub fn new(width: i32, height: i32) -> Self {
        assert!(width > 0 && height > 0, "width and height must be positive");
        // there is 1 padding bit at the end of each row
        let padded_width = width as usize + 1;
        // there is a padding row above and a padding row below.
        let padded_height = height as usize + 2;
        // there is one extra bit so that the unpadded coordinate (width, height),
        // which is 1 cell out of bounds on each axis, can be dereferenced.
        let padded_size = padded_width * padded_height + 1;
        let bytes = (padded_size - 1) / u8::BITS as usize + 1;

        let mut this = BitGrid {
            width,
            height,
            cells: vec![0; 8 + bytes + 8].into_boxed_slice(),
        };

        // initialize padding to 1s
        this.cells[..8].fill(!0);
        let l = this.cells.len();
        this.cells[l - 8..].fill(!0);
        unsafe {
            for x in -1..width {
                this.set_unchecked(x, -1, true);
                this.set_unchecked(x, height, true);
            }
            for y in 0..height {
                this.set_unchecked(-1, y, true);
            }
            this.set_unchecked(width, height, true);
        }

        this
    }

    #[inline(always)]
    pub fn width(&self) -> i32 {
        self.width
    }

    #[inline(always)]
    pub fn height(&self) -> i32 {
        self.height
    }

    #[track_caller]
    #[inline(always)]
    pub fn get(&self, x: i32, y: i32) -> bool {
        self.padded_bounds_check(x, y);
        unsafe { self.get_unchecked(x, y) }
    }

    #[track_caller]
    #[inline(always)]
    pub fn set(&mut self, x: i32, y: i32, v: bool) {
        self.unpadded_bounds_check(x, y);
        unsafe { self.set_unchecked(x, y, v) }
    }

    /// Note: returns 57 tiles of information. The top 7 bits are always 0.
    #[track_caller]
    #[inline(always)]
    pub fn get_row(&self, x: i32, y: i32) -> u64 {
        self.padded_bounds_check(x, y);
        unsafe { self.get_row_unchecked(x, y) }
    }

    /// Note: returns 57 tiles of information. The bottom 7 bits are always 0.
    #[track_caller]
    #[inline(always)]
    pub fn get_row_upper(&self, x: i32, y: i32) -> u64 {
        self.padded_bounds_check(x, y);
        unsafe { self.get_row_upper_unchecked(x, y) }
    }

    #[track_caller]
    #[inline(always)]
    pub fn get_neighbors(&self, x: i32, y: i32) -> EnumSet<Direction> {
        self.unpadded_bounds_check(x, y);
        unsafe { self.get_neighbors_unchecked(x, y) }
    }

    /// SAFETY: `x` must be in `-1..width+1`, `y` must be in `-1..height+1`.
    /// Padding bits can be relied upon to yield `true`.
    #[inline(always)]
    pub unsafe fn get_unchecked(&self, x: i32, y: i32) -> bool {
        let (idx, bit) = self.locate(x, y);
        self.cells.get_unchecked(idx) & 1 << bit != 0
    }

    /// SAFETY: `x` must be in `0..width`, `y` must be in `0..height`
    #[inline(always)]
    pub unsafe fn set_unchecked(&mut self, x: i32, y: i32, v: bool) {
        let (idx, bit) = self.locate(x, y);
        if v {
            *self.cells.get_unchecked_mut(idx) |= 1 << bit;
        } else {
            *self.cells.get_unchecked_mut(idx) &= !(1 << bit);
        }
    }

    /// SAFETY: `x` must be in `-1..width+1`, `y` must be in `-1..height+1`.
    /// Padding bits can be relied upon to yield `true`.
    ///
    /// Note: returns 57 tiles of information. The top 7 bits are always 0.
    #[inline(always)]
    pub unsafe fn get_row_unchecked(&self, x: i32, y: i32) -> u64 {
        let (idx, bit) = self.locate(x, y);
        let ptr: *const u8 = self.cells.get_unchecked(idx);
        let w = (ptr as *const u64).read_unaligned().to_le();
        (w >> bit) & (1 << 57) - 1
    }

    /// SAFETY: `x` must be in `-1..width+1`, `y` must be in `-1..height+1`.
    /// Padding bits can be relied upon to yield `true`.
    ///
    /// Note: returns 57 tiles of information. The bottom 7 bits are always 0.
    #[inline(always)]
    pub unsafe fn get_row_upper_unchecked(&self, x: i32, y: i32) -> u64 {
        let (idx, bit) = self.locate(x, y);
        let ptr: *const u8 = self.cells.get_unchecked(idx - 7);
        let w = (ptr as *const u64).read_unaligned().to_le();
        (w << 7 - bit) & !0 << 7
    }

    /// SAFETY: `x` must be in `0..width`, `y` must be in `0..height`
    #[inline(always)]
    pub unsafe fn get_neighbors_unchecked(&self, x: i32, y: i32) -> EnumSet<Direction> {
        let upper = self.get_row_unchecked(x - 1, y - 1);
        let middle = self.get_row_unchecked(x - 1, y);
        let lower = self.get_row_unchecked(x - 1, y + 1);
        let bits =
            upper & 0b111 | (middle & 0b1) << 3 | (middle & 0b100) << 2 | (lower & 0b111) << 5;
        EnumSet::from_u64_truncated(bits)
    }

    #[inline(always)]
    fn locate(&self, x: i32, y: i32) -> (usize, usize) {
        #[cfg(debug_assertions)]
        self.padded_bounds_check(x, y);

        let padded_y = (y + 1) as usize;
        let padded_width = self.width as usize + 1;
        let padded_x = (x + 1) as usize;
        let id = padded_y * padded_width + padded_x;

        debug_assert!(id < padded_width * (self.height as usize + 2) + 1);

        (id / u8::BITS as usize + 8, id % u8::BITS as usize)
    }

    #[track_caller]
    #[inline(always)]
    fn padded_bounds_check(&self, x: i32, y: i32) {
        #[cfg(not(feature = "unsound"))]
        if !(-1..self.width + 1).contains(&x) || !(-1..self.height + 1).contains(&y) {
            panic!("Grid cell ({}, {}) is out of bounds.", x, y);
        }
    }

    #[track_caller]
    #[inline(always)]
    fn unpadded_bounds_check(&self, x: i32, y: i32) {
        #[cfg(not(feature = "unsound"))]
        if !(0..self.width).contains(&x) || !(0..self.height).contains(&y) {
            panic!("Grid cell ({}, {}) is out of bounds.", x, y);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::prelude::*;
    use rand_pcg::Pcg64;

    #[test]
    fn check_empty() {
        let grid = BitGrid::new(211, 53);
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                assert_eq!(grid.get(x, y), false);
            }
        }
        for x in -1..grid.width() + 1 {
            assert_eq!(grid.get(x, -1), true);
            assert_eq!(grid.get(x, grid.height()), true);
        }
        for y in -1..grid.height() + 1 {
            assert_eq!(grid.get(-1, y), true);
            assert_eq!(grid.get(grid.width(), y), true);
        }
    }

    fn random_board() -> ([[bool; 173]; 89], BitGrid) {
        let canonical_grid: [[bool; 173]; 89] =
            Pcg64::new(0xcafef00dd15ea5e5, 0xa02bdbf7bb3c0a7ac28fa16a64abf96).gen();
        let mut grid = BitGrid::new(canonical_grid[0].len() as i32, canonical_grid.len() as i32);
        for y in 0..canonical_grid.len() {
            for x in 0..canonical_grid[y].len() {
                grid.set(x as i32, y as i32, canonical_grid[y][x]);
            }
        }
        (canonical_grid, grid)
    }

    #[test]
    fn check_random() {
        let (canonical_grid, grid) = random_board();
        for y in 0..canonical_grid.len() {
            for x in 0..canonical_grid[y].len() {
                assert_eq!(grid.get(x as i32, y as i32), canonical_grid[y][x]);
            }
        }
    }

    #[test]
    fn check_bits() {
        let (canonical_grid, grid) = random_board();
        for y in 0..canonical_grid.len() {
            for x in 0..canonical_grid[y].len() {
                let r = grid.get_row(x as i32, y as i32);
                for i in 0..57 {
                    if x + i < canonical_grid[y].len() {
                        assert_eq!(r & 1 << i != 0, canonical_grid[y][x + i]);
                    } else {
                        assert_eq!(r & 1 << i != 0, true);
                        break;
                    }
                }
            }
        }
    }

    #[test]
    fn check_bits_upper() {
        let (canonical_grid, grid) = random_board();
        for y in 0..canonical_grid.len() {
            for x in 0..canonical_grid[y].len() {
                let r = grid.get_row_upper(x as i32, y as i32);
                for i in (7..64).rev() {
                    if x + i >= 63 {
                        assert_eq!(r & 1 << i != 0, canonical_grid[y][x + i - 63]);
                    } else {
                        assert_eq!(r & 1 << i != 0, true);
                        break;
                    }
                }
            }
        }
    }
}
