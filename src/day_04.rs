use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Error)]
enum ParseError {
    #[error("Syntax error")]
    SyntaxError,
}

type Password = [u8; 6];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PasswordRange {
    lower: Password,
    upper: Password,
}

impl FromStr for PasswordRange {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = s.as_bytes();
        if bytes.len() != 13 || bytes[6] != b'-' {
            return Err(ParseError::SyntaxError);
        }
        Ok(Self {
            lower: s.as_bytes()[..6].try_into().unwrap(),
            upper: s.as_bytes()[7..].try_into().unwrap(),
        })
    }
}

#[aoc_generator(day4)]
fn parse(input: &str) -> Result<PasswordRange, ParseError> {
    input.parse()
}

#[aoc(day4, part1)]
fn part_1(range: &PasswordRange) -> usize {
    PasswordEnumerator::new(range)
        .filter(is_valid_part_1)
        .count()
}

#[expect(clippy::trivially_copy_pass_by_ref, reason = "filter")]
fn is_valid_part_1(password: &Password) -> bool {
    let mut counts = [0_u8; 10];
    let mut prev = 0;
    for &ch in password {
        if ch < prev { return false; }
        let ix = (ch - b'0') as usize;
        counts[ix] += 1;
        prev = ch;
    }
    counts.into_iter().any(|c| c >= 2)
}

#[aoc(day4, part2)]
fn part_2(range: &PasswordRange) -> usize {
    PasswordEnumerator::new(range)
        .filter(is_valid_part_2)
        .count()
}

#[expect(clippy::trivially_copy_pass_by_ref, reason = "filter")]
fn is_valid_part_2(password: &Password) -> bool {
    let mut counts = [0_u8; 10];
    let mut prev = 0_u8;
    for &ch in password {
        if ch < prev { return false; }
        let ix = (ch - b'0') as usize;
        counts[ix] += 1;
        prev = ch;
    }
    counts.into_iter().any(|c| c == 2)
}

#[derive(Debug, Clone)]
struct PasswordEnumerator<'a> {
    range: &'a PasswordRange,
    next: Password,
}

impl<'a> PasswordEnumerator<'a> {
    fn new(range: &'a PasswordRange) -> Self {
        let mut next = range.lower;
        // Start at first increasing sequence
        let mut max = b'0';
        for ch in &mut next {
            max = max.max(*ch);
            *ch = max;
        }
        Self { range, next }
    }
}

impl Iterator for PasswordEnumerator<'_> {
    type Item = Password;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next > self.range.upper {
            return None;
        }
        let res = self.next;
        for (ix, ch) in self.next.iter_mut().enumerate().rev() {
            if *ch == b'9' {
                *ch = b'0';
            } else {
                *ch += 1;
                // Skip to next increasing sequence
                let digit = *ch;
                for ch2 in &mut self.next[ix + 1..] {
                    *ch2 = digit;
                }
                break;
            }
        }
        Some(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(b"111111" => true)]
    #[test_case(b"223450" => false)]
    #[test_case(b"123789" => false)]
    #[allow(clippy::trivially_copy_pass_by_ref, reason = "byte literals")]
    fn test_valid_part_1(password: &Password) -> bool {
        is_valid_part_1(password)
    }

    #[test_case(b"112233" => true)]
    #[test_case(b"123444" => false)]
    #[test_case(b"111122" => true)]
    #[allow(clippy::trivially_copy_pass_by_ref, reason = "byte literals")]
    fn test_valid_part_2(password: &Password) -> bool {
        is_valid_part_2(password)
    }
}
