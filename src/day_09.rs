use std::collections::VecDeque;
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

#[aoc_generator(day9)]
fn parse(input: &str) -> Result<Vec<Value>, ParseIntError> {
    input
        .split(',')
        .map(str::trim_ascii)
        .map(str::parse)
        .collect()
}

#[aoc(day9, part1)]
fn part_1(program: &[Value]) -> Value {
    let mut machine = Machine::new(program);
    machine.inputs.push_back(1);
    machine.run_until_stopped();
    machine.outputs.pop_back().unwrap()
}

#[aoc(day9, part2)]
fn part_2(program: &[Value]) -> Value {
    let mut machine = Machine::new(program);
    machine.inputs.push_back(2);
    machine.run_until_stopped();
    machine.outputs.pop_back().unwrap()
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

    fn run_until_stopped(&mut self) {
        while !self.stopped {
            self.step();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    const EXAMPLE1: &str = "109,1,204,-1,1001,100,1,100,1008,100,16,101,1006,101,0,99";
    const EXAMPLE2: &str = "1102,34915192,34915192,7,4,7,99,0";
    const EXAMPLE3: &str = "104,1125899906842624,99";

    #[test_case(EXAMPLE1 => &[109,1,204,-1,1001,100,1,100,1008,100,16,101,1006,101,0,99][..]; "Example 1")]
    #[test_case(EXAMPLE2 => &[34_915_192*34_915_192][..]; "Example 2")]
    #[test_case(EXAMPLE3 => &[1_125_899_906_842_624][..]; "Example 3")]
    fn test_machine(input: &str) -> Vec<Value> {
        let program = parse(input).unwrap();
        let mut machine = Machine::new(&program);
        machine.run_until_stopped();
        machine.outputs.into()
    }
}
