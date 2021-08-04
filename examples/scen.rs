use std::path::{Path, PathBuf};
use std::str::FromStr;

use pathfinding::algs::{AStarSearcher, DijkstraSearcher};
use pathfinding::domains::bitgrid;
use pathfinding::domains::BitGrid;
use pathfinding::traits::GValue;
use pathfinding::util::{GridPool, octile_heuristic};
use pathfinding::NodePool;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Options {
    #[structopt(short, long)]
    algorithm: Algorithm,
    map: PathBuf,
}

enum Algorithm {
    Dijkstra,
    AStar,
}

impl FromStr for Algorithm {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "dijkstra" => Algorithm::Dijkstra,
            "astar" => Algorithm::AStar,
            _ => return Err("invalid algorithm"),
        })
    }
}

pub fn main() {
    let options = Options::from_args();

    let instances = read_scenario(&options.map.with_extension("map.scen"));
    let map = read_map(&options.map);

    match options.algorithm {
        Algorithm::Dijkstra => {
            let mut searcher: DijkstraSearcher<(i32, i32), _, _> = DijkstraSearcher::new(
                GridPool::new(map.width(), map.height()),
                bitgrid::NoCornerCutting::new(&map),
            );

            let t = std::time::Instant::now();
            for instance in instances {
                let t = std::time::Instant::now();
                searcher.search(instance.from, Some(instance.to));
                println!(
                    "{:?} -> {:?}: {:.2?}",
                    instance.from,
                    instance.to,
                    t.elapsed()
                );
                let len = searcher.node_pool.generate(instance.to).get_g();
                assert_eq!(
                    (len * 1_000.0).round() as i64,
                    (instance.expected_length * 1_000.0).round() as i64
                );
            }
            eprintln!("Total: {:.2?}", t.elapsed());
        }
        Algorithm::AStar => {
            let mut searcher: AStarSearcher<(i32, i32), _, _, _> = AStarSearcher::new(
                GridPool::new(map.width(), map.height()),
                bitgrid::NoCornerCutting::new(&map),
                octile_heuristic((0, 0), 1.0)
            );

            let t = std::time::Instant::now();
            for instance in instances {
                searcher.edge_nav.heuristic = octile_heuristic(instance.to, 1.0);
                let t = std::time::Instant::now();
                searcher.search(instance.from, Some(instance.to));
                println!(
                    "{:?} -> {:?}: {:.2?}",
                    instance.from,
                    instance.to,
                    t.elapsed()
                );
                let len = searcher.node_pool.generate(instance.to).get_g();
                assert_eq!(
                    (len * 1_000.0).round() as i64,
                    (instance.expected_length * 1_000.0).round() as i64
                );
            }
            eprintln!("Total: {:.2?}", t.elapsed());
        }
    }
}

struct Instance {
    from: (i32, i32),
    to: (i32, i32),
    expected_length: f64,
}

fn read_scenario(path: &Path) -> Vec<Instance> {
    let content = std::fs::read_to_string(path).unwrap();
    let mut lines = content.lines();
    lines.next();
    let mut instances = vec![];
    for line in lines {
        let mut fields = line.split_whitespace();
        fields.next(); // bucket, whatever that means
        fields.next(); // map, should sanity check
        fields.next(); // map width, should sanity check
        fields.next(); // map height, should sanity check
        let sx = fields.next().unwrap().parse().unwrap();
        let sy = fields.next().unwrap().parse().unwrap();
        let tx = fields.next().unwrap().parse().unwrap();
        let ty = fields.next().unwrap().parse().unwrap();
        let len = fields.next().unwrap().parse().unwrap();
        instances.push(Instance {
            from: (sx, sy),
            to: (tx, ty),
            expected_length: len,
        });
    }
    instances
}

fn read_map(path: &Path) -> BitGrid {
    let content = std::fs::read_to_string(path).unwrap();
    let mut lines = content.lines();
    assert_eq!(lines.next(), Some("type octile"));
    let height = lines
        .next()
        .unwrap()
        .strip_prefix("height ")
        .unwrap()
        .parse()
        .unwrap();
    let width = lines
        .next()
        .unwrap()
        .strip_prefix("width ")
        .unwrap()
        .parse()
        .unwrap();
    assert_eq!(lines.next(), Some("map"));
    let mut grid = BitGrid::new(width, height);
    for (y, row) in lines.enumerate() {
        for (x, c) in row.chars().enumerate() {
            if let '@' | 'O' | 'T' = c {
                grid.set(x as i32, y as i32, true);
            }
        }
    }
    grid
}
