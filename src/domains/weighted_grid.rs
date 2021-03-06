use crate::util::Neighborhood;

pub struct WeightedGrid<V> {
    width: i32,
    height: i32,
    cells: Box<[Option<V>]>,
}

impl<V> WeightedGrid<V> {
    /// Constructs a fully-obstructed weighted grid.
    pub fn new(width: i32, height: i32) -> Self {
        assert!(width > 0 && height > 0, "width and height must be positive");
        // there is 1 padding entry at the end of each row
        let padded_width = width as usize + 1;
        // there is a padding row above and a padding row below.
        let padded_height = height as usize + 2;
        // there is one extra entry so that the unpadded coordinate (width, height),
        // which is 1 cell out of bounds on each axis, can be dereferenced.
        let padded_size = padded_width * padded_height + 1;

        WeightedGrid {
            width,
            height,
            cells: std::iter::repeat_with(|| None).take(padded_size).collect(),
        }
    }

    #[inline(always)]
    pub fn width(&self) -> i32 {
        self.width
    }

    #[inline(always)]
    pub fn height(&self) -> i32 {
        self.height
    }

    #[inline(always)]
    pub fn get(&self, x: i32, y: i32) -> Option<&V> {
        self.padded_bounds_check(x, y);
        unsafe { self.get_unchecked(x, y) }
    }

    #[inline(always)]
    pub fn get_mut(&mut self, x: i32, y: i32) -> &mut Option<V> {
        self.unpadded_bounds_check(x, y);
        unsafe { self.get_unchecked_mut(x, y) }
    }

    #[inline(always)]
    pub fn get_neighborhood(&self, x: i32, y: i32) -> Neighborhood<Option<&V>> {
        self.unpadded_bounds_check(x, y);
        unsafe { self.get_neighborhood_unchecked(x, y) }
    }

    /// SAFETY: `x` must be in `-1..width+1`, `y` must be in `-1..height+1`.
    /// Padding cells can be relied upon being `None`.
    #[inline(always)]
    pub unsafe fn get_unchecked(&self, x: i32, y: i32) -> Option<&V> {
        self.cells.get_unchecked(self.locate(x, y)).as_ref()
    }

    /// SAFETY: `x` must be in `0..width`, `y` must be in `0..height`.
    /// Padding cells can be relied upon being `None`.
    #[inline(always)]
    pub unsafe fn get_unchecked_mut(&mut self, x: i32, y: i32) -> &mut Option<V> {
        self.cells.get_unchecked_mut(self.locate(x, y))
    }

    /// SAFETY: `x` must be in `0..width`, `y` must be in `0..height`.
    /// Padding cells can be relied upon being `None`.
    #[inline(always)]
    pub unsafe fn get_neighborhood_unchecked(&self, x: i32, y: i32) -> Neighborhood<Option<&V>> {
        #[cfg(debug_assertions)]
        self.unpadded_bounds_check(x, y);

        let idx = self.locate(x, y);
        let padded_width = self.width as usize - 1;
        Neighborhood {
            nw: self.cells.get_unchecked(idx - padded_width - 1).as_ref(),
            n: self.cells.get_unchecked(idx - padded_width).as_ref(),
            ne: self.cells.get_unchecked(idx - padded_width + 1).as_ref(),
            w: self.cells.get_unchecked(idx - 1).as_ref(),
            c: self.cells.get_unchecked(idx).as_ref(),
            e: self.cells.get_unchecked(idx + 1).as_ref(),
            sw: self.cells.get_unchecked(idx + padded_width - 1).as_ref(),
            s: self.cells.get_unchecked(idx + padded_width).as_ref(),
            se: self.cells.get_unchecked(idx + padded_width + 1).as_ref(),
        }
    }

    #[inline(always)]
    fn locate(&self, x: i32, y: i32) -> usize {
        #[cfg(debug_assertions)]
        self.padded_bounds_check(x, y);

        let padded_y = (y + 1) as usize;
        let padded_width = self.width as usize + 1;
        let padded_x = (x + 1) as usize;
        let id = padded_y * padded_width + padded_x;

        debug_assert!(id < padded_width * (self.height as usize + 2) + 1);

        id
    }

    #[track_caller]
    #[inline(always)]
    fn padded_bounds_check(&self, x: i32, y: i32) {
        if !(-1..self.width + 1).contains(&x) || !(-1..self.height + 1).contains(&y) {
            panic!("Grid cell ({}, {}) is out of bounds.", x, y);
        }
    }

    #[track_caller]
    #[inline(always)]
    fn unpadded_bounds_check(&self, x: i32, y: i32) {
        if !(0..self.width).contains(&x) || !(0..self.height).contains(&y) {
            panic!("Grid cell ({}, {}) is out of bounds.", x, y);
        }
    }
}

#[cfg(feature = "serde")]
mod serde {
    use std::marker::PhantomData;

    use serde::de::{Error, Visitor};
    use serde::ser::SerializeSeq;
    use serde::{Deserialize, Serialize};

    impl<V: Serialize> Serialize for super::WeightedGrid<V> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let cells = self.width as usize * self.height as usize;
            let mut s = serializer.serialize_seq(Some(cells + 2))?;
            s.serialize_element(&self.width)?;
            s.serialize_element(&self.height)?;
            for y in 0..self.height {
                for x in 0..self.width {
                    s.serialize_element(&self.get(x, y))?;
                }
            }
            s.end()
        }
    }

    impl<'de, V: Deserialize<'de>> Deserialize<'de> for super::WeightedGrid<V> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_seq(WeightedGridVisitor(PhantomData))
        }
    }

    struct WeightedGridVisitor<V>(PhantomData<V>);
    impl<'de, V: Deserialize<'de>> Visitor<'de> for WeightedGridVisitor<V> {
        type Value = super::WeightedGrid<V>;

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let width = seq
                .next_element()?
                .ok_or_else(|| Error::invalid_length(0, &self))?;
            let height = seq
                .next_element()?
                .ok_or_else(|| Error::invalid_length(1, &self))?;

            let mut grid = super::WeightedGrid::new(width, height);
            for y in 0..height {
                for x in 0..width {
                    let i = 2 + x as usize + y as usize * width as usize;
                    *grid.get_mut(x, y) = seq
                        .next_element()?
                        .ok_or_else(|| Error::invalid_length(i, &self))?;
                }
            }

            Ok(grid)
        }

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "a sequence of values")
        }
    }
}
