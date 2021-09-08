use std::f64::consts::SQRT_2;

use enumset::EnumSet;

use crate::domains::BitGrid;
use crate::node_pool::GridPool;
use crate::util::{Direction, GridDomain};
use crate::{astar_unchecked, Edge, ExpansionPolicy, Owner, SearchNode};

pub fn create_tmap(map: &BitGrid) -> BitGrid {
    let mut tmap = BitGrid::new(map.height(), map.width());
    for x in 0..map.width() {
        for y in 0..map.height() {
            tmap.set(y, x, map.get(x, y));
        }
    }
    tmap
}

pub struct JpsExpansionPolicy<'a> {
    map: &'a BitGrid,
    tmap: &'a BitGrid,
    goal: (i32, i32),
}

impl<'a> JpsExpansionPolicy<'a> {
    pub fn new(map: &'a BitGrid, tmap: &'a BitGrid) -> Self {
        // SAFETY: While tmap is supposed to be a transposed copy of the map, our safety
        //         requirements are less strict - tmap need only have transposed width and height.
        assert_eq!(map.width(), tmap.height());
        assert_eq!(map.height(), tmap.width());
        JpsExpansionPolicy {
            map,
            tmap,
            goal: (-1, -1),
        }
    }

    pub fn set_goal(&mut self, new_goal: (i32, i32)) {
        self.goal = new_goal;
    }

    pub fn search(
        &mut self,
        pool: &mut GridPool,
        owner: &mut Owner,
        h: impl FnMut((i32, i32)) -> f64,
        source: (i32, i32),
        goal: (i32, i32),
    ) {
        assert!(pool.width() >= self.map.width());
        assert!(pool.height() >= self.map.height());
        self.map.get_neighbors(source.0, source.1);
        self.goal = goal;
        unsafe {
            // SAFETY: We check that the pool is large enough for our map.
            //         Our implementation never produces edges to cells that are out-of-bounds.
            //         We check that the source cell is in-bounds.
            astar_unchecked(pool, owner, self, h, source, goal)
        }
    }
}

unsafe impl GridDomain for JpsExpansionPolicy<'_> {
    fn width(&self) -> i32 {
        self.map.width()
    }

    fn height(&self) -> i32 {
        self.map.height()
    }
}

impl ExpansionPolicy<(i32, i32)> for JpsExpansionPolicy<'_> {
    unsafe fn expand_unchecked(
        &mut self,
        node: &SearchNode<(i32, i32)>,
        edges: &mut Vec<Edge<(i32, i32)>>,
    ) {
        let &mut JpsExpansionPolicy {
            map,
            tmap,
            goal: (goal_x, goal_y),
        } = self;
        let successors = canonical_successors(map, node.id, get_direction(node.id, node.parent));
        // SAFETY: The caller is responsible for upholding the requirement that the node id is
        //         in-bounds of the map.
        if successors.contains(Direction::East) {
            if let Ok(d) = jump_plus_unchecked(map, node.id.0, node.id.1, goal_x, goal_y) {
                edges.push(Edge {
                    destination: (node.id.0 + d, node.id.1),
                    cost: d as f64,
                });
            }
        }
        if successors.contains(Direction::South) {
            if let Ok(d) = jump_plus_unchecked(tmap, node.id.1, node.id.0, goal_y, goal_x) {
                edges.push(Edge {
                    destination: (node.id.0, node.id.1 + d),
                    cost: d as f64,
                });
            }
        }
        if successors.contains(Direction::West) {
            if let Ok(d) = jump_minus_unchecked(map, node.id.0, node.id.1, goal_x, goal_y) {
                edges.push(Edge {
                    destination: (node.id.0 - d, node.id.1),
                    cost: d as f64,
                });
            }
        }
        if successors.contains(Direction::North) {
            if let Ok(d) = jump_minus_unchecked(tmap, node.id.1, node.id.0, goal_y, goal_x) {
                edges.push(Edge {
                    destination: (node.id.0, node.id.1 - d),
                    cost: d as f64,
                });
            }
        }
        // SAFETY: During construction of self, we check that tmap's dimensions are the transpose
        //         of map's dimension.
        if successors.contains(Direction::NorthWest) {
            if let Some(d) =
                jump_northwest_unchecked(map, tmap, node.id.0, node.id.1, goal_x, goal_y)
            {
                edges.push(Edge {
                    destination: (node.id.0 - d, node.id.1 - d),
                    cost: SQRT_2 * d as f64,
                });
            }
        }
        if successors.contains(Direction::NorthEast) {
            if let Some(d) =
                jump_northeast_unchecked(map, tmap, node.id.0, node.id.1, goal_x, goal_y)
            {
                edges.push(Edge {
                    destination: (node.id.0 + d, node.id.1 - d),
                    cost: SQRT_2 * d as f64,
                });
            }
        }
        if successors.contains(Direction::SouthWest) {
            if let Some(d) =
                jump_southwest_unchecked(map, tmap, node.id.0, node.id.1, goal_x, goal_y)
            {
                edges.push(Edge {
                    destination: (node.id.0 - d, node.id.1 + d),
                    cost: SQRT_2 * d as f64,
                });
            }
        }
        if successors.contains(Direction::SouthEast) {
            if let Some(d) =
                jump_southeast_unchecked(map, tmap, node.id.0, node.id.1, goal_x, goal_y)
            {
                edges.push(Edge {
                    destination: (node.id.0 + d, node.id.1 + d),
                    cost: SQRT_2 * d as f64,
                });
            }
        }
    }

    fn expand(&mut self, node: &SearchNode<(i32, i32)>, edges: &mut Vec<Edge<(i32, i32)>>) {
        self.map.get_neighbors(node.id.0, node.id.1);
        unsafe {
            // SAFETY: The above get_neighbors call does the relevant bounds check for us.
            self.expand_unchecked(node, edges)
        }
    }
}

/// SAFETY: x and y must be in-bounds of the map.
#[inline(always)]
unsafe fn jump_plus_unchecked(
    map: &BitGrid,
    x: i32,
    y: i32,
    goal_x: i32,
    goal_y: i32,
) -> Result<i32, bool> {
    let mut distance = 0;
    loop {
        // SAFETY: Since y is in-bounds of the map and get_row_unchecked has 1 cell padding, the
        //         y parameter is in-bounds.
        // SAFETY: Since we stop jumping when we see the first 1 bit and the map is padded with 1s,
        //         x + distance will never go off the right side of the map and will be in-bounds.
        let bits_above = map.get_row_unchecked(x + distance, y - 1);
        let bits = map.get_row_unchecked(x + distance, y);
        let bits_below = map.get_row_unchecked(x + distance, y + 1);

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

/// SAFETY: x and y must be in-bounds of the map.
#[inline(always)]
unsafe fn jump_minus_unchecked(
    map: &BitGrid,
    x: i32,
    y: i32,
    goal_x: i32,
    goal_y: i32,
) -> Result<i32, bool> {
    let mut distance = 0;
    loop {
        // SAFETY: Since y is in-bounds of the map and get_row_upper_unchecked has 1 cell padding,
        //         the y parameter is in-bounds.
        // SAFETY: Since we stop jumping when we see the first 1 bit and the map is padded with 1s,
        //         x - distance will never go off the left side of the map and will be in-bounds.
        let bits_above = map.get_row_upper_unchecked(x - distance, y - 1);
        let bits = map.get_row_upper_unchecked(x - distance, y);
        let bits_below = map.get_row_upper_unchecked(x - distance, y + 1);

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

/// SAFETY: x and y must be in-bounds of the map, and tmap's dimensions must be transpose of map.
#[inline(always)]
unsafe fn jump_northwest_unchecked(
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
        // SAFETY: Since x and y are in-bounds of the map and we stop when we get to an obstruction
        //         (e.g. the padding 1s around the map), x - distance and y - distance will always
        //         be in-bounds.
        match jump_minus_unchecked(map, x - distance, y - distance, goal_x, goal_y) {
            Ok(_) => return Some(distance),
            Err(d) => done |= d,
        }
        match jump_minus_unchecked(tmap, y - distance, x - distance, goal_y, goal_x) {
            Ok(_) => return Some(distance),
            Err(d) => done |= d,
        }
        if done {
            return None;
        }
    }
}

/// SAFETY: x and y must be in-bounds of the map, and tmap's dimensions must be transpose of map.
#[inline(always)]
unsafe fn jump_northeast_unchecked(
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
        // SAFETY: Since x and y are in-bounds of the map and we stop when we get to an obstruction
        //         (e.g. the padding 1s around the map), x + distance and y - distance will always
        //         be in-bounds.
        match jump_plus_unchecked(map, x + distance, y - distance, goal_x, goal_y) {
            Ok(_) => return Some(distance),
            Err(d) => done |= d,
        }
        match jump_minus_unchecked(tmap, y - distance, x + distance, goal_y, goal_x) {
            Ok(_) => return Some(distance),
            Err(d) => done |= d,
        }
        if done {
            return None;
        }
    }
}

/// SAFETY: x and y must be in-bounds of the map, and tmap's dimensions must be transpose of map.
#[inline(always)]
unsafe fn jump_southwest_unchecked(
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
        // SAFETY: Since x and y are in-bounds of the map and we stop when we get to an obstruction
        //         (e.g. the padding 1s around the map), x - distance and y + distance will always
        //         be in-bounds.
        match jump_minus_unchecked(map, x - distance, y + distance, goal_x, goal_y) {
            Ok(_) => return Some(distance),
            Err(d) => done |= d,
        }
        match jump_plus_unchecked(tmap, y + distance, x - distance, goal_y, goal_x) {
            Ok(_) => return Some(distance),
            Err(d) => done |= d,
        }
        if done {
            return None;
        }
    }
}

/// SAFETY: x and y must be in-bounds of the map, and tmap's dimensions must be transpose of map.
#[inline(always)]
unsafe fn jump_southeast_unchecked(
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
        // SAFETY: Since x and y are in-bounds of the map and we stop when we get to an obstruction
        //         (e.g. the padding 1s around the map), x + distance and y + distance will always
        //         be in-bounds.
        match jump_plus_unchecked(map, x + distance, y + distance, goal_x, goal_y) {
            Ok(_) => return Some(distance),
            Err(d) => done |= d,
        }
        match jump_plus_unchecked(tmap, y + distance, x + distance, goal_y, goal_x) {
            Ok(_) => return Some(distance),
            Err(d) => done |= d,
        }
        if done {
            return None;
        }
    }
}

fn get_direction((x, y): (i32, i32), parent: Option<(i32, i32)>) -> Option<Direction> {
    parent.map(|(px, py)| match y.cmp(&py) {
        std::cmp::Ordering::Less => match x.cmp(&px) {
            std::cmp::Ordering::Less => Direction::NorthWest,
            std::cmp::Ordering::Equal => Direction::North,
            std::cmp::Ordering::Greater => Direction::NorthEast,
        },
        std::cmp::Ordering::Equal => match x.cmp(&px) {
            std::cmp::Ordering::Less => Direction::West,
            std::cmp::Ordering::Equal => unreachable!(),
            std::cmp::Ordering::Greater => Direction::East,
        },
        std::cmp::Ordering::Greater => match x.cmp(&px) {
            std::cmp::Ordering::Less => Direction::SouthWest,
            std::cmp::Ordering::Equal => Direction::South,
            std::cmp::Ordering::Greater => Direction::SouthEast,
        },
    })
}

fn canonical_successors(
    map: &BitGrid,
    (x, y): (i32, i32),
    dir: Option<Direction>,
) -> EnumSet<Direction> {
    let nbs = map.get_neighbors(x, y);
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
