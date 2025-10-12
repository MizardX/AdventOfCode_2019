use std::collections::{HashMap, VecDeque};
use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Error)]
enum ParseError {
    #[error("Syntax error")]
    SyntaxError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Object {
    Unknown,
    Com,
    You,
    San,
    Other(usize),
}

impl Object {
    const fn index(self) -> usize {
        match self {
            Self::Unknown => panic!("Unkown object"),
            Self::Com => 0,
            Self::You => 1,
            Self::San => 2,
            Self::Other(ix) => ix,
        }
    }
}

#[derive(Debug, Clone)]
struct Map {
    direct_orbits: Vec<Object>, // [Com=0] = Com
}

impl FromStr for Map {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut names = HashMap::new();
        names.insert("COM", Object::Com);
        names.insert("YOU", Object::You);
        names.insert("SAN", Object::San);
        for line in s.lines() {
            let (lhs, rhs) = line.split_once(')').ok_or(ParseError::SyntaxError)?;
            let next_ix = names.len();
            names.entry(lhs).or_insert(Object::Other(next_ix));
            let next_ix = names.len();
            names.entry(rhs).or_insert(Object::Other(next_ix));
        }
        let mut direct_orbits = vec![Object::Unknown; names.len()];
        direct_orbits[Object::Com.index()] = Object::Com;
        for line in s.lines() {
            let (lhs, rhs) = line.split_once(')').ok_or(ParseError::SyntaxError)?;
            let left = *names.get(lhs).unwrap();
            let right = *names.get(rhs).unwrap();
            direct_orbits[right.index()] = left;
        }
        Ok(Self { direct_orbits })
    }
}

#[aoc_generator(day6)]
fn parse(input: &str) -> Result<Map, ParseError> {
    input.parse()
}

#[aoc(day6, part1)]
fn part_1(map: &Map) -> usize {
    let n = map.direct_orbits.len();
    let mut waiting_for = vec![vec![]; n];
    let mut orbits = vec![None::<usize>; n];
    orbits[Object::Com.index()] = Some(0);
    let mut pending = (0..n).collect::<VecDeque<usize>>();
    while let Some(ix) = pending.pop_front() {
        if map.direct_orbits[ix] == Object::Unknown {
            continue;
        } // skip
        if orbits[ix].is_some() {
            continue;
        }
        let parent = map.direct_orbits[ix];
        if let Some(parent_orbits) = orbits[parent.index()] {
            orbits[ix] = Some(parent_orbits + 1);
            pending.extend(&waiting_for[ix]);
            waiting_for[ix].clear();
        } else {
            waiting_for[parent.index()].push(ix);
        }
    }
    orbits.into_iter().flatten().sum()
}

#[aoc(day6, part2)]
fn part_2(map: &Map) -> usize {
    let mut you_depth = 0;
    let mut you_node = Object::You;
    while you_node != Object::Com {
        you_node = map.direct_orbits[you_node.index()];
        you_depth += 1;
    }
    let mut san_depth = 0;
    let mut san_node = Object::San;
    while san_node != Object::Com {
        san_node = map.direct_orbits[san_node.index()];
        san_depth += 1;
    }
    you_node = Object::You;
    san_node = Object::San;
    if san_depth < you_depth {
        for _ in san_depth..you_depth {
            you_node = map.direct_orbits[you_node.index()];
        }
    } else if san_depth > you_depth {
        for _ in you_depth..san_depth {
            san_node = map.direct_orbits[san_node.index()];
        }
    }
    let mut common_depth = san_depth.min(you_depth);
    while you_node != san_node {
        you_node = map.direct_orbits[you_node.index()];
        san_node = map.direct_orbits[san_node.index()];
        common_depth -= 1;
    }
    you_depth + san_depth - common_depth * 2 - 2
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE1: &str = "\
        COM)B\n\
        B)C\n\
        C)D\n\
        D)E\n\
        E)F\n\
        B)G\n\
        G)H\n\
        D)I\n\
        E)J\n\
        J)K\n\
        K)L\
    ";

    const EXAMPLE2: &str = "\
        COM)B\n\
        B)C\n\
        C)D\n\
        D)E\n\
        E)F\n\
        B)G\n\
        G)H\n\
        D)I\n\
        E)J\n\
        J)K\n\
        K)L\n\
        K)YOU\n\
        I)SAN\
    ";

    #[test]
    fn test_parse() {
        let result = parse(EXAMPLE1).unwrap();
        assert_eq!(
            result.direct_orbits,
            [
                Object::Com,       // COM to itself
                Object::Unknown,   // YOU not present
                Object::Unknown,   // SAN not present
                Object::Com,       // COM)B
                Object::Other(3),  // B)C
                Object::Other(4),  // C)D
                Object::Other(5),  // D)E
                Object::Other(6),  // E)F
                Object::Other(3),  // B)G
                Object::Other(8),  // G)H
                Object::Other(5),  // D)I
                Object::Other(6),  // E)J
                Object::Other(11), // J)K
                Object::Other(12)  // K)L
            ]
        );
    }

    #[test]
    fn test_part_1() {
        let map = parse(EXAMPLE1).unwrap();
        let result = part_1(&map);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_part_2() {
        let map = parse(EXAMPLE2).unwrap();
        let result = part_2(&map);
        assert_eq!(result, 4);
    }
}
