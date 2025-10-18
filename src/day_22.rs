use std::fmt::Display;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Operation {
    /// Reverse; card k -> position (10.006 - k)
    DealIntoNewDeck,
    /// Rotate; card k -> position (n + k) % 10.007
    Cut(i64),
    /// card k -> position (n * k) % 10.007
    DealWithIncrement(u64),
}

impl Operation {
    fn apply(self, deck: Shuffle) -> Shuffle {
        match self {
            Self::DealIntoNewDeck => {
                let last = deck.card_at_position(deck.size - 1);
                let second_last = deck.card_at_position(deck.size - 2);
                let step = (second_last + deck.size - last) % deck.size;
                Shuffle::new(last, step, deck.size)
            }
            Self::Cut(dist) => {
                let first =
                    deck.card_at_position(deck.size.checked_add_signed(dist).unwrap() % deck.size);
                Shuffle::new(first, deck.step, deck.size)
            }
            Self::DealWithIncrement(scale) => {
                let step = modular_mul(deck.step, modular_inverse(scale, deck.size), deck.size);
                Shuffle::new(deck.first, step, deck.size)
            }
        }
    }
}

impl FromStr for Operation {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if s == "deal into new stack" {
            Self::DealIntoNewDeck
        } else if let Some(num) = s.strip_prefix("deal with increment ") {
            Self::DealWithIncrement(num.parse()?)
        } else if let Some(num) = s.strip_prefix("cut ") {
            Self::Cut(num.parse()?)
        } else {
            return Err(ParseError::SyntaxError);
        })
    }
}

#[aoc_generator(day22)]
fn parse(input: &str) -> Result<Vec<Operation>, ParseError> {
    input.lines().map(str::parse).collect()
}

#[aoc(day22, part1)]
fn part_1(operations: &[Operation]) -> u64 {
    position_of_card(operations, 2019, 10_007)
}

fn position_of_card(operations: &[Operation], card: u64, deck_size: u64) -> u64 {
    let mut poly = Shuffle::new(0, 1, deck_size);
    for op in operations {
        poly = op.apply(poly);
    }
    poly.position_of_card(card)
}

#[aoc(day22, part2)]
fn part_2(operations: &[Operation]) -> u64 {
    repeated_card_at_position(operations, 2020, 119_315_717_514_047, 101_741_582_076_661)
}

fn repeated_card_at_position(
    operations: &[Operation],
    target_position: u64,
    deck_size: u64,
    shuffles: u64,
) -> u64 {
    let mut shuffle = Shuffle::new(0, 1, deck_size);
    for op in operations {
        shuffle = op.apply(shuffle);
    }

    let shuffle_iterated = shuffle.iterated(shuffles);
    shuffle_iterated.card_at_position(target_position)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Shuffle {
    first: u64,
    step: u64,
    size: u64,
}

impl Shuffle {
    const fn new(first: u64, step: u64, size: u64) -> Self {
        Self { first, step, size }
    }

    fn card_at_position(self, position: u64) -> u64 {
        (modular_mul(position, self.step, self.size) + self.first) % self.size
    }

    fn position_of_card(self, card: u64) -> u64 {
        // card = first + step * pos
        // pos * step == card - first
        // pos == (card - first) * step^-1
        modular_mul(
            (card + self.size - self.first) % self.size,
            modular_inverse(self.step, self.size),
            self.size,
        )
    }

    fn iterated(self, times: u64) -> Self {
        // f(x) = (a * x + b) % m
        // f(f(x)) = (a^2 * x + (a + 1) * b) % m
        // f(f(f(x))) = (a^3 * x + (a^2 + a + 1) * b) % m

        // sum(a^k,k=0..n-1) = (a^n - 1)/(a - 1)

        // (f^n)(x) = (a^n * x + (a^n - 1)/(a - 1) * b) % m
        let Self { step, first, size } = self;
        let step2 = modular_pow(step, times, size);
        let first2_scale = modular_mul(step2 - 1, modular_inverse(step - 1, size), size);
        let first2 = modular_mul(first2_scale, first, size);
        Self::new(first2, step2, size)
    }
}

impl Display for Shuffle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries((0..self.size).map(|pos| self.card_at_position(pos)))
            .finish()
    }
}

fn modular_mul(mut a: u64, mut b: u64, modulo: u64) -> u64 {
    a %= modulo;
    b %= modulo;
    a.checked_mul(b).map_or_else(
        || u64::try_from(u128::from(a) * u128::from(b) % u128::from(modulo)).unwrap(),
        |prod| prod % modulo,
    )
}

fn modular_pow(a: u64, n: u64, m: u64) -> u64 {
    match n {
        0 => 1,
        1 => n % m,
        _ => {
            let mut res = 1;
            let mut base = a;
            let mut pow = n;
            while pow > 0 {
                if pow & 1 == 0 {
                    base = modular_mul(base, base, m);
                    pow /= 2;
                } else {
                    res = modular_mul(res, base, m);
                    pow -= 1;
                }
            }
            res
        }
    }
}

fn modular_inverse(a: u64, m: u64) -> u64 {
    let (_, x, _) = egcd(a, m);
    if x < 0 {
        m.checked_add_signed(x).unwrap()
    } else {
        (0_u64).checked_add_signed(x).unwrap()
    }
}

pub fn egcd(a: u64, b: u64) -> (u64, i64, i64) {
    let (mut r0, mut r1) = (a, b);
    let (mut s0, mut s1) = (1, 0);
    let (mut t0, mut t1) = (0, 1);
    while r1 != 0 {
        let q = r0 / r1;
        r0 -= q * r1;
        s0 -= i64::try_from(q).unwrap() * s1;
        t0 -= i64::try_from(q).unwrap() * t1;
        (r0, r1) = (r1, r0);
        (s0, s1) = (s1, s0);
        (t0, t1) = (t1, t0);
    }
    (r0, s0, t0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    const EXAMPLE1: &str = "\
        deal with increment 7\n\
        deal into new stack\n\
        deal into new stack\
    ";

    const EXAMPLE2: &str = "\
        cut 6\n\
        deal with increment 7\n\
        deal into new stack\
    ";

    const EXAMPLE3: &str = "\
        deal with increment 7\n\
        deal with increment 9\n\
        cut -2\
    ";

    const EXAMPLE4: &str = "\
        deal into new stack\n\
        cut -2\n\
        deal with increment 7\n\
        cut 8\n\
        cut -4\n\
        deal with increment 7\n\
        cut 3\n\
        deal with increment 9\n\
        deal with increment 3\n\
        cut -1\
    ";

    #[test]
    fn test_parse() {
        let result = parse(EXAMPLE2).unwrap();
        assert_eq!(
            result,
            [
                Operation::Cut(6),
                Operation::DealWithIncrement(7),
                Operation::DealIntoNewDeck,
            ]
        );
    }

    #[test_case(Operation::DealIntoNewDeck => &[9, 8, 7, 6, 5, 4, 3, 2, 1, 0][..])]
    #[test_case(Operation::Cut(3) => &[3, 4, 5, 6, 7, 8, 9, 0, 1, 2][..])]
    #[test_case(Operation::Cut(-4) => &[6, 7, 8, 9, 0, 1, 2, 3, 4, 5][..])]
    #[test_case(Operation::DealWithIncrement(3) => &[0, 7, 4, 1, 8, 5, 2, 9, 6, 3][..])]
    fn test_single(op: Operation) -> Vec<u64> {
        let shuffle = op.apply(Shuffle::new(0, 1, 10));
        (0..10).map(|card| shuffle.card_at_position(card)).collect()
    }

    #[test_case(EXAMPLE1, 10 => &[0, 3, 6, 9, 2, 5, 8, 1, 4, 7][..])]
    #[test_case(EXAMPLE2, 10 => &[3, 0, 7, 4, 1, 8, 5, 2, 9, 6][..])]
    #[test_case(EXAMPLE3, 10 => &[6, 3, 0, 7, 4, 1, 8, 5, 2, 9][..])]
    #[test_case(EXAMPLE4, 10 => &[9, 2, 5, 8, 1, 4, 7, 0, 3, 6][..])]
    fn test_poly_evaluate(input: &str, deck_size: u64) -> Vec<u64> {
        let operations = parse(input).unwrap();
        let mut shuffle = Shuffle::new(0, 1, 10);
        for op in &operations {
            shuffle = op.apply(shuffle);
        }
        (0..deck_size)
            .map(|card| shuffle.card_at_position(card))
            .collect()
    }

    #[test_case(EXAMPLE1, 10 => &[0, 3, 6, 9, 2, 5, 8, 1, 4, 7][..])]
    #[test_case(EXAMPLE2, 10 => &[3, 0, 7, 4, 1, 8, 5, 2, 9, 6][..])]
    #[test_case(EXAMPLE3, 10 => &[6, 3, 0, 7, 4, 1, 8, 5, 2, 9][..])]
    #[test_case(EXAMPLE4, 10 => &[9, 2, 5, 8, 1, 4, 7, 0, 3, 6][..])]
    fn test_part_1(input: &str, deck_size: u64) -> Vec<u64> {
        let operations = parse(input).unwrap();
        let mut new_deck = vec![0; usize::try_from(deck_size).unwrap()];
        for card in 0..deck_size {
            let pos = position_of_card(&operations, card, deck_size);
            new_deck[usize::try_from(pos).unwrap()] = card;
        }
        new_deck
    }

    #[test_case(221, 431)]
    #[test_case(968, 1367)]
    #[test_case(296, 3413)]
    #[test_case(4782, 5039)]
    #[test_case(3619, 5821)]
    #[test_case(1926, 6343)]
    #[test_case(4294, 7001)]
    #[test_case(6240, 7901)]
    #[test_case(460, 8311)]
    #[test_case(7212, 8831)]
    #[test_case(3, 10)]
    fn test_modular_inverse(num: u64, modulo: u64) {
        let inv = modular_inverse(num, modulo);
        assert_eq!((num * inv) % modulo, 1);
    }

    #[test]
    fn test_poly_inv() {
        let poly = Shuffle::new(74, 41, 431);
        let original = (0..poly.size).collect::<Vec<_>>();
        let evaluated = original
            .iter()
            .map(|&x| poly.card_at_position(x))
            .collect::<Vec<_>>();
        let inverted = evaluated
            .iter()
            .map(|&x| poly.position_of_card(x))
            .collect::<Vec<_>>();
        assert_eq!(inverted, original);
    }

    #[test]
    fn test_poly_iterated() {
        let poly = Shuffle::new(1367, 4782, 5039);
        let poly10 = poly.iterated(10);
        let f10_xs = (0..poly.size)
            .map(|x| poly10.card_at_position(x))
            .collect::<Vec<_>>();
        let f_xs_10 = (0..poly.size)
            .map(|x| (0..10).fold(x, |y, _| poly.card_at_position(y)))
            .collect::<Vec<_>>();
        assert_eq!(f10_xs, f_xs_10);
    }

    #[test_case(EXAMPLE1, 11, 10)]
    #[test_case(EXAMPLE2, 11, 10)]
    #[test_case(EXAMPLE3, 11, 10)]
    #[test_case(EXAMPLE4, 11, 10)]
    fn test_part_2(input: &str, deck_size: u64, shuffles: u64) {
        let operations = parse(input).unwrap();
        let original = (0..deck_size).collect::<Vec<_>>();
        let cards = original
            .iter()
            .map(|&pos| repeated_card_at_position(&operations, pos, deck_size, shuffles))
            .collect::<Vec<_>>();
        let positions = cards
            .iter()
            .map(|&card| {
                (0..shuffles).fold(card, |card, _| {
                    position_of_card(&operations, card, deck_size)
                })
            })
            .collect::<Vec<_>>();
        assert_eq!(positions, original);
    }
}
