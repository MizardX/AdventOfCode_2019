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

#[aoc_generator(day7)]
fn parse(input: &str) -> Result<Vec<i32>, ParseIntError> {
    input
        .split(',')
        .map(str::trim_ascii)
        .map(str::parse)
        .collect()
}

#[aoc(day7, part1)]
fn part_1(program: &[i32]) -> i32 {
    let mut amplifier = Amplifiers::new(program);
    let mut max_signal = i32::MIN;
    permute(&mut [0, 1, 2, 3, 4], 0, &mut |phase_settings| {
        amplifier.reset(*phase_settings);
        if let Some(signal) = amplifier.get_chain_output(0) {
            max_signal = max_signal.max(signal);
        }
    });
    max_signal
}

#[aoc(day7, part2)]
fn part_2(program: &[i32]) -> i32 {
    let mut amplifiers = Amplifiers::new(program);
    let mut max_signal = i32::MIN;
    permute(&mut [5, 6, 7, 8, 9], 0, &mut |&phase_settings| {
        amplifiers.reset(phase_settings);
        let mut signal = 0;
        while let Some(new_signal) = amplifiers.get_chain_output(signal) {
            signal = new_signal;
        }
        max_signal = max_signal.max(signal);
    });
    max_signal
}

fn permute<const N: usize, T>(items: &mut [T; N], index: usize, report: &mut impl FnMut(&[T; N])) {
    if index == N {
        report(items);
    } else {
        for next in index..N {
            items.swap(index, next);
            permute(items, index + 1, report);
            items.swap(index, next);
        }
    }
}

struct Amplifiers<'a> {
    program: &'a [i32],
    machines: [Machine; 5],
}

impl<'a> Amplifiers<'a> {
    fn new(program: &'a [i32]) -> Self {
        Self {
            program,
            machines: [(); 5].map(|()| Machine::new(program)),
        }
    }

    fn reset(&mut self, phase_settings: [i32; 5]) {
        for (machine, phase) in self.machines.iter_mut().zip(phase_settings) {
            machine.reset(self.program);
            machine.inputs.push_back(phase);
        }
    }

    fn get_chain_output(&mut self, first_input: i32) -> Option<i32> {
        let mut signal = first_input;
        for machine in &mut self.machines {
            machine.inputs.push_back(signal);
            signal = machine.run_until_output()?;
        }
        Some(signal)
    }
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

    fn reset(&mut self, program: &[i32]) {
        self.memory.copy_from_slice(program);
        self.ip = 0;
        self.inputs.clear();
        self.outputs.clear();
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

    fn run_until_output(&mut self) -> Option<i32> {
        while self.ip < self.memory.len() && self.outputs.is_empty() {
            self.step();
        }
        self.outputs.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    const EXAMPLE1: &str = "\
        3,15,3,16,1002,16,10,16,1,16,15,15,4,15,99,0,0\
    ";
    const EXAMPLE2: &str = "\
        3,23,3,24,1002,24,10,24,1002,23,-1,23,\
        101,5,23,23,1,24,23,23,4,23,99,0,0\
    ";
    const EXAMPLE3: &str = "\
        3,31,3,32,1002,32,10,32,1001,31,-2,31,1007,31,0,33,\
        1002,33,7,33,1,33,31,31,1,32,31,31,4,31,99,0,0,0\
    ";

    const EXAMPLE4: &str = "\
        3,26,1001,26,-4,26,3,27,1002,27,2,27,1,27,26,\
        27,4,27,1001,28,-1,28,1005,28,6,99,0,0,5\
    ";
    const EXAMPLE5: &str = "\
        3,52,1001,52,-5,52,3,53,1,52,56,54,1007,54,5,55,1005,55,26,1001,54,\
        -5,54,1105,1,12,1,53,54,53,1008,54,0,55,1001,55,1,55,2,53,55,53,4,\
        53,1001,56,-1,56,1005,56,6,99,0,0,0,0,10\
    ";

    #[test_case(EXAMPLE1 => 43_210)]
    #[test_case(EXAMPLE2 => 54_321)]
    #[test_case(EXAMPLE3 => 65_210)]
    fn test_part_1(input: &str) -> i32 {
        let program = parse(input).unwrap();
        part_1(&program)
    }

    #[test_case(EXAMPLE4 => 139_629_729)]
    #[test_case(EXAMPLE5 => 18_216)]
    fn test_part_2(input: &str) -> i32 {
        let program = parse(input).unwrap();
        part_2(&program)
    }
}
