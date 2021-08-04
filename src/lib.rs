#![cfg_attr(feature = "nightly", feature(type_alias_impl_trait))]

use pqueue::PriorityQueue;
use qcell::LCellOwner;
use util::GridPool;

pub mod bitgrid;
pub mod pqueue;
pub mod util;
pub mod weighted_grid;

#[derive(Debug, Copy, Clone)]
pub struct SearchNode {
    search_num: usize,
    pqueue_location: usize,
    pub expansions: usize,
    pub x: i32,
    pub y: i32,
    pub parent: Option<(i32, i32)>,
    pub g: f64,
    pub lb: f64,
}

#[derive(Debug, Copy, Clone)]
pub struct Edge {
    to_x: i32,
    to_y: i32,
    cost: f64,
}

pub fn astar<'id>(
    pool: &mut GridPool<'id>,
    owner: &mut LCellOwner<'id>,
    mut expander: impl FnMut(&SearchNode, &mut Vec<Edge>),
    h: impl Fn(i32, i32) -> f64,
    src_x: i32,
    src_y: i32,
    goal_x: i32,
    goal_y: i32,
) {
    pool.reset();
    let mut queue = PriorityQueue::new();
    let mut edges = vec![];

    let source = pool.get_mut(src_x, src_y, owner);
    owner.rw(source).g = 0.0;
    owner.rw(source).lb = 0.0;

    queue.decrease_key(source, owner);

    while let Some(node) = queue.pop(owner) {
        let n = owner.rw(node);
        n.expansions += 1;
        if n.x == goal_x && n.y == goal_y {
            break;
        }

        expander(n, &mut edges);

        let parent_g = n.g;
        let parent_coords = (n.x, n.y);

        for edge in edges.drain(..) {
            let g = parent_g + edge.cost;
            let node = pool.get_mut(edge.to_x, edge.to_y, owner);
            let n = owner.rw(node);
            if g < n.g {
                n.g = g;
                n.lb = g + h(n.x, n.y);
                n.parent = Some(parent_coords);
                queue.decrease_key(node, owner);
            }
        }
    }
}
