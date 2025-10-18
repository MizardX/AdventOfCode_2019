use std::collections::HashMap;
use std::num::ParseIntError;
use std::ops::{Add, AddAssign};

use thiserror::Error;

use crate::machine::{Machine, MachineError, State, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Direction {
    #[default]
    Up,
    Right,
    Down,
    Left,
}

impl Direction {
    const fn clockwise(self) -> Self {
        match self {
            Self::Up => Self::Right,
            Self::Right => Self::Down,
            Self::Down => Self::Left,
            Self::Left => Self::Up,
        }
    }
    const fn counterclockwise(self) -> Self {
        match self {
            Self::Up => Self::Left,
            Self::Right => Self::Up,
            Self::Down => Self::Right,
            Self::Left => Self::Down,
        }
    }
}

#[derive(Debug, Error)]
enum AntError {
    #[error("Invalid value for a Turn: {0}")]
    InvalidTurn(Value),
    #[error("Invalid value for a Color: {0}")]
    InvalidColor(Value),
    #[error(transparent)]
    MachineError(#[from] MachineError),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Turn {
    Left,
    Right,
}

impl TryFrom<Value> for Turn {
    type Error = AntError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Left,
            1 => Self::Right,
            _ => return Err(AntError::InvalidTurn(value)),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum PixelColor {
    #[default]
    Black = 0,
    White = 1,
}

impl TryFrom<Value> for PixelColor {
    type Error = AntError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Black,
            1 => Self::White,
            _ => return Err(AntError::InvalidColor(value)),
        })
    }
}

#[derive(Debug, Clone, Default)]
struct PainterAnt {
    pixels: HashMap<Position, PixelColor>,
    position: Position,
    direction: Direction,
}

impl PainterAnt {
    fn new() -> Self {
        Self::default()
    }

    fn observe_camera(&self) -> PixelColor {
        self.pixels
            .get(&self.position)
            .copied()
            .unwrap_or(PixelColor::Black)
    }

    fn turn(&mut self, turn: Turn) {
        self.direction = match turn {
            Turn::Left => self.direction.counterclockwise(),
            Turn::Right => self.direction.clockwise(),
        };
        self.position += self.direction;
    }

    fn paint(&mut self, color: PixelColor) {
        self.pixels.insert(self.position, color);
    }

    fn render_image(&self) -> String {
        let mut min_x = i32::MAX;
        let mut max_x = i32::MIN;
        let mut min_y = i32::MAX;
        let mut max_y = i32::MIN;
        for &pos in self.pixels.keys() {
            min_x = min_x.min(pos.x);
            max_x = max_x.max(pos.x);
            min_y = min_y.min(pos.y);
            max_y = max_y.max(pos.y);
        }
        let width = usize::try_from(max_x - min_x + 1).unwrap();
        let height = usize::try_from(max_y - min_y + 1).unwrap();
        let mut image = String::with_capacity((width + 1) * height.div_ceil(2));
        for y in (min_y..=max_y).step_by(2) {
            image.push('\n');
            for x in min_x..=max_x {
                let p1 = self
                    .pixels
                    .get(&Position::new(x, y))
                    .copied()
                    .unwrap_or(PixelColor::Black);
                let p2 = self
                    .pixels
                    .get(&Position::new(x, y + 1))
                    .copied()
                    .unwrap_or(PixelColor::Black);
                image.push(match (p1, p2) {
                    (PixelColor::White, PixelColor::White) => '█',
                    (PixelColor::White, PixelColor::Black) => '▀',
                    (PixelColor::Black, PixelColor::White) => '▄',
                    (PixelColor::Black, PixelColor::Black) => ' ',
                });
            }
        }
        image
    }
}

struct AntController {
    machine: Machine,
    painter: PainterAnt,
}

impl AntController {
    fn new(program: &[Value]) -> Self {
        Self {
            machine: Machine::new(program),
            painter: PainterAnt::new(),
        }
    }

    fn step(&mut self) -> Result<(), AntError> {
        let color = self.painter.observe_camera();
        self.machine.inputs.push_back(color as Value);
        if let Some(new_color) = self.machine.run_until_output()? {
            self.painter.paint(new_color.try_into()?);
        }
        if let Some(turn) = self.machine.run_until_output()? {
            self.painter.turn(turn.try_into()?);
        }
        Ok(())
    }

    fn run_until_completion(&mut self) -> Result<usize, AntError> {
        while self.machine.state() == State::Running {
            self.step()?;
        }
        Ok(self.painter.pixels.len())
    }
}

#[aoc_generator(day11)]
fn parse(input: &str) -> Result<Vec<Value>, ParseIntError> {
    input.split(',').map(str::parse).collect()
}

#[aoc(day11, part1)]
fn part_1(program: &[Value]) -> usize {
    let mut controller = AntController::new(program);
    controller.run_until_completion().unwrap()
}

#[aoc(day11, part2)]
fn part_2(program: &[Value]) -> String {
    let mut controller = AntController::new(program);
    controller.painter.paint(PixelColor::White);
    controller.run_until_completion().unwrap();
    controller.painter.render_image()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ant() {
        let mut ant = PainterAnt::new();
        assert_eq!(ant.observe_camera(), PixelColor::Black);
        for (paint, turn) in [(1, 0), (0, 0), (1, 0), (1, 0), (0, 1), (1, 0), (1, 0)] {
            ant.paint(paint.try_into().unwrap());
            ant.turn(turn.try_into().unwrap());
        }
        assert_eq!(ant.pixels.len(), 6);
        assert_eq!(ant.render_image(), "\n  █\n▀▀ ");
    }
}
