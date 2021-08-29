use std::f64::consts::SQRT_2;

use crate::domains::WeightedGrid;
use crate::util::{Cost, GridDomain, Neighborhood};
use crate::{Edge, ExpansionPolicy, SearchNode};

pub struct AverageOfFour<'a, T>(&'a WeightedGrid<T>);

impl<T> AverageOfFour<'_, T> {
    pub fn new(map: &WeightedGrid<T>) -> AverageOfFour<T> {
        AverageOfFour(map)
    }
}

unsafe impl<T> GridDomain for AverageOfFour<'_, T> {
    fn width(&self) -> i32 {
        self.0.width()
    }

    fn height(&self) -> i32 {
        self.0.height()
    }
}

impl<T: Cost> ExpansionPolicy<(i32, i32)> for AverageOfFour<'_, T> {
    fn expand(&mut self, node: &SearchNode<(i32, i32)>, edges: &mut Vec<Edge<(i32, i32)>>) {
        self.0.get_neighborhood(node.id.0, node.id.1);
        unsafe {
            // SAFETY: bounds are checked by above call
            self.expand_unchecked(node, edges)
        }
    }

    unsafe fn expand_unchecked(
        &mut self,
        node: &SearchNode<(i32, i32)>,
        edges: &mut Vec<Edge<(i32, i32)>>,
    ) {
        let &mut AverageOfFour(map) = self;
        let neighborhood = map.get_neighborhood_unchecked(node.id.0, node.id.1);
        let c = neighborhood.c.cost().unwrap();
        if let Some(cost) = neighborhood.n.cost() {
            edges.push(Edge {
                destination: (node.id.0, node.id.1 - 1),
                cost: (cost + c) / 2.0,
            });
        }
        if let Some(cost) = neighborhood.s.cost() {
            edges.push(Edge {
                destination: (node.id.0, node.id.1 + 1),
                cost: (cost + c) / 2.0,
            });
        }
        if let Some(cost) = neighborhood.w.cost() {
            edges.push(Edge {
                destination: (node.id.0 - 1, node.id.1),
                cost: (cost + c) / 2.0,
            });
        }
        if let Some(cost) = neighborhood.e.cost() {
            edges.push(Edge {
                destination: (node.id.0 + 1, node.id.1),
                cost: (cost + c) / 2.0,
            });
        }
        if let Some(cost) = neighborhood.nw_cost() {
            edges.push(Edge {
                destination: (node.id.0 - 1, node.id.1 - 1),
                cost,
            });
        }
        if let Some(cost) = neighborhood.ne_cost() {
            edges.push(Edge {
                destination: (node.id.0 + 1, node.id.1 - 1),
                cost,
            });
        }
        if let Some(cost) = neighborhood.sw_cost() {
            edges.push(Edge {
                destination: (node.id.0 - 1, node.id.1 + 1),
                cost,
            });
        }
        if let Some(cost) = neighborhood.se_cost() {
            edges.push(Edge {
                destination: (node.id.0 + 1, node.id.1 + 1),
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
