use std::f64::consts::SQRT_2;

use enumset::EnumSetType;

use crate::expansion_policy::ExpansionPolicy;
use crate::node_pool::NodePool;
use crate::{astar_unchecked, Owner};

/// Indicates that the implementing type guarantees the following invariants:
///
/// If `Self` is a `NodePool<(i32, i32)>`:
/// - All ids from `(0, 0)` inclusive to `(self.width(), self.height())` exclusive are in-bounds.
///
/// If `Self` is an `ExpansionPolicy<(i32, i32)>`:
/// - All ids from `(0, 0)` inclusive to `(self.width(), self.height())` exclusive are in-bounds.
/// - The ids of the destinations of all edges produced by `expand_unchecked` are in-bounds.
pub unsafe trait GridDomain {
    fn width(&self) -> i32;
    fn height(&self) -> i32;
}

pub fn grid_search<N, E>(
    pool: &mut N,
    owner: &mut Owner,
    expansion_policy: &mut E,
    h: impl FnMut((i32, i32)) -> f64,
    source: (i32, i32),
    goal: (i32, i32),
) where
    N: NodePool<(i32, i32)> + GridDomain,
    E: ExpansionPolicy<(i32, i32)> + GridDomain,
{
    assert!(pool.width() >= expansion_policy.width());
    assert!(pool.height() >= expansion_policy.height());
    assert!(source.0 >= 0 && source.0 < expansion_policy.width());
    assert!(source.1 >= 0 && source.1 < expansion_policy.height());
    unsafe {
        // SAFETY: We check that the pool is large enough for the expansion policy. The expansion
        //         policy guarantees that it never produces edges leading out-of-bounds. We check
        //         that the source vertex is in-bounds.
        astar_unchecked(pool, owner, expansion_policy, h, source, goal)
    }
}

/// Indicates that the implementing type guarantees the following invariants:
///
/// If `Self` is a `NodePool<usize>`:
/// - All ids from `0` inclusive to `self.len()` exclusive are in-bounds.
///
/// If `Self` is an `ExpansionPolicy<usize>`:
/// - All ids from `0` inclusive to `self.len()` exclusive are in-bounds.
/// - The ids of the destinations of all edges produced by `expand_unchecked` are in-bounds.
pub unsafe trait IndexDomain {
    fn len(&self) -> usize;
}

pub fn index_search<N, E>(
    pool: &mut N,
    owner: &mut Owner,
    expansion_policy: &mut E,
    h: impl FnMut(usize) -> f64,
    source: usize,
    goal: usize,
) where
    N: NodePool<usize> + IndexDomain,
    E: ExpansionPolicy<usize> + IndexDomain,
{
    assert!(pool.len() >= expansion_policy.len());
    assert!(source < expansion_policy.len());
    unsafe {
        // SAFETY: We check that the pool is large enough for the expansion policy. The expansion
        //         policy guarantees that it never produces edges leading out-of-bounds. We check
        //         that the source vertex is in-bounds.
        astar_unchecked(pool, owner, expansion_policy, h, source, goal)
    }
}

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
    fn cost(&self) -> f64;
}

macro_rules! nz_cost_impls {
    ($($t:ident),*) => {
        $(
            impl Cost for std::num::$t {
                fn cost(&self) -> f64 {
                    self.get() as f64
                }
            }
        )*
    };
}
nz_cost_impls!(NonZeroU8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroUsize);

macro_rules! prim_cost_impls {
    ($($t:ty),*) => {
        $(
            impl Cost for $t {
                fn cost(&self) -> f64 {
                    *self as f64
                }
            }
        )*
    };
}
prim_cost_impls!(u8, u16, u32, u64, usize, f32, f64, i8, i16, i32, i64, isize);

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
