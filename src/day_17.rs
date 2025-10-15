use std::collections::VecDeque;
use std::fmt::{Display, Write};
use std::num::ParseIntError;
use std::ops::{Add, AddAssign, Index};

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

    fn run_until_stopped(&mut self) -> Result<(), MachineError> {
        while self.state == State::Running {
            self.step()?;
        }
        Ok(())
    }
}

#[aoc_generator(day17)]
fn parse(input: &str) -> Result<Vec<Value>, ParseIntError> {
    input.split(',').map(str::parse).collect()
}

#[aoc(day17, part1)]
fn part_1(program: &[Value]) -> usize {
    let mut machine = Machine::new(program);
    let map = read_map(&mut machine).unwrap();
    sum_alignment_parameters(&map)
}

fn read_map(machine: &mut Machine) -> Result<Map<u8>, MachineError> {
    match machine.run_until_stopped() {
        Ok(()) | Err(MachineError::EmptyInput) => {}
        err => err?,
    }
    let mut output = Vec::new();
    let mut line_len = 0;
    while let Some(x) = machine.outputs.pop_front() {
        let ch = u8::try_from(x).unwrap();
        if ch == b'\n' {
            if line_len == 0 {
                break;
            }
            line_len = 0;
        } else {
            line_len += 1;
        }
        output.push(ch);
    }
    Ok(Map::new(output, |&ch| ch == b'\n', b' '))
}

struct Map<T> {
    data: Vec<T>,
    fallback: T,
    stride: usize,
    width: usize,
    height: usize,
}

impl<T> Map<T> {
    fn new(data: Vec<T>, split: impl Fn(&T) -> bool, fallback: T) -> Self {
        let width = data.iter().position(split).unwrap();
        let stride = width + 1;
        let height = (data.len() + 1) / stride;
        Self {
            data,
            fallback,
            stride,
            width,
            height,
        }
    }

    fn index_to_pos(&self, index: usize) -> Position {
        Position::new(
            Value::try_from(index % self.stride).unwrap(),
            Value::try_from(index / self.stride).unwrap(),
        )
    }
}

impl<T> Index<Position> for Map<T> {
    type Output = T;

    fn index(&self, index: Position) -> &Self::Output {
        if let Ok(x) = usize::try_from(index.x)
            && let Ok(y) = usize::try_from(index.y)
            && (0..self.width).contains(&x)
            && (0..self.height).contains(&y)
        {
            &self.data[x + self.stride * y]
        } else {
            &self.fallback
        }
    }
}

fn sum_alignment_parameters(map: &Map<u8>) -> usize {
    let mut alignment_sum = 0;
    for y in 1..map.height - 1 {
        for x in 1..map.width - 1 {
            let pos = Position::new(i64::try_from(x).unwrap(), i64::try_from(y).unwrap());
            if map[pos] == b'#'
                && map[pos + Direction::Up] == b'#'
                && map[pos + Direction::Left] == b'#'
                && map[pos + Direction::Right] == b'#'
                && map[pos + Direction::Down] == b'#'
            {
                alignment_sum += x * y;
            }
        }
    }
    alignment_sum
}

#[aoc(day17, part2)]
fn part_2(program: &[Value]) -> Value {
    let mut machine = Machine::new(program);
    machine.write(0, 2);

    let map = read_map(&mut machine).unwrap();

    let path = collect_path(&map);

    let subdiv = PathSubdivision::subdivide_path(&path).unwrap();
    let mut program_text = subdiv.to_string();
    program_text.push_str("n\n");

    machine.inputs.extend(program_text.bytes().map(Value::from));

    machine.run_until_stopped().unwrap();

    machine.outputs.pop_back().unwrap()
}

fn collect_path(map: &Map<u8>) -> Vec<Action> {
    const fn is_open(ch: u8) -> bool {
        matches!(ch, b'#' | b'<' | b'^' | b'>' | b'v')
    }
    let (mut dir, mut pos) = map
        .data
        .iter()
        .enumerate()
        .find_map(|(ix, &ch)| Some((Direction::try_from(ch).ok()?, map.index_to_pos(ix))))
        .unwrap();
    let mut path = Vec::new();
    loop {
        let mut forward_count = 0;
        while is_open(map[pos + dir]) {
            pos += dir;
            forward_count += 1;
        }
        if forward_count > 0 {
            path.push(Action::Forward(forward_count));
        }
        if is_open(map[pos + dir.turn_left()]) {
            dir = dir.turn_left();
            path.push(Action::Left);
        } else if is_open(map[pos + dir.turn_right()]) {
            dir = dir.turn_right();
            path.push(Action::Right);
        } else {
            // End of the path
            break;
        }
    }
    path
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Subroutine {
    A,
    B,
    C,
}

impl Subroutine {
    const fn all() -> [Self; 3] {
        [Self::A, Self::B, Self::C]
    }
}

impl Display for Subroutine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char(match self {
            Self::A => 'A',
            Self::B => 'B',
            Self::C => 'C',
        })
    }
}

#[derive(Debug, Clone, Default)]
struct PathSubdivision {
    main: Vec<Subroutine>,
    subroutines: [Vec<Action>; Subroutine::all().len()],
}

impl PathSubdivision {
    fn walk(&mut self, path: &[Action]) -> bool {
        if path.is_empty() {
            return self.main.len() * 2 - 1 <= 20
                && self
                    .subroutines
                    .iter()
                    .all(|s| s.iter().map(|a| a.len() + 1).sum::<usize>() - 1 <= 20);
        }
        for sub in Subroutine::all() {
            let sub_ix = sub as usize;
            if self.subroutines[sub_ix].is_empty() {
                self.main.push(sub);
                for (path_ix, &action) in path.iter().enumerate() {
                    self.subroutines[sub_ix].push(action);
                    if self.walk(&path[path_ix + 1..]) {
                        return true;
                    }
                }
                self.subroutines[sub_ix].clear();
                self.main.pop();
                return false;
            }
            if path.starts_with(&self.subroutines[sub_ix]) {
                self.main.push(sub);
                if self.walk(&path[self.subroutines[sub_ix].len()..]) {
                    return true;
                }
                self.main.pop();
            }
        }
        false
    }

    fn subdivide_path(path: &[Action]) -> Option<Self> {
        let mut subdiv = Self::default();
        subdiv.walk(path).then_some(subdiv)
    }
}

impl Display for PathSubdivision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, sub) in self.main.iter().enumerate() {
            if i > 0 {
                f.write_char(',')?;
            }
            write!(f, "{sub}")?;
        }
        writeln!(f)?;
        for sub in &self.subroutines {
            for (i, action) in sub.iter().enumerate() {
                if i > 0 {
                    f.write_char(',')?;
                }
                write!(f, "{action}")?;
            }
            writeln!(f)?;
        }
        Ok(())
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
enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Direction {
    const fn turn_left(self) -> Self {
        match self {
            Self::Up => Self::Left,
            Self::Right => Self::Up,
            Self::Down => Self::Right,
            Self::Left => Self::Down,
        }
    }
    const fn turn_right(self) -> Self {
        match self {
            Self::Up => Self::Right,
            Self::Right => Self::Down,
            Self::Down => Self::Left,
            Self::Left => Self::Up,
        }
    }
}

impl TryFrom<u8> for Direction {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            b'<' => Self::Left,
            b'^' => Self::Up,
            b'>' => Self::Right,
            b'v' => Self::Down,
            _ => return Err(()),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    Left,
    Right,
    Forward(u8),
}

impl Action {
    const fn len(self) -> usize {
        match self {
            Self::Left | Self::Right | Self::Forward(0..=9) => 1,
            Self::Forward(..) => 2,
        }
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Left => f.write_char('L'),
            Self::Right => f.write_char('R'),
            Self::Forward(n) => write!(f, "{n}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE1: &str = "\
        ..#..........\n\
        ..#..........\n\
        #######...###\n\
        #.#...#...#.#\n\
        #############\n\
        ..#...#...#..\n\
        ..#####...^..\
    ";

    const EXAMPLE2: &str = "\
        #######...#####\n\
        #.....#...#...#\n\
        #.....#...#...#\n\
        ......#...#...#\n\
        ......#...###.#\n\
        ......#.....#.#\n\
        ^########...#.#\n\
        ......#.#...#.#\n\
        ......#########\n\
        ........#...#..\n\
        ....#########..\n\
        ....#...#......\n\
        ....#...#......\n\
        ....#...#......\n\
        ....#####......\
    ";

    #[test]
    fn test_part_1() {
        let map = Map::new(EXAMPLE1.as_bytes().to_vec(), |&ch| ch == b'\n', b' ');
        let result = sum_alignment_parameters(&map);
        assert_eq!(result, 76);
    }

    #[test]
    fn test_find_path() {
        let map = Map::new(EXAMPLE2.as_bytes().to_vec(), |&ch| ch == b'\n', b' ');
        let path = collect_path(&map);
        let mut displayed = String::new();
        for action in path {
            if !displayed.is_empty() {
                displayed.push(',');
            }
            write!(&mut displayed, "{action}").unwrap();
        }

        assert_eq!(
            displayed,
            "R,8,R,8,R,4,R,4,R,8,L,6,L,2,R,4,R,4,R,8,R,8,R,8,L,6,L,2"
        );
    }

    #[test]
    fn test_subdivide() {
        let map = Map::new(EXAMPLE2.as_bytes().to_vec(), |&ch| ch == b'\n', b' ');
        let path = collect_path(&map);
        let subdiv = PathSubdivision::subdivide_path(&path).unwrap();
        let text = subdiv.to_string();
        for line in text.lines() {
            assert!(line.len() <= 20, "len <= 20: {line:?}");
        }
        let mut reconstucted = Vec::new();
        for &sub in &subdiv.main {
            reconstucted.extend_from_slice(&subdiv.subroutines[sub as usize]);
        }
        assert_eq!(path, reconstucted);
    }
}
