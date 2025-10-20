use std::fmt::{Display, Write};
use std::num::ParseIntError;

use crate::machine::{Machine, MachineError, Value, parse_program};

#[aoc_generator(day25)]
fn parse(input: &str) -> Result<Vec<Value>, ParseIntError> {
    parse_program(input)
}

#[aoc(day25, part1)]
fn part_1(program: &[Value]) -> u64 {
    let mut mud = DroidMud::new(program);
    mud.run().unwrap()
}

#[derive(Debug, Clone)]
enum Action<'a> {
    North,
    East,
    South,
    West,
    TakeItem(&'a str),
    DropItem(&'a str),
    #[allow(unused)]
    Inventory,
}

impl Display for Action<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::North => f.write_str("north"),
            Self::East => f.write_str("east"),
            Self::South => f.write_str("south"),
            Self::West => f.write_str("west"),
            Self::TakeItem(name) => write!(f, "take {name}"),
            Self::DropItem(name) => write!(f, "drop {name}"),
            Self::Inventory => f.write_str("inv"),
        }
    }
}

struct DroidMud {
    machine: Machine,
    log: bool,
}

impl DroidMud {
    fn new(program: &[Value]) -> Self {
        Self {
            machine: Machine::new(program),
            log: false,
        }
    }

    fn get_output(&mut self) -> String {
        match self.machine.run_until_input() {
            Ok(()) | Err(MachineError::Stopped) => (),
            Err(err) => println!("ERROR: {err}"),
        }

        let output = self
            .machine
            .outputs
            .drain(..)
            .filter_map(|v| {
                u8::try_from(v).map_or_else(
                    |_| {
                        println!("INVALID OUTPUT: {v}");
                        None
                    },
                    Some,
                )
            })
            .collect::<Vec<_>>();
        let text = str::from_utf8(&output).unwrap().trim_ascii();
        if self.log {
            println!("{text}");
        }
        text.to_string()
    }

    fn execute(&mut self, action: &Action) -> String {
        if self.log {
            println!("> {action}");
        }
        writeln!(&mut self.machine, "{action}").unwrap();
        self.get_output()
    }

    fn run(&mut self) -> Option<u64> {
        let actions = [
            Action::East,
            Action::TakeItem("weather machine"),
            Action::West,
            Action::West,
            //Action::TakeItem("giant electromagnet"),
            Action::West,
            Action::TakeItem("bowl of rice"),
            Action::East,
            Action::North,
            Action::TakeItem("polygon"),
            Action::East,
            Action::TakeItem("hypercube"),
            Action::South,
            Action::TakeItem("dark matter"),
            Action::West,
            Action::East,
            Action::North,
            Action::West,
            Action::North,
            Action::TakeItem("candy cane"),
            Action::North,
            //Action::TakeItem("escape pod"),
            Action::South,
            Action::West,
            //Action::TakeItem("molten lava"),
            Action::North,
            Action::TakeItem("manifold"),
            Action::West,
            //Action::TakeItem("infinite loop"),
            Action::East,
            Action::South,
            Action::West,
            Action::North,
            Action::TakeItem("dehydrated water"),
            Action::West,
        ];
        self.get_output();
        let mut inventory = Vec::new();
        for action in &actions {
            self.execute(action);
            if let Action::TakeItem(item) = action {
                inventory.push(item);
            }
        }

        let mut inventory_status = vec![true; inventory.len()];

        let mut index: u32 = 1;
        let mut prev_gray_code = 0;
        let mut output = self.execute(&Action::South);
        while output.contains("Alert!") {
            index += 1;
            let gray_code = index ^ (index >> 1);
            let toggled_item = (gray_code ^ prev_gray_code).trailing_zeros() as usize;
            prev_gray_code = gray_code;

            if inventory_status[toggled_item] {
                self.execute(&Action::DropItem(inventory[toggled_item]));
            } else {
                self.execute(&Action::TakeItem(inventory[toggled_item]));
            }
            inventory_status[toggled_item] ^= true;
            output = self.execute(&Action::South);
        }
        output
            .split_ascii_whitespace()
            .find_map(|word| word.parse::<u64>().ok())
    }
}
