use std::collections::HashSet;
use std::fmt::Display;
use std::num::ParseIntError;
use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};
use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Error)]
enum ParseError {
    #[error("Syntax error")]
    SyntaxError,
    #[error(transparent)]
    InvalidNumber(#[from] ParseIntError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct Vector {
    x: i64,
    y: i64,
    z: i64,
}

impl Vector {
    const fn normalized(mut self) -> Self {
        self.x = self.x.signum();
        self.y = self.y.signum();
        self.z = self.z.signum();
        self
    }

    const fn size(self) -> u64 {
        self.x.unsigned_abs() + self.y.unsigned_abs() + self.z.unsigned_abs()
    }
}

impl AddAssign for Vector {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Add for Vector {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl SubAssign for Vector {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl Sub for Vector {
    type Output = Self;

    fn sub(mut self, rhs: Self) -> Self::Output {
        self -= rhs;
        self
    }
}

impl MulAssign<i64> for Vector {
    fn mul_assign(&mut self, rhs: i64) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl Mul<i64> for Vector {
    type Output = Self;

    fn mul(mut self, rhs: i64) -> Self::Output {
        self *= rhs;
        self
    }
}

impl FromStr for Vector {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (x, rest) = s
            .strip_prefix("<x=")
            .ok_or(ParseError::SyntaxError)?
            .split_once(", y=")
            .ok_or(ParseError::SyntaxError)?;
        let x = x.parse()?;
        let (y, rest) = rest.split_once(", z=").ok_or(ParseError::SyntaxError)?;
        let y = y.parse()?;
        let z = rest
            .strip_suffix(">")
            .ok_or(ParseError::SyntaxError)?
            .parse()?;
        Ok(Self { x, y, z })
    }
}

impl Display for Vector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let &Self { x, y, z } = self;
        write!(f, "<x={x:2}, y={y:2}, z={z:2}>")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Moon {
    position: Vector,
    velocity: Vector,
}

impl Moon {
    fn new(position: Vector) -> Self {
        Self {
            position,
            velocity: Vector::default(),
        }
    }

    fn apply_gravity(&mut self, other: &Self) {
        self.velocity += (other.position - self.position).normalized();
    }

    fn apply_velocity(&mut self) {
        self.position += self.velocity;
    }

    const fn energy(&self) -> u64 {
        self.position.size() * self.velocity.size()
    }
}

impl Display for Moon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { position, velocity } = self;
        write!(f, "pos={position}, vel={velocity}")
    }
}

#[derive(Debug, Clone)]
struct Simulation<const N: usize> {
    moons: [Moon; N],
    time: u64,
}

impl<const N: usize> Simulation<N> {
    fn new(moons: &[Moon]) -> Self {
        Self {
            moons: moons.to_vec().try_into().unwrap(),
            time: 0,
        }
    }

    fn apply_gravity(&mut self) {
        for i in 0..N {
            let mut moon1 = self.moons[i]; // Copy
            for (j, moon2) in self.moons.iter().enumerate() {
                if i == j {
                    continue;
                }
                moon1.apply_gravity(moon2);
            }
            self.moons[i] = moon1; // Put back
        }
    }

    fn apply_velocity(&mut self) {
        for moon in &mut self.moons {
            moon.apply_velocity();
        }
    }

    fn time_step(&mut self) {
        self.apply_gravity();
        self.apply_velocity();
        self.time += 1;
    }

    fn total_energy(&self) -> u64 {
        self.moons.iter().map(Moon::energy).sum()
    }
}

impl<const N: usize> Display for Simulation<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { time, moons } = self;
        writeln!(f, "After {time} steps:")?;
        for moon in moons {
            writeln!(f, "{moon}")?;
        }
        Ok(())
    }
}

#[aoc_generator(day12)]
fn parse(input: &str) -> Result<Vec<Moon>, ParseError> {
    input
        .lines()
        .map(|l| str::parse(l).map(Moon::new))
        .collect()
}

#[aoc(day12, part1)]
fn part_1(moons: &[Moon]) -> u64 {
    total_energy_after(moons, 1000)
}

fn total_energy_after(moons: &[Moon], time: u64) -> u64 {
    let mut sim = Simulation::<4>::new(moons);
    for _ in 0..time {
        sim.time_step();
    }
    sim.total_energy()
}

#[aoc(day12, part2)]
fn part_2(moons: &[Moon]) -> u64 {
    let cycle_x = find_time_until_repeat_slice(moons, |v| v.x);
    let cycle_y = find_time_until_repeat_slice(moons, |v| v.y);
    let cycle_z = find_time_until_repeat_slice(moons, |v| v.z);
    lcm(lcm(cycle_x, cycle_y), cycle_z)
}

fn find_time_until_repeat_slice(moons: &[Moon], view: impl Fn(Vector) -> i64) -> u64 {
    let mut sim = Simulation::<4>::new(moons);
    let mut seen = HashSet::new();
    while seen.insert(sim.moons.map(|m| (view(m.position), view(m.velocity)))) {
        sim.time_step();
    }
    sim.time
}

const fn lcm(u: u64, v: u64) -> u64 {
    let g = gcd(u, v);
    u / g * v
}

const fn gcd(mut u: u64, mut v: u64) -> u64 {
    if u == 0 {
        return v;
    }
    if v == 0 {
        return u;
    }

    let shift = (u | v).trailing_zeros();
    u >>= shift;
    v >>= shift;
    u >>= u.trailing_zeros();

    loop {
        v >>= v.trailing_zeros();

        if u > v {
            (u, v) = (v, u);
        }

        v -= u; // here v >= u

        if v == 0 {
            break;
        }
    }

    u << shift
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    const EXAMPLE1: &str = "\
        <x=-1, y=0, z=2>\n\
        <x=2, y=-10, z=-7>\n\
        <x=4, y=-8, z=8>\n\
        <x=3, y=5, z=-1>\
    ";

    const EXAMPLE2: &str = "\
        <x=-8, y=-10, z=0>\n\
        <x=5, y=5, z=10>\n\
        <x=2, y=-7, z=3>\n\
        <x=9, y=-8, z=-3>\
    ";

    macro_rules! moon {
        ($x:expr, $y:expr, $z:expr) => {
            Moon::new(Vector {
                x: $x,
                y: $y,
                z: $z,
            })
        };
    }

    #[test]
    fn test_parse() {
        let result = parse(EXAMPLE1).unwrap();
        assert_eq!(
            result,
            [
                moon!(-1, 0, 2),
                moon!(2, -10, -7),
                moon!(4, -8, 8),
                moon!(3, 5, -1),
            ]
        );
    }

    #[test_case(EXAMPLE1, 10 => 179)]
    #[test_case(EXAMPLE2, 100 => 1940)]
    fn test_part_1(input: &str, time: u64) -> u64 {
        let moons = parse(input).unwrap();
        total_energy_after(&moons, time)
    }

    #[test_case(EXAMPLE1 => 2_772)]
    #[test_case(EXAMPLE2 => 4_686_774_924)]
    fn test_part_2(input: &str) -> u64 {
        let moons = parse(input).unwrap();
        part_2(&moons)
    }
}
