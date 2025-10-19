use std::fmt::{Display, Write};
use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Error)]
enum ParseError {
    #[error("Invalid tile")]
    InvalidTile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Bugs(u32);

impl Bugs {
    fn simple_evolve(self) -> Self {
        // ............bottom-right--v.................top-left--v
        let below = (self.0 & 0b00000_11111_11111_11111_11111) << 5;
        let above = (self.0 & 0b11111_11111_11111_11111_00000) >> 5;
        let left_ = (self.0 & 0b11110_11110_11110_11110_11110) >> 1;
        let right = (self.0 & 0b01111_01111_01111_01111_01111) << 1;
        let mut new_mask = 0_u32;
        for ix in 0..25 {
            let bit = 1 << ix;
            let neighbors = u32::from(below & bit != 0)
                + u32::from(above & bit != 0)
                + u32::from(left_ & bit != 0)
                + u32::from(right & bit != 0);
            new_mask |= u32::from(matches!(
                (self.0 & bit != 0, neighbors),
                (false, 1..=2) | (true, 1)
            )) << ix;
        }
        Self(new_mask)
    }

    const fn biodiversity(self) -> u32 {
        self.0
    }

    const fn count_all(self) -> u32 {
        self.0.count_ones()
    }

    const fn count_outer_right(self) -> u32 {
        (self.0 & 0b10000_10000_10000_10000_10000).count_ones()
    }
    const fn count_outer_left(self) -> u32 {
        (self.0 & 0b00001_00001_00001_00001_00001).count_ones()
    }
    const fn count_outer_top(self) -> u32 {
        (self.0 & 0b00000_00000_00000_00000_11111).count_ones()
    }
    const fn count_outer_bottom(self) -> u32 {
        (self.0 & 0b11111_00000_00000_00000_00000).count_ones()
    }
    const fn count_inner_right(self) -> u32 {
        (self.0 & 0b00000_00000_01000_00000_00000).count_ones()
    }
    const fn count_inner_left(self) -> u32 {
        (self.0 & 0b00000_00000_00010_00000_00000).count_ones()
    }
    const fn count_inner_top(self) -> u32 {
        (self.0 & 0b00000_00000_00000_00100_00000).count_ones()
    }
    const fn count_inner_bottom(self) -> u32 {
        (self.0 & 0b00000_00100_00000_00000_00000).count_ones()
    }
    fn layered_evolve(self, inner: Self, outer: Self) -> Self {
        // ............bottom-right--v.................top-left--v
        let below = (self.0 & 0b00000_11111_11011_11111_11111) << 5;
        let above = (self.0 & 0b11111_11111_11011_11111_00000) >> 5;
        let left_ = (self.0 & 0b11110_11110_11010_11110_11110) >> 1;
        let right = (self.0 & 0b01111_01111_01011_01111_01111) << 1;
        let mut new_mask = 0_u32;
        for ix in 0..25 {
            let row = ix / 5;
            let col = ix % 5;
            if (row, col) == (2, 2) {
                continue;
            }
            let bit = 1 << ix;
            let mut neighbors = u32::from(below & bit != 0)
                + u32::from(above & bit != 0)
                + u32::from(left_ & bit != 0)
                + u32::from(right & bit != 0);
            match row {
                0 => neighbors += outer.count_inner_top(),
                1 if col == 2 => neighbors += inner.count_outer_top(),
                3 if col == 2 => neighbors += inner.count_outer_bottom(),
                4 => neighbors += outer.count_inner_bottom(),
                _ => {}
            }
            match col {
                0 => neighbors += outer.count_inner_left(),
                1 if row == 2 => neighbors += inner.count_outer_left(),
                3 if row == 2 => neighbors += inner.count_outer_right(),
                4 => neighbors += outer.count_inner_right(),
                _ => {}
            }
            new_mask |= u32::from(matches!(
                (self.0 & bit != 0, neighbors),
                (false, 1..=2) | (true, 1)
            )) << ix;
        }
        Self(new_mask)
    }
}

impl FromStr for Bugs {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut mask = 0_u32;
        for (r, line) in input.lines().enumerate() {
            for (c, ch) in line.bytes().enumerate() {
                mask |= match ch {
                    b'#' => 1 << (r * 5 + c),
                    b'.' => 0,
                    _ => return Err(ParseError::InvalidTile),
                };
            }
        }
        Ok(Self(mask))
    }
}

impl Display for Bugs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in 0..5 {
            if row > 0 {
                f.write_char('\n')?;
            }
            for col in 0..5 {
                let bit = self.0 & (1 << (5 * row + col)) != 0;
                if (row, col) == (2, 2) && f.alternate() {
                    f.write_char('?')?;
                } else {
                    f.write_char(if bit { '#' } else { '.' })?;
                }
            }
        }
        Ok(())
    }
}

#[aoc_generator(day24)]
fn parse(input: &str) -> Result<Bugs, ParseError> {
    input.parse()
}

#[aoc(day24, part1)]
#[expect(clippy::trivially_copy_pass_by_ref, reason = "aoc lib")]
fn part_1(bugs: &Bugs) -> u32 {
    let first_repeat = find_first_repeat(*bugs, Bugs::simple_evolve);

    first_repeat.biodiversity()
}

fn find_first_repeat<T: Copy + Eq>(start: T, step: impl Fn(T) -> T) -> T {
    let mut power = 1;
    let mut cycle_len = 1;
    let mut slow = start;
    let mut fast = step(start);
    while slow != fast {
        if power == cycle_len {
            slow = fast;
            power *= 2;
            cycle_len = 0;
        }
        fast = step(fast);
        cycle_len += 1;
    }
    slow = start;
    fast = start;
    for _ in 0..cycle_len {
        fast = step(fast);
    }
    // let mut cycle_start = 0;
    while slow != fast {
        slow = step(slow);
        fast = step(fast);
        // cycle_start += 1;
    }
    slow
}

#[aoc(day24, part2)]
#[expect(clippy::trivially_copy_pass_by_ref, reason = "aoc lib")]
fn part_2(&bugs: &Bugs) -> u32 {
    layered_evolution(bugs, 200).count_all()
}

fn layered_evolution(bugs: Bugs, cycles: usize) -> BugStack {
    let mut stack = BugStack::new(bugs);
    for _ in 0..cycles {
        stack.evolve_layers();
    }
    stack
}

#[derive(Debug, Clone)]
struct BugStack {
    layers: Vec<Bugs>,
    numbering_offset: i32,
}

impl BugStack {
    fn new(initial: Bugs) -> Self {
        Self {
            layers: [initial].into(),
            numbering_offset: 0,
        }
    }

    fn evolve_layers(&mut self) {
        let mut outer = Bugs(0);
        let mut middle = Bugs(0);
        for inner in &mut self.layers {
            let evolved = middle.layered_evolve(*inner, outer);
            outer = middle;
            middle = *inner;
            *inner = evolved;
        }
        self.layers.push(middle.layered_evolve(Bugs(0), outer));
        self.layers.push(Bugs(0).layered_evolve(Bugs(0), middle));
        self.numbering_offset -= 1;
        if self.layers.last().unwrap().count_all() == 0 {
            self.layers.pop();
        }
        if self.layers.first().unwrap().count_all() == 0 {
            self.layers.remove(0);
            self.numbering_offset += 1;
        }
    }

    fn count_all(&self) -> u32 {
        self.layers.iter().copied().map(Bugs::count_all).sum()
    }
}

impl Display for BugStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Display each layer horizontally, to save screen space
        let all = self
            .layers
            .iter()
            .map(|bugs| format!("{bugs:#}"))
            .collect::<Vec<_>>();
        for ix in 0..all.len() {
            let depth = i32::try_from(ix).unwrap() + self.numbering_offset;
            write!(f, "Depth {depth:<2}  ",)?;
        }
        writeln!(f)?;
        for line in 0..5 {
            for bug_str in &all {
                let line = bug_str.lines().nth(line).unwrap();
                write!(f, "{line:<5}     ")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = "\
        ....#\n\
        #..#.\n\
        #..##\n\
        ..#..\n\
        #....\
    ";

    #[test]
    fn test_simple_evolve() {
        let mut bugs = parse(EXAMPLE).unwrap();
        for _ in 0..4 {
            bugs = bugs.simple_evolve();
        }
        let expected = "\
            ####.\n\
            ....#\n\
            ##..#\n\
            .....\n\
            ##...\
        ";
        assert_eq!(bugs.to_string(), expected);
    }

    #[test]
    fn test_part_1() {
        let bugs = parse(EXAMPLE).unwrap();
        let result = part_1(&bugs);
        assert_eq!(result, 2_129_920);
    }

    #[test]
    fn test_layered_evolution() {
        let bugs = parse(EXAMPLE).unwrap();
        let result = layered_evolution(bugs, 10);
        println!("{result}");
        assert_eq!(result.count_all(), 99);
    }
}
