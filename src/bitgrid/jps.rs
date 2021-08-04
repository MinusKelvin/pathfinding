use std::f64::consts::SQRT_2;

use enumset::EnumSet;

use crate::util::Direction;
use crate::{Edge, SearchNode};

use super::BitGrid;

pub fn create_tmap(map: &BitGrid) -> BitGrid {
    let mut tmap = BitGrid::new(map.height(), map.width());
    for x in 0..map.width() {
        for y in 0..map.height() {
            tmap.set(y, x, map.get(x, y));
        }
    }
    tmap
}

pub fn jps<'a>(
    map: &'a BitGrid,
    tmap: &'a BitGrid,
    (goal_x, goal_y): (i32, i32),
) -> impl Fn(&SearchNode, &mut Vec<Edge>) + 'a {
    move |node, edges| {
        expand(map, tmap, goal_x, goal_y, node, edges);
    }
}

fn expand(
    map: &BitGrid,
    tmap: &BitGrid,
    goal_x: i32,
    goal_y: i32,
    node: &SearchNode,
    edges: &mut Vec<Edge>,
) {
    let successors = canonical_successors(map, node);
    if successors.contains(Direction::East) {
        if let Ok(d) = jump_plus(map, node.x, node.y, goal_x, goal_y) {
            edges.push(Edge {
                to_x: node.x + d,
                to_y: node.y,
                cost: d as f64,
            });
        }
    }
    if successors.contains(Direction::South) {
        if let Ok(d) = jump_plus(tmap, node.y, node.x, goal_y, goal_x) {
            edges.push(Edge {
                to_x: node.x,
                to_y: node.y + d,
                cost: d as f64,
            });
        }
    }
    if successors.contains(Direction::West) {
        if let Ok(d) = jump_minus(map, node.x, node.y, goal_x, goal_y) {
            edges.push(Edge {
                to_x: node.x - d,
                to_y: node.y,
                cost: d as f64,
            });
        }
    }
    if successors.contains(Direction::North) {
        if let Ok(d) = jump_minus(tmap, node.y, node.x, goal_y, goal_x) {
            edges.push(Edge {
                to_x: node.x,
                to_y: node.y - d,
                cost: d as f64,
            });
        }
    }
    if successors.contains(Direction::NorthWest) {
        if let Some(d) = jump_northwest(map, tmap, node.x, node.y, goal_x, goal_y) {
            edges.push(Edge {
                to_x: node.x - d,
                to_y: node.y - d,
                cost: SQRT_2 * d as f64,
            });
        }
    }
    if successors.contains(Direction::NorthEast) {
        if let Some(d) = jump_northeast(map, tmap, node.x, node.y, goal_x, goal_y) {
            edges.push(Edge {
                to_x: node.x + d,
                to_y: node.y - d,
                cost: SQRT_2 * d as f64,
            });
        }
    }
    if successors.contains(Direction::SouthWest) {
        if let Some(d) = jump_southwest(map, tmap, node.x, node.y, goal_x, goal_y) {
            edges.push(Edge {
                to_x: node.x - d,
                to_y: node.y + d,
                cost: SQRT_2 * d as f64,
            });
        }
    }
    if successors.contains(Direction::SouthEast) {
        if let Some(d) = jump_southeast(map, tmap, node.x, node.y, goal_x, goal_y) {
            edges.push(Edge {
                to_x: node.x + d,
                to_y: node.y + d,
                cost: SQRT_2 * d as f64,
            });
        }
    }
}

#[inline(always)]
fn jump_plus(map: &BitGrid, x: i32, y: i32, goal_x: i32, goal_y: i32) -> Result<i32, bool> {
    let mut distance = 0;
    loop {
        let bits_above = map.get_row(x + distance, y - 1);
        let bits = map.get_row(x + distance, y);
        let bits_below = map.get_row(x + distance, y + 1);

        let forced_above = (bits_above << 1) & !bits_above;
        let forced_below = (bits_below << 1) & !bits_below;
        let stop = (forced_above | forced_below | bits) & !0 >> 7;

        if stop != 0 {
            let stop = stop.trailing_zeros();
            distance += stop as i32;

            if y == goal_y && x <= goal_x && goal_x <= x + distance {
                return Ok(goal_x - x);
            }

            if bits & 1 << stop != 0 {
                return Err(distance <= 1);
            } else {
                return Ok(distance);
            }
        }

        distance += 56;
    }
}

#[inline(always)]
fn jump_minus(map: &BitGrid, x: i32, y: i32, goal_x: i32, goal_y: i32) -> Result<i32, bool> {
    let mut distance = 0;
    loop {
        let bits_above = map.get_row_upper(x - distance, y - 1);
        let bits = map.get_row_upper(x - distance, y);
        let bits_below = map.get_row_upper(x - distance, y + 1);

        let forced_above = (bits_above >> 1) & !bits_above;
        let forced_below = (bits_below >> 1) & !bits_below;
        let stop = (forced_above | forced_below | bits) & !0 << 7;

        if stop != 0 {
            let stop = stop.leading_zeros();
            distance += stop as i32;

            if y == goal_y && x - distance <= goal_x && goal_x <= x {
                return Ok(x - goal_x);
            }

            if bits & (1 << 63) >> stop != 0 {
                return Err(distance <= 1);
            } else {
                return Ok(distance);
            }
        }

        distance += 56;
    }
}

#[inline(always)]
fn jump_northwest(
    map: &BitGrid,
    tmap: &BitGrid,
    x: i32,
    y: i32,
    goal_x: i32,
    goal_y: i32,
) -> Option<i32> {
    let mut distance = 0;
    loop {
        distance += 1;

        let mut done = false;
        match jump_minus(map, x - distance, y - distance, goal_x, goal_y) {
            Ok(_) => return Some(distance),
            Err(d) => done |= d,
        }
        match jump_minus(tmap, y - distance, x - distance, goal_y, goal_x) {
            Ok(_) => return Some(distance),
            Err(d) => done |= d,
        }
        if done {
            return None;
        }
    }
}

#[inline(always)]
fn jump_northeast(
    map: &BitGrid,
    tmap: &BitGrid,
    x: i32,
    y: i32,
    goal_x: i32,
    goal_y: i32,
) -> Option<i32> {
    let mut distance = 0;
    loop {
        distance += 1;

        let mut done = false;
        match jump_plus(map, x + distance, y - distance, goal_x, goal_y) {
            Ok(_) => return Some(distance),
            Err(d) => done |= d,
        }
        match jump_minus(tmap, y - distance, x + distance, goal_y, goal_x) {
            Ok(_) => return Some(distance),
            Err(d) => done |= d,
        }
        if done {
            return None;
        }
    }
}

#[inline(always)]
fn jump_southwest(
    map: &BitGrid,
    tmap: &BitGrid,
    x: i32,
    y: i32,
    goal_x: i32,
    goal_y: i32,
) -> Option<i32> {
    let mut distance = 0;
    loop {
        distance += 1;

        let mut done = false;
        match jump_minus(map, x - distance, y + distance, goal_x, goal_y) {
            Ok(_) => return Some(distance),
            Err(d) => done |= d,
        }
        match jump_plus(tmap, y + distance, x - distance, goal_y, goal_x) {
            Ok(_) => return Some(distance),
            Err(d) => done |= d,
        }
        if done {
            return None;
        }
    }
}

#[inline(always)]
fn jump_southeast(
    map: &BitGrid,
    tmap: &BitGrid,
    x: i32,
    y: i32,
    goal_x: i32,
    goal_y: i32,
) -> Option<i32> {
    let mut distance = 0;
    loop {
        distance += 1;

        let mut done = false;
        match jump_plus(map, x + distance, y + distance, goal_x, goal_y) {
            Ok(_) => return Some(distance),
            Err(d) => done |= d,
        }
        match jump_plus(tmap, y + distance, x + distance, goal_y, goal_x) {
            Ok(_) => return Some(distance),
            Err(d) => done |= d,
        }
        if done {
            return None;
        }
    }
}

fn canonical_successors(map: &BitGrid, node: &SearchNode) -> EnumSet<Direction> {
    let nbs = map.get_neighbors(node.x, node.y);
    let dir = node.parent.map(|(px, py)| match node.y.cmp(&py) {
        std::cmp::Ordering::Less => match node.x.cmp(&px) {
            std::cmp::Ordering::Less => Direction::NorthWest,
            std::cmp::Ordering::Equal => Direction::North,
            std::cmp::Ordering::Greater => Direction::NorthEast,
        },
        std::cmp::Ordering::Equal => match node.x.cmp(&px) {
            std::cmp::Ordering::Less => Direction::West,
            std::cmp::Ordering::Equal => unreachable!(),
            std::cmp::Ordering::Greater => Direction::East,
        },
        std::cmp::Ordering::Greater => match node.x.cmp(&px) {
            std::cmp::Ordering::Less => Direction::SouthWest,
            std::cmp::Ordering::Equal => Direction::South,
            std::cmp::Ordering::Greater => Direction::SouthEast,
        },
    });
    let mut canonical_successors = EnumSet::empty();
    match dir {
        None => {
            if nbs.is_disjoint(Direction::North.into()) {
                canonical_successors |= Direction::North;
            }
            if nbs.is_disjoint(Direction::South.into()) {
                canonical_successors |= Direction::South;
            }
            if nbs.is_disjoint(Direction::West.into()) {
                canonical_successors |= Direction::West;
            }
            if nbs.is_disjoint(Direction::East.into()) {
                canonical_successors |= Direction::East;
            }
            if nbs.is_disjoint(Direction::North | Direction::West | Direction::NorthWest) {
                canonical_successors |= Direction::NorthWest;
            }
            if nbs.is_disjoint(Direction::North | Direction::East | Direction::NorthEast) {
                canonical_successors |= Direction::NorthEast;
            }
            if nbs.is_disjoint(Direction::South | Direction::West | Direction::SouthWest) {
                canonical_successors |= Direction::SouthWest;
            }
            if nbs.is_disjoint(Direction::South | Direction::East | Direction::SouthEast) {
                canonical_successors |= Direction::SouthEast;
            }
        }
        Some(Direction::NorthWest) => {
            if !nbs.contains(Direction::North) {
                canonical_successors |= Direction::North;
            }
            if !nbs.contains(Direction::West) {
                canonical_successors |= Direction::West;
            }
            if nbs.is_disjoint(Direction::North | Direction::West | Direction::NorthWest) {
                canonical_successors |= Direction::NorthWest;
            }
        }
        Some(Direction::NorthEast) => {
            if !nbs.contains(Direction::North) {
                canonical_successors |= Direction::North;
            }
            if !nbs.contains(Direction::East) {
                canonical_successors |= Direction::East;
            }
            if nbs.is_disjoint(Direction::North | Direction::East | Direction::NorthEast) {
                canonical_successors |= Direction::NorthEast;
            }
        }
        Some(Direction::SouthWest) => {
            if !nbs.contains(Direction::South) {
                canonical_successors |= Direction::South;
            }
            if !nbs.contains(Direction::West) {
                canonical_successors |= Direction::West;
            }
            if nbs.is_disjoint(Direction::South | Direction::West | Direction::SouthWest) {
                canonical_successors |= Direction::SouthWest;
            }
        }
        Some(Direction::SouthEast) => {
            if !nbs.contains(Direction::South) {
                canonical_successors |= Direction::South;
            }
            if !nbs.contains(Direction::East) {
                canonical_successors |= Direction::East;
            }
            if nbs.is_disjoint(Direction::South | Direction::East | Direction::SouthEast) {
                canonical_successors |= Direction::SouthEast;
            }
        }
        Some(Direction::North) => {
            if !nbs.contains(Direction::North) {
                canonical_successors |= Direction::North;
            }
            if nbs.contains(Direction::SouthWest) && !nbs.contains(Direction::West) {
                canonical_successors |= Direction::West;
                if nbs.is_disjoint(Direction::NorthWest | Direction::North) {
                    canonical_successors |= Direction::NorthWest;
                }
            }
            if nbs.contains(Direction::SouthEast) && !nbs.contains(Direction::East) {
                canonical_successors |= Direction::East;
                if nbs.is_disjoint(Direction::NorthEast | Direction::North) {
                    canonical_successors |= Direction::NorthEast;
                }
            }
        }
        Some(Direction::West) => {
            if !nbs.contains(Direction::West) {
                canonical_successors |= Direction::West;
            }
            if nbs.contains(Direction::SouthEast) && !nbs.contains(Direction::South) {
                canonical_successors |= Direction::South;
                if nbs.is_disjoint(Direction::West | Direction::SouthWest) {
                    canonical_successors |= Direction::SouthWest;
                }
            }
            if nbs.contains(Direction::NorthEast) && !nbs.contains(Direction::North) {
                canonical_successors |= Direction::North;
                if nbs.is_disjoint(Direction::West | Direction::NorthWest) {
                    canonical_successors |= Direction::NorthWest;
                }
            }
        }
        Some(Direction::South) => {
            if !nbs.contains(Direction::South) {
                canonical_successors |= Direction::South;
            }
            if nbs.contains(Direction::NorthEast) && !nbs.contains(Direction::East) {
                canonical_successors |= Direction::East;
                if nbs.is_disjoint(Direction::SouthEast | Direction::South) {
                    canonical_successors |= Direction::SouthEast;
                }
            }
            if nbs.contains(Direction::NorthWest) && !nbs.contains(Direction::West) {
                canonical_successors |= Direction::West;
                if nbs.is_disjoint(Direction::SouthWest | Direction::South) {
                    canonical_successors |= Direction::SouthWest;
                }
            }
        }
        Some(Direction::East) => {
            if !nbs.contains(Direction::East) {
                canonical_successors |= Direction::East;
            }
            if nbs.contains(Direction::NorthWest) && !nbs.contains(Direction::North) {
                canonical_successors |= Direction::North;
                if nbs.is_disjoint(Direction::East | Direction::NorthEast) {
                    canonical_successors |= Direction::NorthEast;
                }
            }
            if nbs.contains(Direction::SouthWest) && !nbs.contains(Direction::South) {
                canonical_successors |= Direction::South;
                if nbs.is_disjoint(Direction::East | Direction::SouthEast) {
                    canonical_successors |= Direction::SouthEast;
                }
            }
        }
    };
    canonical_successors
}
