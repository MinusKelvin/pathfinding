use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::{Duration, Instant};

use anyhow::Result;
use itertools::{EitherOrBoth, Itertools};
use pathfinding::expansion_policy::bitgrid::jps::{create_tmap, JpsExpansionPolicy};
use pathfinding::expansion_policy::bitgrid::no_corner_cutting::NoCornerCutting;
use pathfinding::expansion_policy::ExpansionPolicy;
use pathfinding::node_pool::GridPool;
use pathfinding::util::{grid_search, octile_heuristic, zero_heuristic, GridDomain};
use pathfinding::Owner;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

mod movingai;

#[derive(StructOpt)]
struct Options {
    #[structopt(short, long, parse(try_from_str))]
    alg: Algorithm,
    scen: PathBuf,

    #[structopt(short, long, default_value = "1")]
    samples: usize,

    #[structopt(long)]
    save_baseline: bool,
}

fn main() {
    let options = Options::from_args();

    let mut stats = match options.alg {
        Algorithm::Dijkstra => {
            let scenarios = load_scenarios(&options.scen, |path| {
                let (map, problems) = movingai::load_scenario(path)?;
                Ok((format!("/{}", path.display()), map, problems))
            })
            .unwrap();

            let width = scenarios.iter().map(|s| s.1.width()).max().unwrap();
            let height = scenarios.iter().map(|s| s.1.height()).max().unwrap();
            let mut pool = GridPool::new(width, height);

            run_scenarios(
                options.samples,
                scenarios,
                |(_, map, problems)| {
                    run_grid_problems(&mut pool, NoCornerCutting::new(map), problems, |_, _| {
                        zero_heuristic()
                    })
                },
                |s| s.0,
            )
        }
        Algorithm::AStar => {
            let scenarios = load_scenarios(&options.scen, |path| {
                let (map, problems) = movingai::load_scenario(path)?;
                Ok((format!("/{}", path.display()), map, problems))
            })
            .unwrap();

            let width = scenarios.iter().map(|s| s.1.width()).max().unwrap();
            let height = scenarios.iter().map(|s| s.1.height()).max().unwrap();
            let mut pool = GridPool::new(width, height);

            run_scenarios(
                options.samples,
                scenarios,
                |(_, map, problems)| {
                    run_grid_problems(&mut pool, NoCornerCutting::new(map), problems, |_, goal| {
                        octile_heuristic(goal, 1.0)
                    })
                },
                |s| s.0,
            )
        }
        Algorithm::Jps => {
            let scenarios = load_scenarios(&options.scen, |path| {
                let (map, problems) = movingai::load_scenario(path)?;
                let tmap = create_tmap(&map);
                Ok((format!("/{}", path.display()), map, tmap, problems))
            })
            .unwrap();

            let width = scenarios.iter().map(|s| s.1.width()).max().unwrap();
            let height = scenarios.iter().map(|s| s.1.height()).max().unwrap();
            let mut pool = GridPool::new(width, height);

            run_scenarios(
                options.samples,
                scenarios,
                |(_, map, tmap, problems)| {
                    run_grid_problems(
                        &mut pool,
                        JpsExpansionPolicy::new(map, tmap),
                        problems,
                        |jps, goal| {
                            jps.set_goal(goal);
                            octile_heuristic(goal, 1.0)
                        },
                    )
                },
                |s| s.0,
            )
        }
    };

    stats.sort_by(|a, b| a.name.cmp(&b.name));

    if options.samples == 1 {
        for stat in stats {
            println!("{}", stat.name);
            println!("\t{:.2?}", Duration::from_secs_f64(stat.mean));
        }
        return;
    }

    if options.save_baseline {
        bincode::serialize_into(std::fs::File::create("baseline.dat").unwrap(), &stats).unwrap();
    }

    if let Ok(baseline) = std::fs::File::open("baseline.dat") {
        let baseline: Vec<Statistics> = bincode::deserialize_from(baseline).unwrap();

        let stats = baseline
            .into_iter()
            .merge_join_by(stats.into_iter(), |a, b| a.name.cmp(&b.name))
            .filter_map(|e| match e {
                EitherOrBoth::Both(a, b) => Some((a, b)),
                _ => None,
            });
        for (base, measured) in stats {
            let diff_mean = measured.mean - base.mean;

            let diff_stdev =
                (measured.sample_mean_stdev.powi(2) + base.sample_mean_stdev.powi(2)).sqrt();
            let z = diff_mean / diff_stdev;
            let significant = z.abs() > 2.0;

            let diff_percent = diff_mean / base.mean;

            let color = match (significant, diff_mean < 0.0) {
                (true, true) => "\x1B[32m",
                (true, false) => "\x1B[31m",
                (false, _) => "\x1B[90m",
            };

            println!("{}", base.name);
            let base = format!("{:.2?}", Duration::from_secs_f64(base.mean));
            let base = base.split_once('.').unwrap();
            let measured = format!("{:.2?}", Duration::from_secs_f64(measured.mean));
            let measured = measured.split_once('.').unwrap();
            println!(
                "\t{:>3}.{:<4} -> {:>3}.{:<4}\t{}{:+.2}% z={:.1}\x1B[39m",
                base.0, base.1,
                measured.0, measured.1,
                color,
                diff_percent * 100.0,
                z
            );
        }
    }
}

fn run_scenarios<T>(
    samples: usize,
    scenarios: Vec<T>,
    mut run: impl FnMut(&T),
    name: impl Fn(T) -> String,
) -> Vec<Statistics> {
    let mut scenario_times = vec![Vec::with_capacity(samples); scenarios.len()];
    let mut total_times = Vec::with_capacity(samples);
    for _ in 0..samples {
        let t = Instant::now();
        for (scenario, times) in scenarios.iter().zip(scenario_times.iter_mut()) {
            let t = Instant::now();
            run(scenario);
            times.push(t.elapsed());
        }
        total_times.push(t.elapsed());
    }

    scenarios
        .into_iter()
        .map(name)
        .zip(scenario_times.iter())
        .chain(std::iter::once(("Total".to_owned(), &total_times)))
        .map(|(name, times)| {
            let min = times.iter().min().unwrap().as_secs_f64();
            let mean = times.iter().sum::<Duration>() / samples as u32;

            let stdev = (times
                .iter()
                .map(|&t| t.max(mean) - t.min(mean))
                .map(|dt| dt.as_secs_f64() * dt.as_secs_f64())
                .sum::<f64>()
                / times.len() as f64)
                .sqrt();
            let mean = mean.as_secs_f64();

            Statistics {
                name,
                min,
                mean,
                stdev,
                sample_mean_stdev: stdev / (times.len() as f64).sqrt(),
            }
        })
        .collect()
}

fn load_scenarios<T>(path: &Path, mut load: impl FnMut(&Path) -> Result<T>) -> Result<Vec<T>> {
    let mut scenarios = vec![];
    if path.is_file() {
        scenarios.push(load(path)?);
    } else {
        walk(path, &mut |scen| {
            if scen.extension() == Some("scen".as_ref()) {
                scenarios.push(load(scen)?);
            }
            Ok(())
        })
        .unwrap();
    }
    Ok(scenarios)
}

fn walk(dir: &Path, f: &mut impl FnMut(&Path) -> Result<()>) -> Result<()> {
    for entry in dir.read_dir()? {
        let entry = entry?.path();
        if entry.is_dir() {
            walk(&entry, f)?;
        } else if entry.is_file() {
            f(&entry)?;
        }
    }
    Ok(())
}

fn run_grid_problems<E, H>(
    pool: &mut GridPool,
    mut expansion_policy: E,
    problems: &[Instance],
    mut setup: impl FnMut(&mut E, (i32, i32)) -> H,
) where
    E: ExpansionPolicy<(i32, i32)> + GridDomain,
    H: FnMut((i32, i32)) -> f64,
{
    let mut owner = Owner::new();
    for problem in problems {
        let h = setup(&mut expansion_policy, problem.to);
        grid_search(
            pool,
            &mut owner,
            &mut expansion_policy,
            h,
            problem.from,
            problem.to,
        );
    }
}

pub struct Instance {
    from: (i32, i32),
    to: (i32, i32),
}

#[derive(Serialize, Deserialize)]
pub struct Statistics {
    name: String,
    min: f64,
    mean: f64,
    stdev: f64,
    sample_mean_stdev: f64,
}

#[derive(Copy, Clone, Debug)]
enum Algorithm {
    Dijkstra,
    AStar,
    Jps,
}

impl FromStr for Algorithm {
    type Err = InvalidAlgorithm;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "dijkstra" => Algorithm::Dijkstra,
            "astar" => Algorithm::AStar,
            "jps" => Algorithm::Jps,
            _ => return Err(InvalidAlgorithm),
        })
    }
}

#[derive(Debug)]
struct InvalidAlgorithm;

impl std::fmt::Display for InvalidAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid algorithm")
    }
}
