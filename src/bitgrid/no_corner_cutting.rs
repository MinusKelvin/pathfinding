use std::f64::consts::SQRT_2;

use crate::util::Direction;
use crate::{Edge, SearchNode};

use super::BitGrid;

pub fn no_corner_cutting(
    map: &BitGrid,
) -> impl Fn(&SearchNode<(i32, i32)>, &mut Vec<Edge<(i32, i32)>>) + '_ {
    move |node, edges| {
        let nbs = map.get_neighbors(node.id.0, node.id.1);
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
