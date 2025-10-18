use std::collections::{HashMap, HashSet, VecDeque};
use std::num::ParseIntError;
use std::ops::{Add, AddAssign};

use thiserror::Error;

use crate::machine::{parse_program, Machine, MachineError, Value};

#[derive(Debug, Error)]
enum RuntimeError {
    #[error("Invalid direction value: {0}")]
    InvalidDirection(Value),
    #[error("Invalid status value: {0}")]
    InvalidStatus(Value),
    #[error("Program exited before recieving any output")]
    OutputTruncated,
    #[error(transparent)]
    MachineError(#[from] MachineError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    North = 1,
    South = 2,
    West = 3,
    East = 4,
}

impl Direction {
    const fn all() -> [Self; 4] {
        [Self::North, Self::South, Self::West, Self::East]
    }
}

impl TryFrom<Value> for Direction {
    type Error = RuntimeError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Ok(match value {
            1 => Self::North,
            2 => Self::South,
            3 => Self::West,
            4 => Self::East,
            _ => return Err(RuntimeError::InvalidDirection(value)),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Status {
    HitWall = 0,
    MoveSuccess = 1,
    ReachedGoal = 2,
}

impl TryFrom<Value> for Status {
    type Error = RuntimeError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::HitWall,
            1 => Self::MoveSuccess,
            2 => Self::ReachedGoal,
            _ => return Err(RuntimeError::InvalidStatus(value)),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct Position {
    x: i32,
    y: i32,
}

impl AddAssign<Direction> for Position {
    fn add_assign(&mut self, rhs: Direction) {
        match rhs {
            Direction::North => self.y -= 1,
            Direction::South => self.y += 1,
            Direction::West => self.x -= 1,
            Direction::East => self.x += 1,
        }
    }
}

impl Add<Direction> for Position {
    type Output = Self;

    fn add(mut self, rhs: Direction) -> Self::Output {
        self += rhs;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Tile {
    #[default]
    Unknown,
    Open,
    Wall,
    Goal,
}

#[derive(Debug, Clone, Default)]
struct Map {
    tiles: HashMap<Position, Tile>,
    goal: Option<Position>,
}

impl Map {
    fn new() -> Self {
        Self::default()
    }

    fn set(&mut self, pos: Position, tile: Tile) {
        self.tiles.insert(pos, tile);
        if tile == Tile::Goal {
            self.goal = Some(pos);
        }
    }

    fn get(&self, pos: Position) -> Tile {
        self.tiles.get(&pos).copied().unwrap_or_default()
    }

    fn direction_of_nearest_unknown(&self, start_position: Position) -> Option<Direction> {
        assert!(
            self.get(start_position) != Tile::Unknown,
            "Tried to pathfind from already unknown tile"
        );
        let mut pending = VecDeque::new();
        for dir in Direction::all() {
            pending.push_back((start_position + dir, dir));
        }
        let mut visited = HashSet::new();
        visited.insert(start_position);
        while let Some((pos, initial_dir)) = pending.pop_front() {
            if !visited.insert(pos) {
                continue;
            }
            match self.get(pos) {
                Tile::Unknown => return Some(initial_dir),
                Tile::Wall => {}
                Tile::Goal | Tile::Open => {
                    for next_dir in Direction::all() {
                        pending.push_back((pos + next_dir, initial_dir));
                    }
                }
            }
        }
        None
    }

    fn shortest_distance_to_goal(&self) -> Option<usize> {
        let start_position = Position::default();
        let mut pending = VecDeque::new();
        pending.push_back((start_position, 0));
        let mut visited = HashSet::new();
        while let Some((pos, dist)) = pending.pop_front() {
            if !visited.insert(pos) {
                continue;
            }
            match self.get(pos) {
                Tile::Wall => continue,
                Tile::Unknown => return None,
                Tile::Open => {}
                Tile::Goal => return Some(dist),
            }
            for dir in Direction::all() {
                if !visited.contains(&(pos + dir)) {
                    pending.push_back((pos + dir, dist + 1));
                }
            }
        }
        None
    }

    fn longest_distance_from_goal(&self) -> Option<usize> {
        let start_position = self.goal?;
        let mut pending = VecDeque::new();
        pending.push_back((start_position, 0));
        let mut visited = HashSet::new();
        let mut max_dist = 0;
        while let Some((pos, dist)) = pending.pop_front() {
            if !visited.insert(pos) {
                continue;
            }
            match self.get(pos) {
                Tile::Wall => continue,
                Tile::Unknown => return None,
                Tile::Open | Tile::Goal => {}
            }
            max_dist = max_dist.max(dist);
            for dir in Direction::all() {
                if !visited.contains(&(pos + dir)) {
                    pending.push_back((pos + dir, dist + 1));
                }
            }
        }
        Some(max_dist)
    }
}

#[derive(Debug, Clone)]
struct RepairDroid {
    controller: Machine,
    map: Map,
    position: Position,
}

impl RepairDroid {
    fn new(program: &[Value]) -> Self {
        let position = Position::default();
        let mut map = Map::new();
        map.set(position, Tile::Open);
        Self {
            controller: Machine::new(program),
            map,
            position,
        }
    }

    fn explore(&mut self) -> Result<(), RuntimeError> {
        while let Some(dir) = self.map.direction_of_nearest_unknown(self.position) {
            self.controller.inputs.push_back(dir as Value);
            let status: Status = self
                .controller
                .run_until_output()?
                .ok_or(RuntimeError::OutputTruncated)?
                .try_into()?;
            match status {
                Status::HitWall => self.map.set(self.position + dir, Tile::Wall),
                Status::MoveSuccess => {
                    self.position += dir;
                    self.map.set(self.position, Tile::Open);
                }
                Status::ReachedGoal => {
                    self.position += dir;
                    self.map.set(self.position, Tile::Goal);
                }
            }
        }
        Ok(())
    }
}

#[aoc_generator(day15)]
fn parse(input: &str) -> Result<Vec<Value>, ParseIntError> {
    parse_program(input)
}

#[aoc(day15, part1)]
fn part_1(program: &[Value]) -> usize {
    let mut droid = RepairDroid::new(program);
    droid.explore().unwrap();
    droid.map.shortest_distance_to_goal().unwrap()
}

#[aoc(day15, part2)]
fn part_2(program: &[Value]) -> usize {
    let mut droid = RepairDroid::new(program);
    droid.explore().unwrap();
    droid.map.longest_distance_from_goal().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_MAP: &str = "\
        ?##???\n\
        #..##?\n\
        #.#S.#\n\
        #.G.#?\n\
        ?###??\
    ";

    #[test]
    #[expect(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    fn test_map() {
        let mut map = Map::new();
        let mut origin = Position::default();
        'y: for (y, line) in EXAMPLE_MAP.lines().enumerate() {
            for (x, ch) in line.bytes().enumerate() {
                if ch == b'S' {
                    origin = Position {
                        x: x as i32,
                        y: y as i32,
                    };
                    break 'y;
                }
            }
        }

        for (y, line) in EXAMPLE_MAP.lines().enumerate() {
            for (x, ch) in line.bytes().enumerate() {
                let pos = Position {
                    x: x as i32 - origin.x,
                    y: y as i32 - origin.y,
                };
                match ch {
                    b'#' => map.set(pos, Tile::Wall),
                    b'.' | b'S' => map.set(pos, Tile::Open),
                    b'G' => map.set(pos, Tile::Goal),
                    _ => {}
                }
            }
        }

        assert_eq!(map.direction_of_nearest_unknown(Position::default()), None);
        assert_eq!(map.shortest_distance_to_goal(), Some(2));
        assert_eq!(map.longest_distance_from_goal(), Some(4));
    }
}
