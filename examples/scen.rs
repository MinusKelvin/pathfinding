use std::path::{Path, PathBuf};
use std::str::FromStr;

use pathfinding::bitgrid::{create_tmap, jps, no_corner_cutting, BitGrid};
use pathfinding::util::{octile_heuristic, zero_heuristic, GridPool};
use pathfinding::{astar_unchecked, ExpansionPolicy, NodePool};
use qcell::TLCellOwner;
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
    Jps,
}

impl FromStr for Algorithm {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "dijkstra" => Algorithm::Dijkstra,
            "astar" => Algorithm::AStar,
            "jps" => Algorithm::Jps,
            _ => return Err("invalid algorithm"),
        })
    }
}

pub fn main() {
    let options = Options::from_args();

    let instances = read_scenario(&options.map.with_extension("map.scen"));

    match options.algorithm {
        Algorithm::Dijkstra => {
            let map = read_map(&options.map);
            run(&instances, map.width(), map.height(), |_| {
                (no_corner_cutting(&map), zero_heuristic())
            });
        }
        Algorithm::AStar => {
            let map = read_map(&options.map);
            run(&instances, map.width(), map.height(), |goal| {
                (no_corner_cutting(&map), octile_heuristic(goal, 1.0))
            });
        }
        Algorithm::Jps => {
            let map = read_map(&options.map);
            let tmap = create_tmap(&map);
            run(&instances, map.width(), map.height(), |goal| {
                (jps(&map, &tmap, goal), octile_heuristic(goal, 1.0))
            });
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

fn run<E, H>(
    instances: &[Instance],
    width: i32,
    height: i32,
    mut init: impl FnMut((i32, i32)) -> (E, H),
) where
    E: ExpansionPolicy<(i32, i32)>,
    H: Fn((i32, i32)) -> f64,
{
    let mut owner = TLCellOwner::new();
    let mut pool = GridPool::new(width, height);

    let t = std::time::Instant::now();
    for instance in instances {
        let (mut expander, heuristic) = init(instance.to);
        let t = std::time::Instant::now();
        unsafe {
            // definitely not safe
            astar_unchecked(
                &mut pool,
                &mut owner,
                &mut expander,
                heuristic,
                instance.from,
                instance.to,
            );
        }
        eprintln!(
            "{:?} -> {:?}: {:.2?}",
            instance.from,
            instance.to,
            t.elapsed()
        );
        let dst = pool.generate(instance.to, &mut owner);
        let len = owner.ro(dst).g;
        assert_eq!(
            (len * 1_000.0).round() as i64,
            (instance.expected_length * 1_000.0).round() as i64
        );
    }
    println!("Total: {}", t.elapsed().as_secs_f64());
}
