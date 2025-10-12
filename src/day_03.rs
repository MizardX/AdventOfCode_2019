use std::collections::{HashMap, HashSet};
use std::num::ParseIntError;
use std::ops::{Add, AddAssign};
use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Error)]
enum ParseError {
    #[error("Syntax error")]
    SyntaxError,
    #[error(transparent)]
    InvalidNumber(#[from] ParseIntError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl TryFrom<u8> for Direction {
    type Error = ParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            b'U' => Self::Up,
            b'R' => Self::Right,
            b'D' => Self::Down,
            b'L' => Self::Left,
            _ => return Err(ParseError::SyntaxError),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Step {
    direction: Direction,
    count: u16,
}

impl FromStr for Step {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let &[first, ..] = s.as_bytes() else {
            return Err(ParseError::SyntaxError);
        };
        Ok(Self {
            direction: first.try_into()?,
            count: s[1..].parse()?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Wires {
    first: Vec<Step>,
    second: Vec<Step>,
}

impl FromStr for Wires {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        let first = lines
            .next()
            .ok_or(ParseError::SyntaxError)?
            .split(',')
            .map(str::parse)
            .collect::<Result<_, _>>()?;
        let second = lines
            .next()
            .ok_or(ParseError::SyntaxError)?
            .split(',')
            .map(str::parse)
            .collect::<Result<_, _>>()?;
        Ok(Self { first, second })
    }
}

#[aoc_generator(day3)]
fn parse(input: &str) -> Result<Wires, ParseError> {
    input.parse()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct Position {
    x: i64,
    y: i64,
}

impl Position {
    const fn dist(self) -> u64 {
        self.x.unsigned_abs() + self.y.unsigned_abs()
    }
}

impl AddAssign<Direction> for Position {
    fn add_assign(&mut self, rhs: Direction) {
        match rhs {
            Direction::Up => self.y -= 1,
            Direction::Right => self.x += 1,
            Direction::Down => self.y += 1,
            Direction::Left => self.x -= 1,
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

#[aoc(day3, part1)]
fn part_1(wires: &Wires) -> u64 {
    let mut visited = HashSet::new();
    let mut closest_dist = u64::MAX;
    for pos in WireStepper::new(&wires.first) {
        visited.insert(pos);
    }
    for pos in WireStepper::new(&wires.second) {
        if visited.contains(&pos) {
            closest_dist = closest_dist.min(pos.dist());
        }
    }
    closest_dist
}

#[aoc(day3, part2)]
fn part_2(wires: &Wires) -> u64 {
    let mut visited = HashMap::new();
    for (pos, time1) in WireStepper::new(&wires.first).zip(1..) {
        visited.entry(pos).or_insert(time1);
    }
    let mut minimum_steps = u64::MAX;
    for (pos, time2) in WireStepper::new(&wires.second).zip(1..) {
        if let Some(&time1) = visited.get(&pos) {
            minimum_steps = minimum_steps.min(time2 + time1);
        }
    }
    minimum_steps
}

struct WireStepper<'a> {
    steps: &'a [Step],
    index: usize,
    stepped: u16,
    position: Position,
}

impl<'a> WireStepper<'a> {
    fn new(steps: &'a [Step]) -> Self {
        Self {
            steps,
            index: 0,
            stepped: 0,
            position: Position::default(),
        }
    }
}

impl Iterator for WireStepper<'_> {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        let current = *self.steps.get(self.index)?;
        self.position += current.direction;
        self.stepped += 1;
        if self.stepped >= current.count {
            self.index += 1;
            self.stepped = 0;
        }
        Some(self.position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    const EXAMPLE1: &str = "\
        R8,U5,L5,D3\n\
        U7,R6,D4,L4
    ";

    const EXAMPLE2: &str = "\
        R75,D30,R83,U83,L12,D49,R71,U7,L72\n\
        U62,R66,U55,R34,D71,R55,D58,R83\
    ";

    const EXAMPLE3: &str = "\
        R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51\n\
        U98,R91,D20,R16,D67,R40,U7,R15,U6,R7\
    ";

    macro_rules! step {
        ($dir:ident $count:literal) => {
            Step {
                direction: Direction::$dir,
                count: $count,
            }
        };
    }

    #[test]
    fn test_parse() {
        let result = parse(EXAMPLE1).unwrap();
        assert_eq!(
            result.first,
            [step!(Right 8), step!(Up 5), step!(Left 5), step!(Down 3)]
        );
        assert_eq!(
            result.second,
            [step!(Up 7), step!(Right 6), step!(Down 4), step!(Left 4)]
        );
    }

    #[test_case(EXAMPLE1 => 6)]
    #[test_case(EXAMPLE2 => 159)]
    #[test_case(EXAMPLE3 => 135)]
    fn test_part_1(input: &str) -> u64 {
        let wires = parse(input).unwrap();
        part_1(&wires)
    }

    #[test_case(EXAMPLE1 => 30)]
    #[test_case(EXAMPLE2 => 610)]
    #[test_case(EXAMPLE3 => 410)]
    fn test_part_2(input: &str) -> u64 {
        let wires = parse(input).unwrap();
        part_2(&wires)
    }
}
