use crate::util::{Direction, GridEdge};
use crate::EdgeGenerator;

use super::BitGrid;

pub struct OrthoEdges<'a> {
    map: &'a BitGrid,
    successors: Vec<GridEdge>,
}

impl<'a> OrthoEdges<'a> {
    pub fn new(map: &'a BitGrid) -> Self {
        OrthoEdges {
            map,
            successors: Vec::with_capacity(4),
        }
    }
}

impl<N> EdgeGenerator<(i32, i32), N> for OrthoEdges<'_> {
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
        &self.successors
    }
}
