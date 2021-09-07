use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use pathfinding::domains::BitGrid;

use crate::Instance;

pub fn load_scenario(scen: &Path) -> Result<(BitGrid, Vec<Instance>), MovingAiParseError> {
    let map = parse_map(&scen.with_extension("").with_extension("map"))?;
    let problems = parse_scen(scen, &map)?;

    Ok((map, problems))
}

fn parse_scen(scen: &Path, map: &BitGrid) -> Result<Vec<Instance>, MovingAiParseError> {
    let scen = BufReader::new(File::open(scen)?);
    let mut scen = scen.lines();
    let mut next = || match scen.next() {
        Some(v) => Ok(v?),
        None => Err(MovingAiParseError::UnexpectedEof),
    };

    let l = next()?;
    let [version, v] = split(&l).ok_or(MovingAiParseError::InvalidHeader)?;
    if version != "version" || !(v == "1" || v == "1.0") {
        return Err(MovingAiParseError::InvalidHeader);
    }

    let mut problems = vec![];
    for line in scen {
        let line = line?;
        if line.is_empty() {
            continue;
        }
        let [_, _, map_width, map_height, start_x, start_y, goal_x, goal_y, _opt_length] =
            split(&line).ok_or(MovingAiParseError::InvalidData)?;

        let map_width: i32 = map_width.parse()?;
        let map_height: i32 = map_height.parse()?;
        if map_width != map.width() || map_height != map.height() {
            return Err(MovingAiParseError::InvalidData);
        }

        let start_x: i32 = start_x.parse()?;
        let start_y: i32 = start_y.parse()?;
        let goal_x: i32 = goal_x.parse()?;
        let goal_y: i32 = goal_y.parse()?;

        if start_x < 0 || start_x >= map.width() || start_y < 0 || start_y >= map.height() {
            return Err(MovingAiParseError::InvalidData);
        }
        if goal_x < 0 || goal_x >= map.width() || goal_y < 0 || goal_y >= map.height() {
            return Err(MovingAiParseError::InvalidData);
        }

        problems.push(Instance {
            from: (start_x, start_y),
            to: (goal_x, goal_y),
        });
    }

    Ok(problems)
}

fn parse_map(map: &Path) -> Result<BitGrid, MovingAiParseError> {
    let map = BufReader::new(File::open(map)?);
    let mut map = map.lines();
    let mut next = || match map.next() {
        Some(v) => Ok(v?),
        None => Err(MovingAiParseError::UnexpectedEof),
    };

    if split(&next()?) != Some(["type", "octile"]) {
        return Err(MovingAiParseError::InvalidHeader);
    }

    let l = next()?;
    let [height_str, height] = split(&l).ok_or(MovingAiParseError::InvalidHeader)?;
    if height_str != "height" {
        return Err(MovingAiParseError::InvalidHeader);
    }
    let height = height.parse()?;

    if height <= 0 {
        return Err(MovingAiParseError::InvalidData);
    }

    let l = next()?;
    let [width_str, width] = split(&l).ok_or(MovingAiParseError::InvalidHeader)?;
    if width_str != "width" {
        return Err(MovingAiParseError::InvalidHeader);
    }
    let width = width.parse()?;

    if width <= 0 {
        return Err(MovingAiParseError::InvalidData);
    }

    if split(&next()?) != Some(["map"]) {
        return Err(MovingAiParseError::InvalidHeader);
    }

    let mut grid = BitGrid::new(width, height);

    for y in 0..height {
        let line = next()?;
        let line = line.as_bytes();
        if line.len() != width as usize {
            return Err(MovingAiParseError::InvalidData);
        }
        for x in 0..width {
            grid.set(x, y, matches!(line[x as usize], b'@' | b'O' | b'T'));
        }
    }

    Ok(grid)
}

fn split<const N: usize>(l: &str) -> Option<[&str; N]> {
    let mut result = [""; N];
    let mut iter = l.split_whitespace();
    for i in 0..N {
        result[i] = iter.next()?;
    }
    if iter.next().is_some() {
        return None;
    }
    Some(result)
}

#[derive(Debug)]
pub enum MovingAiParseError {
    Stdio(std::io::Error),
    ParseError(std::num::ParseIntError),
    InvalidHeader,
    InvalidData,
    UnexpectedEof,
}

impl From<std::io::Error> for MovingAiParseError {
    fn from(e: std::io::Error) -> Self {
        Self::Stdio(e)
    }
}

impl From<std::num::ParseIntError> for MovingAiParseError {
    fn from(e: std::num::ParseIntError) -> Self {
        Self::ParseError(e)
    }
}

impl std::fmt::Display for MovingAiParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stdio(e) => write!(f, "{}", e),
            Self::ParseError(e) => write!(f, "{}", e),
            Self::InvalidHeader => write!(f, "Invalid file header"),
            Self::UnexpectedEof => write!(f, "Expected more data, but got EOF"),
            Self::InvalidData => write!(f, "Invalid data provided"),
        }
    }
}

impl std::error::Error for MovingAiParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Stdio(e) => Some(e),
            Self::ParseError(e) => Some(e),
            _ => None,
        }
    }
}
