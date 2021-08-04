use std::f64::consts::SQRT_2;

use enumset::EnumSetType;
use qcell::{LCell, LCellOwner};

use crate::weighted_grid::WeightedGrid;
use crate::SearchNode;

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

pub struct GridPool<'id> {
    search_num: usize,
    grid: WeightedGrid<LCell<'id, SearchNode>>,
}

impl<'id> GridPool<'id> {
    pub fn new(width: i32, height: i32) -> Self {
        GridPool {
            search_num: 0,
            grid: WeightedGrid::new(width, height, |x, y| {
                LCell::new(SearchNode {
                    search_num: 0,
                    expansions: 0,
                    pqueue_location: 0,
                    x,
                    y,
                    parent: None,
                    g: 0.0,
                    lb: 0.0,
                })
            }),
        }
    }

    pub fn reset(&mut self) {
        self.search_num += 1;
    }

    pub fn get(&self, x: i32, y: i32, owner: &LCellOwner<'id>) -> Option<&LCell<'id, SearchNode>> {
        let cell = self.grid.get(x, y);
        if owner.ro(cell).search_num == self.search_num {
            Some(cell)
        } else {
            None
        }
    }

    pub fn get_mut(&self, x: i32, y: i32, owner: &mut LCellOwner<'id>) -> &LCell<'id, SearchNode> {
        let cell = self.grid.get(x, y);
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

pub fn octile_heuristic((tx, ty): (i32, i32), scale: f64) -> impl Fn(i32, i32) -> f64 {
    move |x, y| {
        let dx = (tx - x).abs();
        let dy = (ty - y).abs();
        let diagonal_moves = dx.min(dy);
        let ortho_moves = dx.max(dy) - dx.min(dy);
        (ortho_moves as f64 + SQRT_2 * diagonal_moves as f64) * scale
    }
}

pub fn manhattan_heuristic((tx, ty): (i32, i32), scale: f64) -> impl Fn(i32, i32) -> f64 {
    move |x, y| {
        let dx = (tx - x).abs();
        let dy = (ty - y).abs();
        (dx + dy) as f64 * scale
    }
}

pub fn zero_heuristic() -> impl Fn(i32, i32) -> f64 {
    |_, _| 0.0
}
