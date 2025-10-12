use std::collections::VecDeque;
use std::fmt::Display;
use std::num::ParseIntError;

use thiserror::Error;

#[derive(Debug, Error)]
enum MachineError {
    #[error("Invalid instruction: {0}")]
    InvalidInstruction(i32),
    #[error("Invalid parameter mode: {0}")]
    InvalidParameterMode(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum ParameterMode {
    Position = 0,
    Immediate = 1,
}

impl TryFrom<i32> for ParameterMode {
    type Error = MachineError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Ok(match value % 10 {
            0 => Self::Position,
            1 => Self::Immediate,
            _ => return Err(MachineError::InvalidParameterMode(value)),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Argument {
    ByPosition(i32),
    ByValue(i32),
}

impl Argument {
    fn read(self, machine: &Machine) -> i32 {
        match self {
            Self::ByPosition(index) => machine.read(index),
            Self::ByValue(val) => val,
        }
    }

    fn write(self, value: i32, machine: &mut Machine) {
        if let Self::ByPosition(index) = self {
            machine.write(index, value);
        } else {
            panic!("Trying to write into immediate value");
        }
    }
}

impl Display for Argument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::ByPosition(pos) => write!(f, "#{pos}"),
            Self::ByValue(val) => write!(f, "{val}"),
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
    #[expect(clippy::unnecessary_wraps, reason = "match pattern")]
    const fn execute(self, machine: &Machine) -> Option<usize> {
        match self {
            Self::Halt => Some(machine.memory.len()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum OpCode1 {
    Input = 3,
    Output = 4,
}

impl OpCode1 {
    #[must_use]
    fn execute(self, arg1: Argument, machine: &mut Machine) -> Option<usize> {
        match self {
            Self::Input => {
                let value = machine.read_input();
                arg1.write(value, machine);
            }
            Self::Output => {
                let value = arg1.read(machine);
                machine.write_output(value);
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
    fn execute(self, arg1: Argument, arg2: Argument, machine: &Machine) -> Option<usize> {
        match self {
            Self::JumpIfTrue => {
                let condition = arg1.read(machine);
                if condition != 0 {
                    usize::try_from(arg2.read(machine)).ok()
                } else {
                    None
                }
            }
            Self::JumpIfFalse => {
                let condition = arg1.read(machine);
                if condition == 0 {
                    usize::try_from(arg2.read(machine)).ok()
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
        arg1: Argument,
        arg2: Argument,
        arg3: Argument,
        machine: &mut Machine,
    ) -> Option<usize> {
        match self {
            Self::Add => arg3.write(arg1.read(machine) + arg2.read(machine), machine),
            Self::Mul => arg3.write(arg1.read(machine) * arg2.read(machine), machine),
            Self::LessThan => {
                arg3.write(i32::from(arg1.read(machine) < arg2.read(machine)), machine);
            }
            Self::Equals => {
                arg3.write(i32::from(arg1.read(machine) == arg2.read(machine)), machine);
            }
        }
        None
    }
}

impl TryFrom<i32> for OpCode {
    type Error = MachineError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
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
            code @ (3 | 4) => Self::Unary(
                match code {
                    3 => OpCode1::Input,
                    4 => OpCode1::Output,
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

#[aoc_generator(day5)]
fn parse(input: &str) -> Result<Vec<i32>, ParseIntError> {
    input
        .split(',')
        .map(str::trim_ascii)
        .map(str::parse)
        .collect()
}

#[aoc(day5, part1)]
fn part_1(program: &[i32]) -> i32 {
    let mut machine = Machine::new(program);
    machine.inputs.push_back(1);
    machine.run();
    machine.outputs.pop_back().unwrap()
}

#[aoc(day5, part2)]
fn part_2(program: &[i32]) -> i32 {
    let mut machine = Machine::new(program);
    machine.inputs.push_back(5);
    machine.run();
    machine.outputs.pop_back().unwrap()
}

#[derive(Debug)]
struct Machine {
    memory: Vec<i32>,
    ip: usize,
    log: bool,
    inputs: VecDeque<i32>,
    outputs: VecDeque<i32>,
}

impl Machine {
    fn new(program: &[i32]) -> Self {
        Self {
            memory: program.to_vec(),
            ip: 0,
            log: false,
            inputs: VecDeque::new(),
            outputs: VecDeque::new(),
        }
    }

    fn get_arg(&self, offset: usize, mode: ParameterMode) -> Argument {
        let value = self.memory[self.ip + offset];
        match mode {
            ParameterMode::Position => Argument::ByPosition(value),
            ParameterMode::Immediate => Argument::ByValue(value),
        }
    }

    fn get_op(&self) -> OpCode {
        self.memory[self.ip].try_into().expect("Invalid opcode")
    }

    fn read(&self, index: i32) -> i32 {
        if let Ok(index) = usize::try_from(index)
            && let Some(mem) = self.memory.get(index)
        {
            *mem
        } else {
            0
        }
    }

    fn write(&mut self, index: i32, value: i32) {
        if let Ok(index) = usize::try_from(index)
            && let Some(mem) = self.memory.get_mut(index)
        {
            *mem = value;
        }
    }

    #[expect(unused, reason = "Not needed in this problem")]
    fn reset(&mut self, program: &[i32]) {
        self.memory.copy_from_slice(program);
        self.ip = 0;
    }

    fn read_input(&mut self) -> i32 {
        self.inputs.pop_front().unwrap()
    }

    fn write_output(&mut self, value: i32) {
        self.outputs.push_back(value);
    }

    fn step(&mut self) {
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

    fn run(&mut self) {
        while self.ip < self.memory.len() {
            self.step();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test]
    fn test_input_output() {
        let program = [3, 0, 4, 0, 99];
        let mut machine = Machine::new(&program);
        machine.inputs.push_back(123);
        machine.run();
        assert_eq!(machine.outputs.pop_back().unwrap(), 123);
    }

    #[test_case("1002,4,3,4,33" => &[1002,4,3,4,99][..])]
    #[test_case("1101,100,-1,4,0" => &[1101,100,-1,4,99][..])]
    fn test_part_1(input: &str) -> Vec<i32> {
        let program = parse(input).unwrap();
        let mut machine = Machine::new(&program);
        machine.log = true;
        machine.run();
        machine.memory
    }

    const LARGER_EXAMPLE: &str = "3,21,1008,21,8,20,1005,20,22,107,8,21,20,1006,20,31,1106,0,36,98,0,0,1002,21,125,20,4,20,1105,1,46,104,999,1105,1,46,1101,1000,1,20,4,20,1105,1,46,98,99";

    #[test_case("3,9,8,9,10,9,4,9,99,-1,8", 8 => &[1][..])]
    #[test_case("3,9,8,9,10,9,4,9,99,-1,8", 7 => &[0][..])]
    #[test_case("3,9,7,9,10,9,4,9,99,-1,8", 8 => &[0][..])]
    #[test_case("3,9,7,9,10,9,4,9,99,-1,8", 7 => &[1][..])]
    #[test_case("3,3,1108,-1,8,3,4,3,99", 8 => &[1][..])]
    #[test_case("3,3,1108,-1,8,3,4,3,99", 7 => &[0][..])]
    #[test_case("3,3,1107,-1,8,3,4,3,99", 8 => &[0][..])]
    #[test_case("3,3,1107,-1,8,3,4,3,99", 7 => &[1][..])]
    #[test_case("3,12,6,12,15,1,13,14,13,4,13,99,-1,0,1,9", 0 => &[0][..])]
    #[test_case("3,12,6,12,15,1,13,14,13,4,13,99,-1,0,1,9", 123 => &[1][..])]
    #[test_case("3,3,1105,-1,9,1101,0,0,12,4,12,99,1", 0 => &[0][..])]
    #[test_case("3,3,1105,-1,9,1101,0,0,12,4,12,99,1", 123 => &[1][..])]
    #[test_case(LARGER_EXAMPLE, 1 => &[999][..])]
    #[test_case(LARGER_EXAMPLE, 8 => &[1000][..])]
    #[test_case(LARGER_EXAMPLE, 123 => &[1001][..])]
    fn test_parameter_mode(program: &str, input: i32) -> Vec<i32> {
        let program = parse(program).unwrap();
        let mut machine = Machine::new(&program);
        machine.inputs.push_back(input);
        machine.run();
        machine.outputs.into()
    }
}
