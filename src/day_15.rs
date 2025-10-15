use std::collections::{HashMap, HashSet, VecDeque};
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
    input.split(',').map(str::parse).collect()
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
