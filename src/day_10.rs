use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use thiserror::Error;

#[derive(Debug, Error)]
enum TileError {
    #[error("Invalid tile")]
    InvalidTile,
}

#[derive(Debug, Clone)]
struct Map {
    asteroid_vec: Vec<(i32, i32)>,
}

#[aoc_generator(day10)]
fn parse(input: &str) -> Result<Map, TileError> {
    let mut asteroid_vec = Vec::new();
    for (y, line) in input.lines().enumerate() {
        for (x, ch) in line.bytes().enumerate() {
            match ch {
                b'#' => {
                    let pt = (i32::try_from(x).unwrap(), i32::try_from(y).unwrap());
                    asteroid_vec.push(pt);
                }
                b'.' => {}
                _ => return Err(TileError::InvalidTile),
            }
        }
    }
    Ok(Map { asteroid_vec })
}

#[aoc(day10, part1)]
fn part_1(map: &Map) -> usize {
    find_base_asteroid(map).0
}

fn find_base_asteroid(map: &Map) -> (usize, (i32, i32)) {
    let mut max_visible = 0;
    let mut best_position = (0, 0);
    let mut lines = HashSet::new();
    for (i, &(x1, y1)) in map.asteroid_vec.iter().enumerate() {
        lines.clear();
        for (j, &(x2, y2)) in map.asteroid_vec.iter().enumerate() {
            if j == i {
                continue;
            }
            let mut dx = x2 - x1;
            let mut dy = y2 - y1;
            let scale = gcd(dx, dy);
            dx /= scale;
            dy /= scale;
            lines.insert((dx, dy));
        }
        let visible = lines.len();
        if visible > max_visible {
            max_visible = visible;
            best_position = (x1, y1);
        }
    }
    (max_visible, best_position)
}

#[aoc(day10, part2)]
fn part_2(map: &Map) -> i32 {
    let base_position = find_base_asteroid(map).1;
    let (x, y) = find_nth_destroyed_asteroid(map, base_position, 200);
    100 * x + y
}

fn find_nth_destroyed_asteroid(map: &Map, (x0, y0): (i32, i32), nth: usize) -> (i32, i32) {
    let mut lines = HashMap::<_, Vec<_>>::new();
    for &(x1, y1) in &map.asteroid_vec {
        let mut dx = x1 - x0;
        let mut dy = y1 - y0;
        if (dx, dy) == (0, 0) {
            continue;
        }
        let scale = gcd(dx, dy);
        dx /= scale;
        dy /= scale;
        lines.entry((dx, dy)).or_default().push((x1, y1));
    }
    let mut all = lines
        .iter_mut()
        .flat_map(|(&(dx, dy), angle_group)| {
            let angle = pseduo_angle(dx, dy);
            angle_group.sort_unstable_by_key(|&(x1, y1)| {
                (x1 - x0).unsigned_abs() + (y1 - x0).unsigned_abs()
            });
            // Index within the group is the turn it will get eliminated
            angle_group
                .iter()
                .enumerate()
                .map(move |(turn, &asteroid)| ((turn, angle), asteroid))
        })
        .collect::<Vec<_>>();
    // f64 is not Ord, so have to use PartialOrd
    all.select_nth_unstable_by(nth - 1, partial_cmp_first).1.1
}

fn partial_cmp_first<K: PartialOrd, V>((x, _): &(K, V), (y, _): &(K, V)) -> Ordering {
    x.partial_cmp(y).unwrap()
}

/// Same ordering as `f64::atan2(-f64::from(dx), f64::from(dy)) + std::f64::consts::PI`
/// X-axis going right, and Y-axis going down. Negative Y-axis is zero, and increasing clockwise.
fn pseduo_angle(dx: i32, dy: i32) -> f64 {
    if dx >= 0 {
        1.0 + f64::from(dy) / f64::from(dx.abs() + dy.abs())
    } else {
        3.0 - f64::from(dy) / f64::from(dx.abs() + dy.abs())
    }
}

const fn gcd(mut u: i32, mut v: i32) -> i32 {
    u = u.abs();
    v = v.abs();
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
        .#..#\n\
        .....\n\
        #####\n\
        ....#\n\
        ...##\
    ";

    const EXAMPLE2: &str = "\
        ......#.#.\n\
        #..#.#....\n\
        ..#######.\n\
        .#.#.###..\n\
        .#..#.....\n\
        ..#....#.#\n\
        #..#....#.\n\
        .##.#..###\n\
        ##...#..#.\n\
        .#....####\
    ";

    const EXAMPLE3: &str = "\
        #.#...#.#.\n\
        .###....#.\n\
        .#....#...\n\
        ##.#.#.#.#\n\
        ....#.#.#.\n\
        .##..###.#\n\
        ..#...##..\n\
        ..##....##\n\
        ......#...\n\
        .####.###.\
    ";

    const EXAMPLE4: &str = "\
        .#..#..###\n\
        ####.###.#\n\
        ....###.#.\n\
        ..###.##.#\n\
        ##.##.#.#.\n\
        ....###..#\n\
        ..#.#..#.#\n\
        #..#.#.###\n\
        .##...##.#\n\
        .....#.#..\
    ";

    const EXAMPLE5: &str = "\
        .#..##.###...#######\n\
        ##.############..##.\n\
        .#.######.########.#\n\
        .###.#######.####.#.\n\
        #####.##.#.##.###.##\n\
        ..#####..#.#########\n\
        ####################\n\
        #.####....###.#.#.##\n\
        ##.#################\n\
        #####.##.###..####..\n\
        ..######..##.#######\n\
        ####.##.####...##..#\n\
        .#####..#.######.###\n\
        ##...#.##########...\n\
        #.##########.#######\n\
        .####.#.###.###.#.##\n\
        ....##.##.###..#####\n\
        .#.#.###########.###\n\
        #.#.#.#####.####.###\n\
        ###.##.####.##.#..##\
    ";

    const EXAMPLE6: &str = "\
        .#....#####...#..\n\
        ##...##.#####..##\n\
        ##...#...#.#####.\n\
        ..#.....#...###..\n\
        ..#.#.....#....##\
    ";

    #[test]
    fn test_parse() {
        let map = parse(EXAMPLE1).unwrap();
        assert_eq!(
            map.asteroid_vec,
            [
                (1, 0),
                (4, 0),
                (0, 2),
                (1, 2),
                (2, 2),
                (3, 2),
                (4, 2),
                (4, 3),
                (3, 4),
                (4, 4),
            ]
        );
    }

    #[test_case(EXAMPLE1 => (8, (3, 4)))]
    #[test_case(EXAMPLE2 => (33, (5, 8)))]
    #[test_case(EXAMPLE3 => (35, (1, 2)))]
    #[test_case(EXAMPLE4 => (41, (6, 3)))]
    #[test_case(EXAMPLE5 => (210, (11, 13)))]
    fn test_part_1(input: &str) -> (usize, (i32, i32)) {
        let map = parse(input).unwrap();
        find_base_asteroid(&map)
    }

    #[test_case(EXAMPLE6, (8, 3), 36 => (14, 3))]
    #[test_case(EXAMPLE5, (11, 13), 199 => (9, 6))]
    #[test_case(EXAMPLE5, (11, 13), 200 => (8, 2))]
    #[test_case(EXAMPLE5, (11, 13), 201 => (10, 9))]
    fn test_part_2(input: &str, base_position: (i32, i32), nth: usize) -> (i32, i32) {
        let map = parse(input).unwrap();
        find_nth_destroyed_asteroid(&map, base_position, nth)
    }
}
