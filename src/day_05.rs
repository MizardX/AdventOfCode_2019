use std::num::ParseIntError;

use crate::machine::{parse_program, Machine, Value};

#[aoc_generator(day5)]
fn parse(input: &str) -> Result<Vec<Value>, ParseIntError> {
    parse_program(input)
}

#[aoc(day5, part1)]
fn part_1(program: &[Value]) -> Value {
    let mut machine = Machine::new(program);
    machine.inputs.push_back(1);
    machine.run_until_stopped().unwrap();
    machine.outputs.pop_back().unwrap()
}

#[aoc(day5, part2)]
fn part_2(program: &[Value]) -> Value {
    let mut machine = Machine::new(program);
    machine.inputs.push_back(5);
    machine.run_until_output().unwrap();
    machine.outputs.pop_back().unwrap()
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
        machine.run_until_stopped().unwrap();
        assert_eq!(machine.outputs.pop_back().unwrap(), 123);
    }

    #[test_case("1002,4,3,4,33" => &[1002,4,3,4,99][..])]
    #[test_case("1101,100,-1,4,0" => &[1101,100,-1,4,99][..])]
    fn test_part_1(input: &str) -> Vec<Value> {
        let program = parse(input).unwrap();
        let mut machine = Machine::new(&program);
        machine.log = true;
        machine.run_until_stopped().unwrap();
        machine.into_memory()
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
    fn test_parameter_mode(program: &str, input: Value) -> Vec<Value> {
        let program = parse(program).unwrap();
        let mut machine = Machine::new(&program);
        machine.inputs.push_back(input);
        machine.run_until_stopped().unwrap();
        machine.outputs.into()
    }
}
