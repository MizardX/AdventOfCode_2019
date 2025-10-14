use std::collections::{HashMap, VecDeque};
use std::fmt::Display;
use std::num::ParseIntError;
use std::ops::{Add, AddAssign};

type Value = i64;

use thiserror::Error;

#[derive(Debug, Error)]
enum MachineError {
    #[error("Invalid instruction: {0}")]
    InvalidInstruction(Value),
    #[error("Invalid parameter mode: {0}")]
    InvalidParameterMode(Value),
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
    #[must_use]
    const fn execute(self, machine: &mut Machine) -> Option<Value> {
        match self {
            Self::Halt => machine.stopped = true,
        }
        None
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
    #[must_use]
    fn execute(self, arg1: ArgumentBy, machine: &mut Machine) -> Option<Value> {
        match self {
            Self::Input => {
                let value = machine.read_input();
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
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum OpCode2 {
    JumpIfTrue = 3,
    JumpIfFalse = 4,
}

impl OpCode2 {
    #[must_use]
    fn execute(self, arg1: ArgumentBy, arg2: ArgumentBy, machine: &Machine) -> Option<Value> {
        match self {
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
        }
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
    #[must_use]
    fn execute(
        self,
        arg1: ArgumentBy,
        arg2: ArgumentBy,
        arg3: ArgumentBy,
        machine: &mut Machine,
    ) -> Option<Value> {
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
        None
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

#[derive(Debug)]
struct Machine {
    memory: Vec<Value>,
    ip: Value,
    stopped: bool,
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
            stopped: false,
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
        self.stopped = false;
        self.inputs.clear();
        self.outputs.clear();
    }

    fn read_input(&mut self) -> Value {
        self.inputs.pop_front().unwrap()
    }

    fn write_output(&mut self, value: Value) {
        self.outputs.push_back(value);
    }

    fn step(&mut self) {
        if self.stopped {
            return;
        }
        let op = self.get_op();
        match op {
            OpCode::Nonary(op) => {
                if self.log {
                    println!("[{}] {op:?}", self.ip);
                }
                self.ip = op.execute(self).unwrap_or(self.ip + 1);
            }
            OpCode::Unary(op, p1) => {
                let arg1 = self.get_arg(1, p1);
                if self.log {
                    println!("[{}] {op:?} {arg1}", self.ip);
                }
                self.ip = op.execute(arg1, self).unwrap_or(self.ip + 2);
            }
            OpCode::Binary(op, p1, p2) => {
                let arg1 = self.get_arg(1, p1);
                let arg2 = self.get_arg(2, p2);
                if self.log {
                    println!("[{}] {op:?} {arg1} {arg2}", self.ip);
                }
                self.ip = op.execute(arg1, arg2, self).unwrap_or(self.ip + 3);
            }
            OpCode::Trinary(op, p1, p2, p3) => {
                let arg1 = self.get_arg(1, p1);
                let arg2 = self.get_arg(2, p2);
                let arg3 = self.get_arg(3, p3);
                if self.log {
                    println!("[{}] {op:?} {arg1} {arg2} {arg3}", self.ip);
                }
                self.ip = op.execute(arg1, arg2, arg3, self).unwrap_or(self.ip + 4);
            }
        }
    }

    fn run_until_output(&mut self) -> Option<Value> {
        while !self.stopped && self.outputs.is_empty() {
            self.step();
        }
        self.outputs.pop_front()
    }
}

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
        if let Some(new_color) = self.machine.run_until_output() {
            self.painter.paint(new_color.try_into()?);
        }
        if let Some(turn) = self.machine.run_until_output() {
            self.painter.turn(turn.try_into()?);
        }
        Ok(())
    }

    fn run_until_completion(&mut self) -> Result<usize, AntError> {
        while !self.machine.stopped {
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
