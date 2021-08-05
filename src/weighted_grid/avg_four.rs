use std::f64::consts::SQRT_2;

use crate::{Edge, SearchNode};

use super::{Cost, Neighborhood, WeightedGrid};

pub fn avg_four<T: Cost>(map: &WeightedGrid<T>) -> impl Fn(&SearchNode, &mut Vec<Edge>) + '_ {
    move |node, edges| {
        let neighborhood = map.get_neighborhood(node.x, node.y);
        let c = neighborhood.c.cost().unwrap();
        if let Some(cost) = neighborhood.n.cost() {
            edges.push(Edge {
                to_x: node.x,
                to_y: node.y - 1,
                cost: (cost + c) / 2.0,
            });
        }
        if let Some(cost) = neighborhood.s.cost() {
            edges.push(Edge {
                to_x: node.x,
                to_y: node.y + 1,
                cost: (cost + c) / 2.0,
            });
        }
        if let Some(cost) = neighborhood.w.cost() {
            edges.push(Edge {
                to_x: node.x - 1,
                to_y: node.y,
                cost: (cost + c) / 2.0,
            });
        }
        if let Some(cost) = neighborhood.e.cost() {
            edges.push(Edge {
                to_x: node.x + 1,
                to_y: node.y,
                cost: (cost + c) / 2.0,
            });
        }
        if let Some(cost) = neighborhood.nw_cost() {
            edges.push(Edge {
                to_x: node.x - 1,
                to_y: node.y - 1,
                cost,
            });
        }
        if let Some(cost) = neighborhood.ne_cost() {
            edges.push(Edge {
                to_x: node.x + 1,
                to_y: node.y - 1,
                cost,
            });
        }
        if let Some(cost) = neighborhood.sw_cost() {
            edges.push(Edge {
                to_x: node.x - 1,
                to_y: node.y + 1,
                cost,
            });
        }
        if let Some(cost) = neighborhood.se_cost() {
            edges.push(Edge {
                to_x: node.x + 1,
                to_y: node.y + 1,
                cost,
            });
        }
    }
}

impl<T: Cost> Neighborhood<&T> {
    fn nw_cost(&self) -> Option<f64> {
        let c = self.c.cost()?;
        let n = self.n.cost()?;
        let w = self.w.cost()?;
        let nw = self.nw.cost()?;
        Some((c + n + w + nw) * SQRT_2 / 4.0)
    }

    fn sw_cost(&self) -> Option<f64> {
        self.rotate_cw().nw_cost()
    }

    fn se_cost(&self) -> Option<f64> {
        self.rotate_cw().sw_cost()
    }

    fn ne_cost(&self) -> Option<f64> {
        self.rotate_cw().se_cost()
    }
}
