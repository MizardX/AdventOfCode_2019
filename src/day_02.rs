use std::num::ParseIntError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OpCode {
    Nonary(OpCode0),
    Trinary(OpCode3),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
enum OpCode0 {
    Halt = 99,
}

impl OpCode0 {
    const fn execute(self, machine: &mut Machine) {
        match self {
            Self::Halt => machine.ip = machine.memory.len(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
enum OpCode3 {
    Add = 1,
    Mul = 2,
}

impl OpCode3 {
    fn execute(self, arg1: u32, arg2: u32, arg3: u32, machine: &mut Machine) {
        match self {
            Self::Add => machine.write(arg3, machine.read(arg1) + machine.read(arg2)),
            Self::Mul => machine.write(arg3, machine.read(arg1) * machine.read(arg2)),
        }
    }
}

impl TryFrom<u32> for OpCode {
    type Error = u32;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            1 => Self::Trinary(OpCode3::Add),
            2 => Self::Trinary(OpCode3::Mul),
            99 => Self::Nonary(OpCode0::Halt),
            _ => return Err(value),
        })
    }
}

#[aoc_generator(day2)]
fn parse(input: &str) -> Result<Vec<u32>, ParseIntError> {
    input
        .split(',')
        .map(str::trim_ascii)
        .map(str::parse)
        .collect()
}

#[aoc(day2, part1)]
fn part_1(program: &[u32]) -> u32 {
    let mut machine = Machine::new(program);
    machine.write(1, 12);
    machine.write(2, 2);
    machine.run();
    machine.read(0)
}

#[aoc(day2, part2)]
fn part_2(program: &[u32]) -> u32 {
    let mut machine = Machine::new(program);
    for noun in 0..=99 {
        for verb in 0..=99 {
            machine.reset(program);
            machine.write(1, noun);
            machine.write(2, verb);
            machine.run();
            if machine.read(0) == 19_690_720 {
                return 100 * noun + verb;
            }
        }
    }
    0
}

#[derive(Debug)]
struct Machine {
    memory: Vec<u32>,
    ip: usize,
    log: bool,
}

impl Machine {
    fn new(program: &[u32]) -> Self {
        Self {
            memory: program.to_vec(),
            ip: 0,
            log: false,
        }
    }

    fn get_arg(&self, offset: usize) -> u32 {
        self.memory[self.ip + offset]
    }

    fn get_op(&self) -> OpCode {
        self.memory[self.ip].try_into().expect("Invalid opcode")
    }

    fn read(&self, index: u32) -> u32 {
        self.memory.get(index as usize).copied().unwrap_or(0)
    }

    fn write(&mut self, index: u32, value: u32) {
        self.memory[index as usize] = value;
    }

    fn reset(&mut self, program: &[u32]) {
        self.memory.copy_from_slice(program);
        self.ip = 0;
    }

    fn step(&mut self) {
        let op = self.get_op();
        match op {
            OpCode::Nonary(op) => {
                if self.log {
                    println!("[{}] {op:?}", self.ip);
                }
                op.execute(self);
                self.ip += 1;
            }
            OpCode::Trinary(op) => {
                let arg1 = self.get_arg(1);
                let arg2 = self.get_arg(2);
                let arg3 = self.get_arg(3);
                if self.log {
                    println!("[{}] {op:?} {arg1} {arg2} {arg3}", self.ip);
                }
                op.execute(arg1, arg2, arg3, self);
                self.ip += 4;
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
    fn test_machine_run(input: &str) -> Vec<u32> {
        let program = parse(input).unwrap();
        let mut machine = Machine::new(&program);
        machine.run();
        machine.memory
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
