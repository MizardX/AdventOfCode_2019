use std::collections::HashMap;
use std::num::ParseIntError;
use std::ops::{Add, AddAssign};

use thiserror::Error;

use crate::machine::{parse_program, Machine, MachineError, Value};

#[aoc_generator(day19)]
fn parse(input: &str) -> Result<Vec<Value>, ParseIntError> {
    parse_program(input)
}

#[aoc(day19, part1)]
fn part_1(program: &[Value]) -> usize {
    let mut controller = DroneController::new(program);
    let mut count = 0;
    for y in 0..50 {
        for x in 0..50 {
            let pos = Position::new(x, y);
            if controller.test_coordinates(pos).unwrap() == DroneResult::BeingPulled {
                count += 1;
            }
        }
    }
    count
}

#[aoc(day19, part2)]
fn part_2(program: &[Value]) -> i32 {
    let pos = find_contained_box(program, 100).unwrap();
    pos.x * 10000 + pos.y
}

fn find_contained_box(program: &[Value], size: i32) -> Result<Position, RuntimeError> {
    let mut controller = DroneController::new(program);
    let mut corner = Position::new(50, 0);
    while controller.test_coordinates(corner)? == DroneResult::Stationary {
        corner += Direction::Down;
    }
    let mut bottom = Position::new(corner.x, corner.y + size - 1);
    let mut right = Position::new(corner.x + size - 1, corner.x);
    loop {
        if controller.test_coordinates(corner + Direction::DownRight)? == DroneResult::BeingPulled
            && controller.test_coordinates(right)? == DroneResult::Stationary
            && controller.test_coordinates(bottom)? == DroneResult::Stationary
        {
            corner += Direction::DownRight;
            bottom += Direction::DownRight;
            right += Direction::DownRight;
        } else if controller.test_coordinates(corner + Direction::Right)?
            == DroneResult::BeingPulled
            && controller.test_coordinates(bottom)? == DroneResult::Stationary
        {
            corner += Direction::Right;
            bottom += Direction::Right;
            right += Direction::Right;
        } else if controller.test_coordinates(corner + Direction::Down)? == DroneResult::BeingPulled
            && controller.test_coordinates(right)? == DroneResult::Stationary
        {
            corner += Direction::Down;
            bottom += Direction::Down;
            right += Direction::Down;
        } else {
            break;
        }
    }
    let mut closest = corner;
    for y in -size / 4..=0 {
        for x in (-size / 4).max(y - 10)..=0.min(y + 10) {
            let test = Position::new(corner.x + x, corner.y + y);
            let right = Position::new(corner.x + x + size - 1, corner.y + y);
            let bottom = Position::new(corner.x + x, corner.y + y + size - 1);
            if controller.test_coordinates(test)? == DroneResult::BeingPulled
                && controller.test_coordinates(right)? == DroneResult::BeingPulled
                && controller.test_coordinates(bottom)? == DroneResult::BeingPulled
                && test.dist() < closest.dist() {
                    closest = test;
                } 
        }
    }
    Ok(closest)
}

struct DroneController<'a> {
    machine: Machine,
    program: &'a [Value],
    cache: HashMap<Position, DroneResult>,
    log: bool,
}

impl<'a> DroneController<'a> {
    fn new(program: &'a [Value]) -> Self {
        Self {
            machine: Machine::new(program),
            program,
            cache: HashMap::new(),
            log: false,
        }
    }

    fn test_coordinates(&mut self, pos: Position) -> Result<DroneResult, RuntimeError> {
        if let Some(&old) = self.cache.get(&pos) {
            return Ok(old);
        }

        self.machine.reset(self.program);

        self.machine.inputs.push_back(pos.x.into());
        self.machine.inputs.push_back(pos.y.into());
        let res = self
            .machine
            .run_until_output()?
            .ok_or(RuntimeError::UnexpectedTermination)?
            .try_into()?;

        if self.log {
            println!("{pos:?} -> {res:?}");
        }

        self.cache.insert(pos, res);

        Ok(res)
    }
}

#[derive(Debug, Error)]
enum RuntimeError {
    #[error("Invalid result: {0}")]
    InvalidResult(Value),
    #[error(transparent)]
    MachineError(#[from] MachineError),
    #[error("Program terminated unexpectedly")]
    UnexpectedTermination,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DroneResult {
    Stationary = 0,
    BeingPulled = 1,
}

impl TryFrom<Value> for DroneResult {
    type Error = RuntimeError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Stationary,
            1 => Self::BeingPulled,
            _ => return Err(RuntimeError::InvalidResult(value)),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct Position {
    x: i32,
    y: i32,
}

impl Position {
    const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    const fn dist(self) -> u32 {
        self.x.unsigned_abs() + self.y.unsigned_abs()
    }
}

impl AddAssign<Direction> for Position {
    fn add_assign(&mut self, rhs: Direction) {
        match rhs {
            Direction::Down => self.y += 1,
            Direction::Right => self.x += 1,
            Direction::DownRight => {
                self.y += 1;
                self.x += 1;
            }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Down,
    Right,
    DownRight,
}

// No test cases