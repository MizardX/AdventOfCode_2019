use std::collections::VecDeque;
use std::fmt::Display;
use std::num::ParseIntError;

use thiserror::Error;

pub type Value = i64;

#[derive(Debug, Error)]
pub enum MachineError {
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
pub enum State {
    Running,
    Stopped,
}

#[derive(Debug, Clone)]
pub struct Machine {
    memory: Vec<Value>,
    ip: Value,
    state: State,
    pub log: bool,
    pub inputs: VecDeque<Value>,
    pub outputs: VecDeque<Value>,
    relative_base: Value,
}

impl Machine {
    pub fn new(program: &[Value]) -> Self {
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

    pub const fn state(&self) -> State {
        self.state
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

    pub fn read(&self, index: Value) -> Value {
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

    pub fn write(&mut self, index: Value, value: Value) {
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

    pub fn reset(&mut self, program: &[Value]) {
        self.memory.resize(program.len(), 0);
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

    pub fn run_until_stopped(&mut self) -> Result<(), MachineError> {
        while self.state == State::Running {
            self.step()?;
        }
        Ok(())
    }

    pub fn run_until_output(&mut self) -> Result<Option<Value>, MachineError> {
        while self.outputs.is_empty() {
            self.step()?;
        }
        Ok(self.outputs.pop_front())
    }

    pub fn run_until_input(&mut self) -> Result<(), MachineError> {
        loop {
            match self.step() {
                Ok(()) => (),
                Err(MachineError::EmptyInput) => return Ok(()),
                Err(err) => return Err(err),
            }
        }
    }

    #[allow(unused, reason = "tests")]
    pub fn into_memory(self) -> Vec<Value> {
        self.memory
    }
}

pub fn parse_program(input: &str) -> Result<Vec<Value>, ParseIntError> {
    input.split(',').map(str::parse).collect()
}
