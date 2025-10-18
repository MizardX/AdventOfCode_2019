use std::fmt::{Display, Write};
use std::num::ParseIntError;

use crate::machine::{Machine, MachineError, Value, parse_program};

#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Reg {
    /// Temporary
    T = 0,
    /// Jump
    J,
    /// Sensor 1m
    A,
    /// Sensor 2m
    B,
    /// Sensor 3m,
    C,
    /// Sensor 4m,
    D,
    /// Sensor 5m,
    E,
    /// Sensor 6m,
    F,
    /// Sensor 7m,
    G,
    /// Sensor 8m,
    H,
    /// Sensor 9m,
    I,
}

impl Display for Reg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Instruction {
    And(Reg, Reg),
    Or(Reg, Reg),
    Not(Reg, Reg),
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::And(src, dst) => write!(f, "AND {src} {dst}"),
            Self::Or(src, dst) => write!(f, "OR {src} {dst}"),
            Self::Not(src, dst) => write!(f, "NOT {src} {dst}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Walk,
    Run,
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Walk => f.write_str("WALK"),
            Self::Run => f.write_str("RUN"),
        }
    }
}

struct SpringDroid<'a> {
    program: &'a [Value],
    machine: Machine,
}

impl<'a> SpringDroid<'a> {
    fn new(program: &'a [Value]) -> Self {
        Self {
            program,
            machine: Machine::new(program),
        }
    }

    fn execute(
        &mut self,
        instructions: &[Instruction],
        mode: Mode,
    ) -> Result<Option<Value>, MachineError> {
        self.machine.reset(self.program);
        let mut buf = String::new();
        for instr in instructions {
            writeln!(&mut buf, "{instr}").unwrap();
        }
        writeln!(&mut buf, "{mode}").unwrap();
        self.machine.inputs.extend(buf.bytes().map(Value::from));
        self.machine.run_until_stopped()?;
        let mut output = Vec::new();
        for &val in &self.machine.outputs {
            if let Ok(byte) = u8::try_from(val)
                && byte > 0
            {
                output.push(byte);
            } else {
                return Ok(Some(val));
            }
        }
        println!("{}", str::from_utf8(&output).unwrap());
        Ok(None)
    }
}

#[aoc_generator(day21)]
fn parse(input: &str) -> Result<Vec<Value>, ParseIntError> {
    parse_program(input)
}

#[aoc(day21, part1)]
fn part_1(program: &[Value]) -> Value {
    let mut droid = SpringDroid::new(program);
    // When jumping, it will jump to the tile at distnace 4m, same as the 'D' register.
    // The logic is J = (!A | !B | !C) & D
    // That is, if there are any gaps, and a jump is safe, do it.
    droid
        .execute(
            &[
                Instruction::Not(Reg::D, Reg::T),
                Instruction::Or(Reg::A, Reg::T),
                Instruction::And(Reg::B, Reg::T),
                Instruction::And(Reg::C, Reg::T),
                Instruction::Not(Reg::T, Reg::J),
                Instruction::And(Reg::D, Reg::J),
            ],
            Mode::Walk,
        )
        .unwrap()
        .unwrap()
}

#[aoc(day21, part2)]
fn part_2(program: &[Value]) -> Value {
    let mut droid = SpringDroid::new(program);
    // ABCDEFGHI
    // .???????? -- Imminent gap, must jump
    // ??.##???# -- Jump-Step-Jump to exit
    // ??.#???#? -- Jump-Jump-Jump to exit
    // ?.?##???# -- Jump-Step-Jump to exit
    // ?.?#???#? -- Jump-Jump-Jump to exit
    //
    // Combined: .???????? OR ?(?.|.?)#(???#?|#???#)
    //
    // Logic: !A | (!B | !C) & D & (H | E & I)
    //
    // (!B | !C) & D
    // (!D | !B | !C) & D   -- Adding D does not change result.
    // !!(!D | !B | !C) & D -- Double negation.
    // !(D & B & C) & D     -- De Morgan.
    // !(!!D & B & C) & D   -- Double negation, unable to just copy.
    //
    // (H | E & I)
    // (H | !H & E & I)     -- Adding !H does not change result.
    droid
        .execute(
            &[
                Instruction::Not(Reg::H, Reg::J), // J = !H
                Instruction::And(Reg::I, Reg::J), // J = I & !H
                Instruction::And(Reg::E, Reg::J), // J = E & I & !H
                Instruction::Or(Reg::H, Reg::J), // J = H | (E & I & !H) = H | (E & I)
                Instruction::Not(Reg::D, Reg::T), // T = !D
                Instruction::Not(Reg::T, Reg::T), // T = !!D = D
                Instruction::And(Reg::C, Reg::T), // T = C & D
                Instruction::And(Reg::B, Reg::T), // T = B & C & D
                Instruction::Not(Reg::T, Reg::T), // T = !(B & C & D) = (!B | !C | !D)
                Instruction::And(Reg::D, Reg::T), // T = D & (!B | !C | !D) = D & (!B | !C)
                Instruction::And(Reg::T, Reg::J), // J = D & (!B | !C) & (H | (E & I))
                Instruction::Not(Reg::A, Reg::T), // T = !A
                Instruction::Or(Reg::T, Reg::J), // J = !A | D & (!B | !C) & (H | (E & I))
            ],
            Mode::Run,
        )
        .unwrap()
        .unwrap()
}
