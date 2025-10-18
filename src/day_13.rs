use std::collections::HashMap;
use std::fmt::Display;
use std::num::ParseIntError;

use thiserror::Error;

use crate::machine::{parse_program, Machine, MachineError, Value};

#[derive(Debug, Error)]
enum RuntimeError {
    #[error("Invalid tile value: {0}")]
    InvalidTile(Value),
    #[error("Could not find location of the ball")]
    MissingBall,
    #[error("Could not find location of the paddle")]
    MissingPaddle,
    #[error(transparent)]
    MachineError(#[from] MachineError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tile {
    Empty = 0,
    Wall = 1,
    Block = 2,
    HorizontalPaddle = 3,
    Ball = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[allow(unused)]
enum AnsiColor {
    Black = 0,
    Red,
    Green,
    Yellow,
    Blue,
    Purple,
    Cyan,
    White,
    Reset,
}

impl Display for AnsiColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if matches!(self, Self::Reset) {
            write!(f, "\x1b[0m")
        } else if f.alternate() {
            let n = *self as u8;
            write!(f, "\x1b[3{n}m")
        } else {
            let n = *self as u8;
            write!(f, "\x1b[4{n}m")
        }
    }
}

impl Tile {
    const fn color(self) -> AnsiColor {
        match self {
            Self::Empty => AnsiColor::Black,
            Self::Wall => AnsiColor::White,
            Self::Block => AnsiColor::Purple,
            Self::HorizontalPaddle => AnsiColor::Yellow,
            Self::Ball => AnsiColor::Blue,
        }
    }
}

impl TryFrom<Value> for Tile {
    type Error = RuntimeError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Empty,
            1 => Self::Wall,
            2 => Self::Block,
            3 => Self::HorizontalPaddle,
            4 => Self::Ball,
            _ => return Err(RuntimeError::InvalidTile(value)),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Position {
    x: Value,
    y: Value,
}
impl Position {
    const fn new(x: Value, y: Value) -> Self {
        Self { x, y }
    }
}

impl From<(Value, Value)> for Position {
    fn from((x, y): (Value, Value)) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Default)]
struct Screen {
    tiles: HashMap<Position, Tile>,
}

impl Screen {
    fn new() -> Self {
        Self::default()
    }

    fn set_tile(&mut self, x: Value, y: Value, tile: Tile) {
        self.tiles.insert((x, y).into(), tile);
    }
}

impl Display for Screen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in (0..20).step_by(2) {
            writeln!(f)?;
            for x in 0..44 {
                let tile1 = self
                    .tiles
                    .get(&Position::new(x, y))
                    .copied()
                    .unwrap_or(Tile::Empty)
                    .color();
                let tile2 = self
                    .tiles
                    .get(&Position::new(x, y + 1))
                    .copied()
                    .unwrap_or(Tile::Empty)
                    .color();
                write!(f, "{tile1}{tile2:#}▀")?;
            }
            write!(f, "{}", AnsiColor::Reset)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct Arcade {
    controller: Machine,
    screen: Screen,
    score: Value,
    log: bool,
}

impl Arcade {
    fn new(program: &[Value]) -> Self {
        Self {
            controller: Machine::new(program),
            screen: Screen::new(),
            score: 0,
            log: false,
        }
    }

    fn tick(&mut self) -> Result<(), RuntimeError> {
        while let Some(x) = self.controller.run_until_output()?
            && let Some(y) = self.controller.run_until_output()?
            && let Some(tile) = self.controller.run_until_output()?
        {
            if (x, y) == (-1, 0) {
                self.score = tile;
            } else {
                self.screen.set_tile(x, y, tile.try_into()?);
            }
        }
        Ok(())
    }

    fn count_blocks(&self) -> usize {
        self.screen
            .tiles
            .values()
            .filter(|t| matches!(t, Tile::Block))
            .count()
    }

    fn play(&mut self) -> Result<(), RuntimeError> {
        let mut first = true;
        loop {
            match self.tick().unwrap_err() {
                RuntimeError::MachineError(MachineError::Stopped) => {
                    return Ok(())
                }
                RuntimeError::MachineError(MachineError::EmptyInput) => {
                    if self.log {
                        if first {
                            first = false;
                        } else {
                            print!("\x1b[11A");
                        }
                        println!("{}", &self.screen);
                    }
                    let ball_x = self
                        .screen
                        .tiles
                        .iter()
                        .find_map(|(&pos, &tile)| (tile == Tile::Ball).then_some(pos.x))
                        .ok_or(RuntimeError::MissingBall)?;
                    let paddle_x = self
                        .screen
                        .tiles
                        .iter()
                        .find_map(|(&pos, &tile)| (tile == Tile::HorizontalPaddle).then_some(pos.x))
                        .ok_or(RuntimeError::MissingPaddle)?;
                    self.controller
                        .inputs
                        .push_back((ball_x - paddle_x).signum());
                }
                e => Err(e)?,
            }
        }
    }
}

#[aoc_generator(day13)]
fn parse(input: &str) -> Result<Vec<Value>, ParseIntError> {
    parse_program(input)
}

#[aoc(day13, part1)]
fn part_1(program: &[Value]) -> usize {
    let mut arcade = Arcade::new(program);
    arcade.play().unwrap();
    arcade.count_blocks()
}

#[aoc(day13, part2)]
fn part_2(program: &[Value]) -> Value {
    let mut arcade = Arcade::new(program);
    arcade.controller.write(0, 2);
    arcade.play().unwrap();
    arcade.score
}

// No test cases