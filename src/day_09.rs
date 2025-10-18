use std::num::ParseIntError;

use crate::machine::{parse_program, Machine, Value};

#[aoc_generator(day9)]
fn parse(input: &str) -> Result<Vec<Value>, ParseIntError> {
    parse_program(input)
}

#[aoc(day9, part1)]
fn part_1(program: &[Value]) -> Value {
    let mut machine = Machine::new(program);
    machine.inputs.push_back(1);
    machine.run_until_stopped().unwrap();
    machine.outputs.pop_back().unwrap()
}

#[aoc(day9, part2)]
fn part_2(program: &[Value]) -> Value {
    let mut machine = Machine::new(program);
    machine.inputs.push_back(2);
    machine.run_until_stopped().unwrap();
    machine.outputs.pop_back().unwrap()
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
        machine.run_until_stopped().unwrap();
        machine.outputs.into()
    }
}
