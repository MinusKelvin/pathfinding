use std::f64::consts::SQRT_2;

use crate::util::{Direction, GridEdge};
use crate::EdgeGenerator;

use super::BitGrid;

pub struct NoCornerCutting<'a> {
    map: &'a BitGrid,
    successors: Vec<GridEdge>,
}

impl<'a> NoCornerCutting<'a> {
    pub fn new(map: &'a BitGrid) -> Self {
        NoCornerCutting {
            map,
            successors: Vec::with_capacity(8),
        }
    }
}

impl<N> EdgeGenerator<(i32, i32), N> for NoCornerCutting<'_> {
    type Edge = GridEdge;

    fn edges(
        &mut self,
        (x, y): (i32, i32),
        _node_pool: &mut impl crate::NodePool<(i32, i32), N>,
    ) -> &[GridEdge] {
        self.successors.clear();
        let obstructions = self.map.get_neighbors(x, y);
        if !obstructions.contains(Direction::North) {
            self.successors.push(GridEdge {
                direction: Direction::North,
                destination: (x, y - 1),
                cost: 1.0,
            });
        }
        if !obstructions.contains(Direction::West) {
            self.successors.push(GridEdge {
                direction: Direction::West,
                destination: (x - 1, y),
                cost: 1.0,
            });
        }
        if !obstructions.contains(Direction::East) {
            self.successors.push(GridEdge {
                direction: Direction::East,
                destination: (x + 1, y),
                cost: 1.0,
            });
        }
        if !obstructions.contains(Direction::South) {
            self.successors.push(GridEdge {
                direction: Direction::South,
                destination: (x, y + 1),
                cost: 1.0,
            });
        }
        if obstructions.is_disjoint(Direction::North | Direction::West | Direction::NorthWest) {
            self.successors.push(GridEdge {
                direction: Direction::NorthWest,
                destination: (x - 1, y - 1),
                cost: SQRT_2,
            });
        }
        if obstructions.is_disjoint(Direction::North | Direction::East | Direction::NorthEast) {
            self.successors.push(GridEdge {
                direction: Direction::NorthEast,
                destination: (x + 1, y - 1),
                cost: SQRT_2,
            });
        }
        if obstructions.is_disjoint(Direction::South | Direction::West | Direction::SouthWest) {
            self.successors.push(GridEdge {
                direction: Direction::SouthWest,
                destination: (x - 1, y + 1),
                cost: SQRT_2,
            });
        }
        if obstructions.is_disjoint(Direction::South | Direction::East | Direction::SouthEast) {
            self.successors.push(GridEdge {
                direction: Direction::SouthEast,
                destination: (x + 1, y + 1),
                cost: SQRT_2,
            });
        }
        &self.successors
    }
}
