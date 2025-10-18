use std::num::ParseIntError;

use crate::machine::{parse_program, Machine, Value};

#[aoc_generator(day2)]
fn parse(input: &str) -> Result<Vec<Value>, ParseIntError> {
    parse_program(input)
}

#[aoc(day2, part1)]
fn part_1(program: &[Value]) -> Value {
    let mut machine = Machine::new(program);
    machine.write(1, 12);
    machine.write(2, 2);
    machine.run_until_stopped().unwrap();
    machine.read(0)
}

#[aoc(day2, part2)]
fn part_2(program: &[Value]) -> Value {
    let mut machine = Machine::new(program);
    for noun in 0..=99 {
        for verb in 0..=99 {
            machine.reset(program);
            machine.write(1, noun);
            machine.write(2, verb);
            machine.run_until_stopped().unwrap();
            if machine.read(0) == 19_690_720 {
                return 100 * noun + verb;
            }
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    const EXAMPLE1: &str = "1,9,10,3,2,3,11,0,99,30,40,50";
    const EXAMPLE2: &str = "1,0,0,0,99";
    const EXAMPLE3: &str = "2,3,0,3,99";
    const EXAMPLE4: &str = "2,4,4,5,99,0";
    const EXAMPLE5: &str = "1,1,1,4,99,5,6,0,99";

    #[test]
    fn test_parse() {
        let result = parse(EXAMPLE1).unwrap();
        assert_eq!(result, [1, 9, 10, 3, 2, 3, 11, 0, 99, 30, 40, 50]);
    }

    #[test_case(EXAMPLE1 => &[3500,9,10,70,2,3,11,0,99,30,40,50][..])]
    #[test_case(EXAMPLE2 => &[2,0,0,0,99][..])]
    #[test_case(EXAMPLE3 => &[2,3,0,6,99][..])]
    #[test_case(EXAMPLE4=> &[2,4,4,5,99,9801][..])]
    #[test_case(EXAMPLE5 => &[30,1,1,4,2,5,6,0,99][..])]
    fn test_machine_run(input: &str) -> Vec<Value> {
        let program = parse(input).unwrap();
        let mut machine = Machine::new(&program);
        machine.run_until_stopped().unwrap();
        machine.into_memory()
    }

    #[test]
    fn test_part_1() {
        let program = parse(EXAMPLE1).unwrap();
        let reuslt = part_1(&program);
        // Not quite what part 1 demonstrates, but this is the result
        // of applying noun 12 and verb 2 to the first example.
        assert_eq!(reuslt, 100);
    }
}
