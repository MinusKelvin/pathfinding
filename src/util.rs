use std::f64::consts::SQRT_2;

use enumset::EnumSetType;

use crate::NodePool;
use crate::domains::WeightedGrid;
use crate::traits::{Cost, Destination};

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
pub struct GridEdge {
    pub direction: Direction,
    pub destination: (i32, i32),
    pub cost: f64,
}

impl Destination<(i32, i32)> for GridEdge {
    fn destination(&self) -> (i32, i32) {
        self.destination
    }
}

impl Cost for GridEdge {
    fn cost(&self) -> f64 {
        self.cost
    }
}

pub struct GridPool<N> {
    search_num: usize,
    grid: WeightedGrid<(usize, N)>
}

impl<N: Default> GridPool<N> {
    pub fn new(width: i32, height: i32) -> Self {
        GridPool {
            search_num: 0,
            grid: WeightedGrid::new(width, height, |_, _| (0, N::default()))
        }
    }
}

impl<N: Default> NodePool<(i32, i32), N> for GridPool<N> {
    fn reset(&mut self) {
        self.search_num += 1;
    }

    fn generate(&mut self, (x, y): (i32, i32)) -> &mut N {
        let (sn, node) = self.grid.get_mut(x, y);
        if *sn != self.search_num {
            *sn = self.search_num;
            *node = N::default();
        }
        node
    }

    fn get(&self, (x, y): (i32, i32)) -> Option<&N> {
        let &(sn, ref node) = self.grid.get(x, y);
        Some(node)
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
