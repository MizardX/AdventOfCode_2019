use std::collections::{HashMap, VecDeque};
use std::num::ParseIntError;
use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Error)]
enum ParseError {
    #[error("Syntax error")]
    SyntaxError,
    #[error(transparent)]
    InvalidNumber(#[from] ParseIntError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Chemical {
    Ore,
    Fuel,
    Other(usize),
}

impl Chemical {
    const fn index(self) -> usize {
        match self {
            Self::Ore => 0,
            Self::Fuel => 1,
            Self::Other(ix) => ix,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Reaction {
    requires: Vec<(u64, Chemical)>,
    quantity: u64,
    produces: Chemical,
}

#[derive(Debug, Clone)]
struct ReactionList {
    reactions: Vec<Reaction>,
    num_chemicals: usize,
}

impl FromStr for ReactionList {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut names = HashMap::new();
        names.insert("ORE", Chemical::Ore);
        names.insert("FUEL", Chemical::Fuel);
        let mut reactions = Vec::new();
        for line in s.lines() {
            let (lhs, rhs) = line.split_once(" => ").ok_or(ParseError::SyntaxError)?;
            let requires = lhs
                .split(", ")
                .map(|part| {
                    let (quantity, chemical) =
                        part.split_once(' ').ok_or(ParseError::SyntaxError)?;
                    let quantity = quantity.parse()?;
                    let next_name = names.len();
                    let chemical = *names.entry(chemical).or_insert(Chemical::Other(next_name));
                    Ok::<_, ParseError>((quantity, chemical))
                })
                .collect::<Result<_, _>>()?;
            let (quantity, produces) = rhs.split_once(' ').ok_or(ParseError::SyntaxError)?;
            let quantity: u64 = quantity.parse()?;
            let next_name = names.len();
            let produces = *names.entry(produces).or_insert(Chemical::Other(next_name));
            reactions.push(Reaction {
                requires,
                quantity,
                produces,
            });
        }
        reactions.sort_by_key(|r| r.produces);
        Ok(Self {
            reactions,
            num_chemicals: names.len(),
        })
    }
}

#[aoc_generator(day14)]
fn parse(input: &str) -> Result<ReactionList, ParseError> {
    input.parse()
}

#[aoc(day14, part1)]
fn part_1(list: &ReactionList) -> u64 {
    ore_to_produce_fuel(list, 1)
}

#[aoc(day14, part2)]
fn part_2(list: &ReactionList) -> u64 {
    let target = 1_000_000_000_000_u64;
    let one_fuel = ore_to_produce_fuel(list, 1);
    let mut high = target.div_ceil(one_fuel) * 2;
    let mut low = 1;
    while low < high {
        let mid = (low + high).div_ceil(2);
        let result = ore_to_produce_fuel(list, mid);
        if result > target {
            high = mid - 1;
        } else {
            low = mid;
        }
    }
    low
}

fn ore_to_produce_fuel(list: &ReactionList, num_fuel: u64) -> u64 {
    let mut lookup = vec![None; list.num_chemicals];
    for reaction in &list.reactions {
        lookup[reaction.produces.index()] = Some(reaction);
    }
    let mut leftovers = vec![0; list.num_chemicals];
    let mut pending = VecDeque::<(u64, Chemical)>::new();
    let mut ores = 0;
    pending.push_back((num_fuel, Chemical::Fuel));
    while let Some((qty, chem)) = pending.pop_front() {
        if chem == Chemical::Ore {
            ores += qty;
        } else if let Some(reaction) = lookup[chem.index()] {
            let servings = qty
                .saturating_sub(leftovers[chem.index()])
                .div_ceil(reaction.quantity);
            if servings > 0 {
                for &(qty2, chem2) in &reaction.requires {
                    pending.push_back((servings * qty2, chem2));
                }
                leftovers[chem.index()] += servings * reaction.quantity;
            }
            leftovers[chem.index()] -= qty;
        }
    }
    ores
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    const EXAMPLE1: &str = "\
        10 ORE => 10 A\n\
        1 ORE => 1 B\n\
        7 A, 1 B => 1 C\n\
        7 A, 1 C => 1 D\n\
        7 A, 1 D => 1 E\n\
        7 A, 1 E => 1 FUEL\
    ";

    const EXAMPLE2: &str = "\
        9 ORE => 2 A\n\
        8 ORE => 3 B\n\
        7 ORE => 5 C\n\
        3 A, 4 B => 1 AB\n\
        5 B, 7 C => 1 BC\n\
        4 C, 1 A => 1 CA\n\
        2 AB, 3 BC, 4 CA => 1 FUEL\
    ";

    const EXAMPLE3: &str = "\
        157 ORE => 5 NZVS\n\
        165 ORE => 6 DCFZ\n\
        44 XJWVT, 5 KHKGT, 1 QDVJ, 29 NZVS, 9 GPVTF, 48 HKGWZ => 1 FUEL\n\
        12 HKGWZ, 1 GPVTF, 8 PSHF => 9 QDVJ\n\
        179 ORE => 7 PSHF\n\
        177 ORE => 5 HKGWZ\n\
        7 DCFZ, 7 PSHF => 2 XJWVT\n\
        165 ORE => 2 GPVTF\n\
        3 DCFZ, 7 NZVS, 5 HKGWZ, 10 PSHF => 8 KHKGT\
    ";

    const EXAMPLE4: &str = "\
        2 VPVL, 7 FWMGM, 2 CXFTF, 11 MNCFX => 1 STKFG\n\
        17 NVRVD, 3 JNWZP => 8 VPVL\n\
        53 STKFG, 6 MNCFX, 46 VJHF, 81 HVMC, 68 CXFTF, 25 GNMV => 1 FUEL\n\
        22 VJHF, 37 MNCFX => 5 FWMGM\n\
        139 ORE => 4 NVRVD\n\
        144 ORE => 7 JNWZP\n\
        5 MNCFX, 7 RFSQX, 2 FWMGM, 2 VPVL, 19 CXFTF => 3 HVMC\n\
        5 VJHF, 7 MNCFX, 9 VPVL, 37 CXFTF => 6 GNMV\n\
        145 ORE => 6 MNCFX\n\
        1 NVRVD => 8 CXFTF\n\
        1 VJHF, 6 MNCFX => 4 RFSQX\n\
        176 ORE => 6 VJHF\
    ";

    const EXAMPLE5: &str = "\
        171 ORE => 8 CNZTR\n\
        7 ZLQW, 3 BMBT, 9 XCVML, 26 XMNCP, 1 WPTQ, 2 MZWV, 1 RJRHP => 4 PLWSL\n\
        114 ORE => 4 BHXH\n\
        14 VRPVC => 6 BMBT\n\
        6 BHXH, 18 KTJDG, 12 WPTQ, 7 PLWSL, 31 FHTLT, 37 ZDVW => 1 FUEL\n\
        6 WPTQ, 2 BMBT, 8 ZLQW, 18 KTJDG, 1 XMNCP, 6 MZWV, 1 RJRHP => 6 FHTLT\n\
        15 XDBXC, 2 LTCX, 1 VRPVC => 6 ZLQW\n\
        13 WPTQ, 10 LTCX, 3 RJRHP, 14 XMNCP, 2 MZWV, 1 ZLQW => 1 ZDVW\n\
        5 BMBT => 4 WPTQ\n\
        189 ORE => 9 KTJDG\n\
        1 MZWV, 17 XDBXC, 3 XCVML => 2 XMNCP\n\
        12 VRPVC, 27 CNZTR => 2 XDBXC\n\
        15 KTJDG, 12 BHXH => 5 XCVML\n\
        3 BHXH, 2 VRPVC => 7 MZWV\n\
        121 ORE => 7 VRPVC\n\
        7 XCVML => 6 RJRHP\n\
        5 BHXH, 4 VRPVC => 5 LTCX\
    ";

    #[test]
    fn test_parse() {
        const ORE: Chemical = Chemical::Ore;
        const FUEL: Chemical = Chemical::Fuel;
        const A: Chemical = Chemical::Other(2);
        const B: Chemical = Chemical::Other(3);
        const C: Chemical = Chemical::Other(4);
        const D: Chemical = Chemical::Other(5);
        const E: Chemical = Chemical::Other(6);
        macro_rules! reaction {
            ($($qty1:literal $chm1:ident),* $(,)? => $qty2:literal $chm2:ident) => {
                Reaction {
                    requires: vec![$(($qty1, $chm1)),*],
                    quantity: $qty2,
                    produces: $chm2
                }
            }
        }
        let result = parse(EXAMPLE1).unwrap();
        assert_eq!(
            result.reactions,
            [
                reaction!(10 ORE => 10 A),
                reaction!(1 ORE => 1 B),
                reaction!(7 A, 1 B => 1 C),
                reaction!(7 A, 1 C => 1 D),
                reaction!(7 A, 1 D => 1 E),
                reaction!(7 A, 1 E => 1 FUEL),
            ]
        );
    }

    #[test_case(EXAMPLE1 => 31)]
    #[test_case(EXAMPLE2 => 165)]
    #[test_case(EXAMPLE3 => 13_312)]
    #[test_case(EXAMPLE4 => 180_697)]
    #[test_case(EXAMPLE5 => 2_210_736)]
    fn test_part_1(input: &str) -> u64 {
        let list = parse(input).unwrap();
        part_1(&list)
    }

    #[test_case(EXAMPLE3 => 82_892_753)]
    #[test_case(EXAMPLE4 => 5_586_022)]
    #[test_case(EXAMPLE5 => 460_664)]
    fn test_part_2(input: &str) -> u64 {
        let list = parse(input).unwrap();
        part_2(&list)
    }
}
