use std::num::ParseIntError;

use thiserror::Error;

use crate::machine::{Machine, MachineError, Value, parse_program};

#[aoc_generator(day23)]
fn parse(input: &str) -> Result<Vec<Value>, ParseIntError> {
    parse_program(input)
}

#[aoc(day23, part1)]
fn part_1(program: &[Value]) -> Value {
    let mut sim = NetworkSimulator::new(program, 50).unwrap();
    let (_, y) = sim.run_until_first_nat_package().unwrap().unwrap();
    y
}

#[aoc(day23, part2)]
fn part_2(program: &[Value]) -> Value {
    let mut sim = NetworkSimulator::new(program, 50).unwrap();
    let (_, y) = sim.run_with_nat().unwrap().unwrap();
    y
}

#[derive(Debug, Error)]
enum RuntimeError {
    #[error("Network is idle, but no NAT package stored")]
    NoNatPackage,
    #[error(transparent)]
    MachineError(#[from] MachineError),
}

#[derive(Debug, Clone)]
struct NetworkSimulator {
    machines: Vec<Machine>,
    nat_package: Option<(Value, Value)>,
}

impl NetworkSimulator {
    fn new(program: &[Value], count: usize) -> Result<Self, MachineError> {
        Ok(Self {
            machines: (0..count)
                .map(|address| {
                    let mut machine = Machine::new(program);
                    machine.inputs.push_back(Value::try_from(address).unwrap());
                    machine.run_until_input()?;
                    Ok(machine)
                })
                .collect::<Result<_, MachineError>>()?,
            nat_package: None,
        })
    }

    fn run_until_first_nat_package(&mut self) -> Result<Option<(Value, Value)>, RuntimeError> {
        loop {
            for machine_ix in 0..self.machines.len() {
                self.process_machine(machine_ix)?;
                if let Some(nat_package) = self.nat_package {
                    return Ok(Some(nat_package));
                }
            }
        }
    }

    fn run_with_nat(&mut self) -> Result<Option<(Value, Value)>, RuntimeError> {
        let mut prev_nat_package = None;
        loop {
            let mut any_activity = false;
            for machine_ix in 0..self.machines.len() {
                any_activity = self.process_machine(machine_ix)? || any_activity;
            }
            if !any_activity {
                if let Some((x, y)) = self.nat_package {
                    if prev_nat_package == Some((x, y)) {
                        return Ok(Some((x, y)));
                    }
                    prev_nat_package = Some((x, y));
                    self.send_package(0, x, y);
                } else {
                    return Err(RuntimeError::NoNatPackage);
                }
            }
        }
    }

    fn process_machine(&mut self, machine_ix: usize) -> Result<bool, RuntimeError> {
        let machine = &mut self.machines[machine_ix];
        if machine.inputs.is_empty() {
            machine.inputs.push_back(-1);
        }
        machine.run_until_input()?;
        if machine.outputs.len() < 3 {
            return Ok(false);
        }
        let outputs = machine
            .outputs
            .drain(..machine.outputs.len() / 3 * 3)
            .collect::<Vec<_>>();
        for ((&dest, &x), &y) in outputs
            .iter()
            .zip(&outputs[1..])
            .zip(&outputs[2..])
            .step_by(3)
        {
            self.send_package(dest, x, y);
        }
        Ok(true)
    }

    fn send_package(&mut self, dest: Value, x: Value, y: Value) {
        if dest == 255 {
            self.nat_package = Some((x, y));
            return;
        }
        if let Ok(ix) = usize::try_from(dest)
            && let Some(machine) = self.machines.get_mut(ix)
        {
            machine.inputs.push_back(x);
            machine.inputs.push_back(y);
        }
    }
}

// No test cases