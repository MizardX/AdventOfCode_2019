use std::collections::{HashMap, VecDeque};
use std::fmt::Display;
use std::num::ParseIntError;

type Value = i64;

use thiserror::Error;

#[derive(Debug, Error)]
enum MachineError {
    #[error("Invalid instruction: {0}")]
    InvalidInstruction(Value),
    #[error("Invalid parameter mode: {0}")]
    InvalidParameterMode(Value),
    #[error("Tried to read empty input")]
    EmptyInput,
    #[error("Machine is not in state Running")]
    Stopped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum ParameterMode {
    Position = 0,
    Immediate = 1,
    Relative = 2,
}

impl TryFrom<Value> for ParameterMode {
    type Error = MachineError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Ok(match value % 10 {
            0 => Self::Position,
            1 => Self::Immediate,
            2 => Self::Relative,
            _ => return Err(MachineError::InvalidParameterMode(value)),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArgumentBy {
    Position(Value),
    Value(Value),
    Relative(Value),
}

impl ArgumentBy {
    fn read(self, machine: &Machine) -> Value {
        match self {
            Self::Position(index) => machine.read(index),
            Self::Value(val) => val,
            Self::Relative(index) => machine.read_relative(index),
        }
    }

    fn write(self, value: Value, machine: &mut Machine) {
        match self {
            Self::Position(index) => {
                machine.write(index, value);
            }
            Self::Relative(index) => {
                machine.write_relative(index, value);
            }
            Self::Value(..) => {
                panic!("Trying to write into immediate value");
            }
        }
    }
}

impl Display for ArgumentBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Position(pos) => write!(f, "#{pos}"),
            Self::Value(val) => write!(f, "{val}"),
            Self::Relative(val) => write!(f, "${val:+}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OpCode {
    Nonary(OpCode0),
    Unary(OpCode1, ParameterMode),
    Binary(OpCode2, ParameterMode, ParameterMode),
    Trinary(OpCode3, ParameterMode, ParameterMode, ParameterMode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum OpCode0 {
    Halt = 99,
}

impl OpCode0 {
    #[allow(clippy::unnecessary_wraps)]
    const fn execute(self, machine: &mut Machine) -> Result<Option<Value>, MachineError> {
        match self {
            Self::Halt => machine.state = State::Stopped,
        }
        Ok(None)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum OpCode1 {
    Input = 3,
    Output = 4,
    AdjustRelativeBase = 9,
}

impl OpCode1 {
    fn execute(
        self,
        arg1: ArgumentBy,
        machine: &mut Machine,
    ) -> Result<Option<Value>, MachineError> {
        match self {
            Self::Input => {
                let value = machine.read_input()?;
                arg1.write(value, machine);
            }
            Self::Output => {
                let value = arg1.read(machine);
                machine.write_output(value);
            }
            Self::AdjustRelativeBase => {
                let value = arg1.read(machine);
                machine.relative_base += value;
            }
        }
        Ok(None)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum OpCode2 {
    JumpIfTrue = 3,
    JumpIfFalse = 4,
}

impl OpCode2 {
    #[allow(clippy::unnecessary_wraps)]
    fn execute(
        self,
        arg1: ArgumentBy,
        arg2: ArgumentBy,
        machine: &Machine,
    ) -> Result<Option<Value>, MachineError> {
        Ok(match self {
            Self::JumpIfTrue => {
                let condition = arg1.read(machine);
                if condition != 0 {
                    Some(arg2.read(machine))
                } else {
                    None
                }
            }
            Self::JumpIfFalse => {
                let condition = arg1.read(machine);
                if condition == 0 {
                    Some(arg2.read(machine))
                } else {
                    None
                }
            }
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
enum OpCode3 {
    Add = 1,
    Mul = 2,
    LessThan = 7,
    Equals = 8,
}

impl OpCode3 {
    #[allow(clippy::unnecessary_wraps)]
    fn execute(
        self,
        arg1: ArgumentBy,
        arg2: ArgumentBy,
        arg3: ArgumentBy,
        machine: &mut Machine,
    ) -> Result<Option<Value>, MachineError> {
        match self {
            Self::Add => arg3.write(arg1.read(machine) + arg2.read(machine), machine),
            Self::Mul => arg3.write(arg1.read(machine) * arg2.read(machine), machine),
            Self::LessThan => {
                arg3.write(
                    Value::from(arg1.read(machine) < arg2.read(machine)),
                    machine,
                );
            }
            Self::Equals => {
                arg3.write(
                    Value::from(arg1.read(machine) == arg2.read(machine)),
                    machine,
                );
            }
        }
        Ok(None)
    }
}

impl TryFrom<Value> for OpCode {
    type Error = MachineError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Ok(match value % 100 {
            code @ (1 | 2 | 7 | 8) => Self::Trinary(
                match code {
                    1 => OpCode3::Add,
                    2 => OpCode3::Mul,
                    7 => OpCode3::LessThan,
                    8 => OpCode3::Equals,
                    _ => unreachable!(),
                },
                ParameterMode::try_from(value / 100 % 10)?,
                ParameterMode::try_from(value / 1_000 % 10)?,
                ParameterMode::try_from(value / 10_000 % 10)?,
            ),
            code @ (3 | 4 | 9) => Self::Unary(
                match code {
                    3 => OpCode1::Input,
                    4 => OpCode1::Output,
                    9 => OpCode1::AdjustRelativeBase,
                    _ => unreachable!(),
                },
                ParameterMode::try_from(value / 100 % 10)?,
            ),
            code @ (5 | 6) => Self::Binary(
                match code {
                    5 => OpCode2::JumpIfTrue,
                    6 => OpCode2::JumpIfFalse,
                    _ => unreachable!(),
                },
                ParameterMode::try_from(value / 100 % 10)?,
                ParameterMode::try_from(value / 1_000 % 10)?,
            ),
            99 => Self::Nonary(OpCode0::Halt),
            _ => return Err(MachineError::InvalidInstruction(value)),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Running,
    Stopped,
}

#[derive(Debug, Clone)]
struct Machine {
    memory: Vec<Value>,
    ip: Value,
    state: State,
    log: bool,
    inputs: VecDeque<Value>,
    outputs: VecDeque<Value>,
    relative_base: Value,
}

impl Machine {
    fn new(program: &[Value]) -> Self {
        Self {
            memory: program.to_vec(),
            ip: 0,
            state: State::Running,
            log: false,
            inputs: VecDeque::new(),
            outputs: VecDeque::new(),
            relative_base: 0,
        }
    }

    fn get_arg(&self, offset: Value, mode: ParameterMode) -> ArgumentBy {
        let value = self.read(self.ip + offset);
        match mode {
            ParameterMode::Position => ArgumentBy::Position(value),
            ParameterMode::Immediate => ArgumentBy::Value(value),
            ParameterMode::Relative => ArgumentBy::Relative(value),
        }
    }

    fn get_op(&self) -> OpCode {
        self.read(self.ip).try_into().expect("Invalid opcode")
    }

    fn read(&self, index: Value) -> Value {
        if let Ok(index) = usize::try_from(index)
            && let Some(&mem) = self.memory.get(index)
        {
            mem
        } else {
            0
        }
    }

    fn read_relative(&self, index: Value) -> Value {
        self.read(self.relative_base + index)
    }

    fn write(&mut self, index: Value, value: Value) {
        if let Ok(index) = usize::try_from(index) {
            if index >= self.memory.len() {
                self.memory.resize(index + 1, value);
            }
            self.memory[index] = value;
        } else {
            panic!("Tried to write to negative address");
        }
    }

    fn write_relative(&mut self, index: Value, value: Value) {
        self.write(self.relative_base + index, value);
    }

    #[expect(unused, reason = "Not needed in this problem")]
    fn reset(&mut self, program: &[Value]) {
        self.memory.copy_from_slice(program);
        self.ip = 0;
        self.state = State::Running;
        self.inputs.clear();
        self.outputs.clear();
    }

    fn read_input(&mut self) -> Result<Value, MachineError> {
        self.inputs.pop_front().ok_or(MachineError::EmptyInput)
    }

    fn write_output(&mut self, value: Value) {
        self.outputs.push_back(value);
    }

    fn step(&mut self) -> Result<(), MachineError> {
        if self.state != State::Running {
            return Err(MachineError::Stopped);
        }
        let op = self.get_op();
        match op {
            OpCode::Nonary(op) => {
                if self.log {
                    println!("[{}] {op:?}", self.ip);
                }
                self.ip = op.execute(self)?.unwrap_or(self.ip + 1);
            }
            OpCode::Unary(op, p1) => {
                let arg1 = self.get_arg(1, p1);
                if self.log {
                    println!("[{}] {op:?} {arg1}", self.ip);
                }
                self.ip = op.execute(arg1, self)?.unwrap_or(self.ip + 2);
            }
            OpCode::Binary(op, p1, p2) => {
                let arg1 = self.get_arg(1, p1);
                let arg2 = self.get_arg(2, p2);
                if self.log {
                    println!("[{}] {op:?} {arg1} {arg2}", self.ip);
                }
                self.ip = op.execute(arg1, arg2, self)?.unwrap_or(self.ip + 3);
            }
            OpCode::Trinary(op, p1, p2, p3) => {
                let arg1 = self.get_arg(1, p1);
                let arg2 = self.get_arg(2, p2);
                let arg3 = self.get_arg(3, p3);
                if self.log {
                    println!("[{}] {op:?} {arg1} {arg2} {arg3}", self.ip);
                }
                self.ip = op.execute(arg1, arg2, arg3, self)?.unwrap_or(self.ip + 4);
            }
        }
        Ok(())
    }

    fn run_until_output(&mut self) -> Result<Option<Value>, MachineError> {
        while self.outputs.is_empty() {
            self.step()?;
        }
        Ok(self.outputs.pop_front())
    }
}

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
    input.split(',').map(str::parse).collect()
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