use std::f64::consts::SQRT_2;

use crate::domains::BitGrid;
use crate::util::{Direction, GridDomain};
use crate::{Edge, ExpansionPolicy, SearchNode};

pub struct NoCornerCutting<'a>(&'a BitGrid);

impl NoCornerCutting<'_> {
    pub fn new(map: &BitGrid) -> NoCornerCutting {
        NoCornerCutting(map)
    }
}

unsafe impl GridDomain for NoCornerCutting<'_> {
    fn width(&self) -> i32 {
        self.0.width()
    }

    fn height(&self) -> i32 {
        self.0.height()
    }
}

impl ExpansionPolicy<(i32, i32)> for NoCornerCutting<'_> {
    fn expand(&mut self, node: &SearchNode<(i32, i32)>, edges: &mut Vec<Edge<(i32, i32)>>) {
        self.0.get_neighbors(node.id.0, node.id.1);
        unsafe {
            // SAFETY: Bounds checked by above call
            self.expand_unchecked(node, edges)
        }
    }

    unsafe fn expand_unchecked(
        &mut self,
        node: &SearchNode<(i32, i32)>,
        edges: &mut Vec<Edge<(i32, i32)>>,
    ) {
        let &mut Self(map) = self;
        let nbs = map.get_neighbors_unchecked(node.id.0, node.id.1);
        if nbs.is_disjoint(Direction::North.into()) {
            edges.push(Edge {
                destination: (node.id.0, node.id.1 - 1),
                cost: 1.0,
            });
        }
        if nbs.is_disjoint(Direction::South.into()) {
            edges.push(Edge {
                destination: (node.id.0, node.id.1 + 1),
                cost: 1.0,
            });
        }
        if nbs.is_disjoint(Direction::West.into()) {
            edges.push(Edge {
                destination: (node.id.0 - 1, node.id.1),
                cost: 1.0,
            });
        }
        if nbs.is_disjoint(Direction::East.into()) {
            edges.push(Edge {
                destination: (node.id.0 + 1, node.id.1),
                cost: 1.0,
            });
        }
        if nbs.is_disjoint(Direction::North | Direction::West | Direction::NorthWest) {
            edges.push(Edge {
                destination: (node.id.0 - 1, node.id.1 - 1),
                cost: SQRT_2,
            });
        }
        if nbs.is_disjoint(Direction::North | Direction::East | Direction::NorthEast) {
            edges.push(Edge {
                destination: (node.id.0 + 1, node.id.1 - 1),
                cost: SQRT_2,
            });
        }
        if nbs.is_disjoint(Direction::South | Direction::West | Direction::SouthWest) {
            edges.push(Edge {
                destination: (node.id.0 - 1, node.id.1 + 1),
                cost: SQRT_2,
            });
        }
        if nbs.is_disjoint(Direction::South | Direction::East | Direction::SouthEast) {
            edges.push(Edge {
                destination: (node.id.0 + 1, node.id.1 + 1),
                cost: SQRT_2,
            });
        }
    }
}
