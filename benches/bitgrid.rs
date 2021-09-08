use common::movingai::Problem;
use criterion::{criterion_group, criterion_main, Criterion};
use pathfinding::expansion_policy::bitgrid::jps::{create_tmap, JpsExpansionPolicy};
use pathfinding::expansion_policy::bitgrid::no_corner_cutting::NoCornerCutting;
use pathfinding::expansion_policy::ExpansionPolicy;
use pathfinding::node_pool::GridPool;
use pathfinding::util::{grid_search, octile_heuristic, zero_heuristic, GridDomain};
use pathfinding::Owner;

mod common;

fn benchmark(c: &mut Criterion) {
    common::walk("maps/bitgrid", &mut vec![], &mut |path, rope| {
        if path.extension() != Some("scen".as_ref()) {
            return;
        }
        let name = rope.join("/");
        c.benchmark_group(name)
            .bench_function("dijkstra", |b| {
                let (map, problems) = common::movingai::load_scenario(path).unwrap();
                let mut pool = GridPool::new(map.width(), map.height());
                let mut ep = NoCornerCutting::new(&map);
                b.iter(|| run(&mut pool, &problems, &mut ep, |_, _| zero_heuristic()));
            })
            .bench_function("astar", |b| {
                let (map, problems) = common::movingai::load_scenario(path).unwrap();
                let mut pool = GridPool::new(map.width(), map.height());
                let mut ep = NoCornerCutting::new(&map);
                b.iter(|| {
                    run(&mut pool, &problems, &mut ep, |_, goal| {
                        octile_heuristic(goal, 1.0)
                    })
                });
            })
            .bench_function("jps", |b| {
                let (map, problems) = common::movingai::load_scenario(path).unwrap();
                let tmap = create_tmap(&map);
                let mut pool = GridPool::new(map.width(), map.height());
                let mut ep = JpsExpansionPolicy::new(&map, &tmap);
                b.iter(|| {
                    run(&mut pool, &problems, &mut ep, |ep, goal| {
                        ep.set_goal(goal);
                        octile_heuristic(goal, 1.0)
                    })
                });
            });
    })
}

fn run<E, H>(
    pool: &mut GridPool,
    problems: &[Problem],
    ep: &mut E,
    mut init: impl FnMut(&mut E, (i32, i32)) -> H,
) where
    E: ExpansionPolicy<(i32, i32)> + GridDomain,
    H: FnMut((i32, i32)) -> f64,
{
    let mut owner = Owner::new();
    for problem in problems {
        let h = init(ep, problem.to);
        grid_search(pool, &mut owner, ep, h, problem.from, problem.to);
    }
}

criterion_group! {
    name = bench;
    config = Criterion::default();
    targets = benchmark
}

criterion_main!(bench);
