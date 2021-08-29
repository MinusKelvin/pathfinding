use std::f64::consts::SQRT_2;

use enumset::EnumSetType;

#[derive(Debug, EnumSetType)]
pub enum Direction {
    NorthWest,
    North,
    NorthEast,
    West,
    East,
    SouthWest,
    South,
    SouthEast,
}

#[derive(Copy, Clone, Debug)]
pub struct Neighborhood<T> {
    pub nw: T,
    pub n: T,
    pub ne: T,
    pub w: T,
    pub c: T,
    pub e: T,
    pub sw: T,
    pub s: T,
    pub se: T,
}

impl<T> Neighborhood<T> {
    /// Rotate clockwise 90 degrees
    pub fn rotate_cw(self) -> Self {
        Neighborhood {
            c: self.c,
            ne: self.nw,
            e: self.n,
            se: self.ne,
            s: self.e,
            sw: self.se,
            w: self.s,
            nw: self.sw,
            n: self.w,
        }
    }

    /// Flip across north-south
    pub fn flip_ortho(self) -> Self {
        Neighborhood {
            n: self.n,
            c: self.c,
            s: self.s,
            ne: self.nw,
            e: self.w,
            se: self.sw,
            nw: self.ne,
            w: self.e,
            sw: self.se,
        }
    }

    /// Flip across southwest-northeast
    pub fn flip_diagonal(self) -> Self {
        Neighborhood {
            ne: self.ne,
            c: self.c,
            sw: self.sw,
            n: self.e,
            e: self.n,
            w: self.s,
            s: self.w,
            nw: self.se,
            se: self.nw,
        }
    }
}

impl<T: Copy> Neighborhood<&T> {
    pub fn copied(self) -> Neighborhood<T> {
        Neighborhood {
            nw: *self.nw,
            n: *self.n,
            ne: *self.ne,
            w: *self.w,
            c: *self.c,
            e: *self.e,
            sw: *self.sw,
            s: *self.s,
            se: *self.se,
        }
    }
}

pub trait Cost {
    fn cost(&self) -> Option<f64>;
}

macro_rules! nz_cost_impls {
    ($($t:ident),*) => {
        $(
            impl Cost for Option<std::num::$t> {
                fn cost(&self) -> Option<f64> {
                    self.map(|c| c.get() as f64)
                }
            }
        )*
    };
}

nz_cost_impls!(NonZeroU8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroUsize);

impl Cost for f64 {
    fn cost(&self) -> Option<f64> {
        self.is_finite().then(|| *self)
    }
}

impl Cost for f32 {
    fn cost(&self) -> Option<f64> {
        self.is_finite().then(|| *self as f64)
    }
}

pub fn octile_heuristic((tx, ty): (i32, i32), scale: f64) -> impl Fn((i32, i32)) -> f64 {
    move |(x, y)| {
        let dx = (tx - x).abs();
        let dy = (ty - y).abs();
        let diagonal_moves = dx.min(dy);
        let ortho_moves = dx.max(dy) - dx.min(dy);
        (ortho_moves as f64 + SQRT_2 * diagonal_moves as f64) * scale
    }
}

pub fn manhattan_heuristic((tx, ty): (i32, i32), scale: f64) -> impl Fn((i32, i32)) -> f64 {
    move |(x, y)| {
        let dx = (tx - x).abs();
        let dy = (ty - y).abs();
        (dx + dy) as f64 * scale
    }
}

pub fn zero_heuristic<VertexId>() -> impl Fn(VertexId) -> f64 {
    |_| 0.0
}
