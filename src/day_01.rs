use std::num::ParseIntError;

#[aoc_generator(day1)]
fn parse(input: &str) -> Result<Vec<u32>, ParseIntError> {
    input.lines().map(str::parse).collect()
}

#[aoc(day1, part1)]
fn part_1(masses: &[u32]) -> u32 {
    masses.iter().map(|&m| (m / 3).saturating_sub(2)).sum()
}

#[aoc(day1, part2)]
fn part_2(masses: &[u32]) -> u32 {
    masses
        .iter()
        .map(|&m| {
            let mut fuel = (m / 3).saturating_sub(2);
            let mut remining_mass = fuel;
            while remining_mass > 0 {
                remining_mass = (remining_mass / 3).saturating_sub(2);
                fuel += remining_mass;
            }
            fuel
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(&[12] => 2)]
    #[test_case(&[14] => 2)]
    #[test_case(&[1969] => 654)]
    #[test_case(&[100_756] => 33_583)]
    #[test_case(&[12, 14, 1969, 100_756] => 34_241)]
    fn test_part_1(messes: &[u32]) -> u32 {
        part_1(messes)
    }
    #[test_case(&[14] => 2)]
    #[test_case(&[1969] => 966)]
    #[test_case(&[100_756] => 50346)]
    #[test_case(&[14, 1969, 100_756] => 51_314)]
    fn test_part_2(messes: &[u32]) -> u32 {
        part_2(messes)
    }
}
