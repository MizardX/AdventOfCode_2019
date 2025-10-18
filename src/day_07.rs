use std::num::ParseIntError;

use thiserror::Error;

use crate::machine::{Machine, MachineError, Value, parse_program};

#[aoc_generator(day7)]
fn parse(input: &str) -> Result<Vec<Value>, ParseIntError> {
    parse_program(input)
}

#[aoc(day7, part1)]
fn part_1(program: &[Value]) -> Value {
    let mut amplifier = Amplifiers::new(program);
    let mut max_signal = Value::MIN;
    permute(&mut [0, 1, 2, 3, 4], 0, &mut |phase_settings| {
        amplifier.reset(*phase_settings);
        if let Ok(signal) = amplifier.get_chain_output(0) {
            max_signal = max_signal.max(signal);
        }
    });
    max_signal
}

#[aoc(day7, part2)]
fn part_2(program: &[Value]) -> Value {
    let mut amplifiers = Amplifiers::new(program);
    let mut max_signal = Value::MIN;
    permute(&mut [5, 6, 7, 8, 9], 0, &mut |&phase_settings| {
        amplifiers.reset(phase_settings);
        let mut signal = 0;
        while let Ok(new_signal) = amplifiers.get_chain_output(signal) {
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

#[derive(Debug, Error)]
enum RuntimeError {
    #[error(transparent)]
    MachineError(#[from] MachineError),
    #[error("No output produced")]
    OutputEmpty,
}

struct Amplifiers<'a> {
    program: &'a [Value],
    machines: [Machine; 5],
}

impl<'a> Amplifiers<'a> {
    fn new(program: &'a [Value]) -> Self {
        Self {
            program,
            machines: [(); 5].map(|()| Machine::new(program)),
        }
    }

    fn reset(&mut self, phase_settings: [Value; 5]) {
        for (machine, phase) in self.machines.iter_mut().zip(phase_settings) {
            machine.reset(self.program);
            machine.inputs.push_back(phase);
        }
    }

    fn get_chain_output(&mut self, first_input: Value) -> Result<Value, RuntimeError> {
        let mut signal = first_input;
        for machine in &mut self.machines {
            machine.inputs.push_back(signal);
            signal = machine
                .run_until_output()?
                .ok_or(RuntimeError::OutputEmpty)?;
        }
        Ok(signal)
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
    fn test_part_1(input: &str) -> Value {
        let program = parse(input).unwrap();
        part_1(&program)
    }

    #[test_case(EXAMPLE4 => 139_629_729)]
    #[test_case(EXAMPLE5 => 18_216)]
    fn test_part_2(input: &str) -> Value {
        let program = parse(input).unwrap();
        part_2(&program)
    }
}
