use std::f64::consts::SQRT_2;

use crate::util::Direction;
use crate::{Edge, SearchNode};

use super::BitGrid;

pub fn no_corner_cutting(map: &BitGrid) -> impl Fn(&SearchNode, &mut Vec<Edge>) + '_ {
    move |node, edges| {
        let nbs = map.get_neighbors(node.x, node.y);
        if nbs.is_disjoint(Direction::North.into()) {
            edges.push(Edge {
                to_x: node.x,
                to_y: node.y - 1,
                cost: 1.0,
            });
        }
        if nbs.is_disjoint(Direction::South.into()) {
            edges.push(Edge {
                to_x: node.x,
                to_y: node.y + 1,
                cost: 1.0,
            });
        }
        if nbs.is_disjoint(Direction::West.into()) {
            edges.push(Edge {
                to_x: node.x - 1,
                to_y: node.y,
                cost: 1.0,
            });
        }
        if nbs.is_disjoint(Direction::East.into()) {
            edges.push(Edge {
                to_x: node.x + 1,
                to_y: node.y,
                cost: 1.0,
            });
        }
        if nbs.is_disjoint(Direction::North | Direction::West | Direction::NorthWest) {
            edges.push(Edge {
                to_x: node.x - 1,
                to_y: node.y - 1,
                cost: SQRT_2,
            });
        }
        if nbs.is_disjoint(Direction::North | Direction::East | Direction::NorthEast) {
            edges.push(Edge {
                to_x: node.x + 1,
                to_y: node.y - 1,
                cost: SQRT_2,
            });
        }
        if nbs.is_disjoint(Direction::South | Direction::West | Direction::SouthWest) {
            edges.push(Edge {
                to_x: node.x - 1,
                to_y: node.y + 1,
                cost: SQRT_2,
            });
        }
        if nbs.is_disjoint(Direction::South | Direction::East | Direction::SouthEast) {
            edges.push(Edge {
                to_x: node.x + 1,
                to_y: node.y + 1,
                cost: SQRT_2,
            });
        }
    }
}
