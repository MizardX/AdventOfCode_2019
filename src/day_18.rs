use std::cmp::Reverse;
use std::collections::hash_map::Entry;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
use std::ops::{Add, AddAssign, Index, IndexMut};

use thiserror::Error;

#[derive(Debug, Error)]
enum ParseError {
    #[error("Invalid tile: {0:?}")]
    InvalidTile(char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Location {
    Entrance(u8),
    Key(u8),
    Door(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tile {
    Open,
    Wall,
    Location(Location),
    Void,
}

impl TryFrom<u8> for Tile {
    type Error = ParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            b'\n' => Self::Void,
            b'.' => Self::Open,
            b'#' => Self::Wall,
            b'@' => Self::Location(Location::Entrance(0)),
            b'a'..=b'z' => Self::Location(Location::Key(value - b'a')),
            b'A'..=b'Z' => Self::Location(Location::Door(value - b'A')),
            _ => return Err(ParseError::InvalidTile(value as char)),
        })
    }
}

type Value = i32;

#[derive(Debug, Clone)]
struct Map<T> {
    data: Vec<T>,
    fallback: T,
    stride: usize,
    width: usize,
    height: usize,
}

impl<T> Map<T> {
    fn new(data: Vec<T>, split: impl Fn(&T) -> bool, fallback: T) -> Self {
        let width = data.iter().position(split).unwrap();
        let stride = width + 1;
        let height = (data.len() + 1) / stride;
        Self {
            data,
            fallback,
            stride,
            width,
            height,
        }
    }

    fn index_to_pos(&self, index: usize) -> Position {
        Position::new(
            Value::try_from(index % self.stride).unwrap(),
            Value::try_from(index / self.stride).unwrap(),
        )
    }
}

impl<T> Index<Position> for Map<T> {
    type Output = T;

    fn index(&self, index: Position) -> &Self::Output {
        if let Ok(x) = usize::try_from(index.x)
            && let Ok(y) = usize::try_from(index.y)
            && (0..self.width).contains(&x)
            && (0..self.height).contains(&y)
        {
            &self.data[x + self.stride * y]
        } else {
            &self.fallback
        }
    }
}

impl<T> IndexMut<Position> for Map<T> {
    fn index_mut(&mut self, index: Position) -> &mut Self::Output {
        if let Ok(x) = usize::try_from(index.x)
            && let Ok(y) = usize::try_from(index.y)
            && (0..self.width).contains(&x)
            && (0..self.height).contains(&y)
        {
            &mut self.data[x + self.stride * y]
        } else {
            panic!("Tried to modify outside the grid")
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Position {
    x: Value,
    y: Value,
}

impl Position {
    const fn new(x: Value, y: Value) -> Self {
        Self { x, y }
    }
}

impl AddAssign<Direction> for Position {
    fn add_assign(&mut self, rhs: Direction) {
        match rhs {
            Direction::Up => self.y -= 1,
            Direction::Right => self.x += 1,
            Direction::Down => self.y += 1,
            Direction::Left => self.x -= 1,
        }
    }
}

impl Add<Direction> for Position {
    type Output = Self;

    fn add(mut self, rhs: Direction) -> Self::Output {
        self += rhs;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Direction {
    const fn all() -> [Self; 4] {
        [Self::Up, Self::Right, Self::Down, Self::Left]
    }
}

#[aoc_generator(day18)]
fn parse(input: &str) -> Result<Map<Tile>, ParseError> {
    let mut data = input
        .bytes()
        .map(Tile::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    let mut entrence_count = 0;
    for tile in &mut data {
        if let Tile::Location(Location::Entrance(n)) = tile {
            *n = entrence_count;
            entrence_count += 1;
        }
    }
    Ok(Map::new(data, |t| matches!(t, Tile::Void), Tile::Void))
}

#[aoc(day18, part1)]
fn part_1(map: &Map<Tile>) -> usize {
    let (locations, positions) = locations_ans_positions(map);
    let neighbors = find_all_neighbors(map, &positions);
    find_all_keys(Location::Entrance(0), &locations, &neighbors).unwrap()
}

#[aoc(day18, part2)]
fn part_2(map: &Map<Tile>) -> usize {
    let (mut locations, mut positions) = locations_ans_positions(map);
    // All but one of the examples have the entrances already expanded, so check before trying.
    let modified_map = if locations
        .iter()
        .any(|l| matches!(l, Location::Entrance(1..=3)))
    {
        map
    } else {
        &expand_entrance(map, &mut locations, &mut positions)
    };

    let neighbors = find_all_neighbors(modified_map, &positions);

    find_all_keys_parallel(
        [
            Location::Entrance(0),
            Location::Entrance(1),
            Location::Entrance(2),
            Location::Entrance(3),
        ],
        &locations,
        &neighbors,
    )
    .unwrap()
}

fn expand_entrance(
    map: &Map<Tile>,
    locations: &mut Vec<Location>,
    positions: &mut Vec<Position>,
) -> Map<Tile> {
    let mut modified_map = map.clone();
    let entrance_index = locations
        .iter()
        .position(|&l| l == Location::Entrance(0))
        .unwrap();
    let pos = positions[entrance_index];
    modified_map[pos] = Tile::Wall;
    modified_map[pos + Direction::Up] = Tile::Wall;
    modified_map[pos + Direction::Right] = Tile::Wall;
    modified_map[pos + Direction::Down] = Tile::Wall;
    modified_map[pos + Direction::Left] = Tile::Wall;
    modified_map[pos + Direction::Up + Direction::Left] = Tile::Location(Location::Entrance(0));
    modified_map[pos + Direction::Up + Direction::Right] = Tile::Location(Location::Entrance(1));
    modified_map[pos + Direction::Down + Direction::Left] = Tile::Location(Location::Entrance(2));
    modified_map[pos + Direction::Down + Direction::Right] = Tile::Location(Location::Entrance(3));
    locations.extend_from_slice(&[
        Location::Entrance(1),
        Location::Entrance(2),
        Location::Entrance(3),
    ]);
    positions[entrance_index] = pos + Direction::Up + Direction::Left;
    positions.extend_from_slice(&[
        pos + Direction::Up + Direction::Right,
        pos + Direction::Down + Direction::Left,
        pos + Direction::Down + Direction::Right,
    ]);
    modified_map
}

fn locations_ans_positions(map: &Map<Tile>) -> (Vec<Location>, Vec<Position>) {
    map.data
        .iter()
        .enumerate()
        .filter_map(|(index, &tile)| {
            if let Tile::Location(loc) = tile {
                Some((loc, map.index_to_pos(index)))
            } else {
                None
            }
        })
        .unzip()
}

fn find_all_neighbors(map: &Map<Tile>, positions: &[Position]) -> Vec<Vec<(Location, usize)>> {
    let mut neighbors = vec![vec![]; positions.len()];
    for (index, &pos) in positions.iter().enumerate() {
        find_neighbors(map, pos, &mut neighbors[index]);
    }
    neighbors
}

fn find_neighbors(map: &Map<Tile>, start: Position, neighbors: &mut Vec<(Location, usize)>) {
    let mut pending = VecDeque::new();
    pending.push_back((start, 0));
    let mut visited = HashSet::new();
    while let Some((pos, dist)) = pending.pop_front() {
        if !visited.insert(pos) {
            continue;
        }
        if pos != start
            && let Tile::Location(loc) = map[pos]
        {
            neighbors.push((loc, dist));
            continue;
        }
        for dir in Direction::all() {
            let next = pos + dir;
            if matches!(map[next], Tile::Wall | Tile::Void) || visited.contains(&next) {
                continue;
            }
            pending.push_back((next, dist + 1));
        }
    }
}

fn find_all_keys(
    start: Location,
    locations: &[Location],
    neighbors: &[Vec<(Location, usize)>],
) -> Option<usize> {
    let all_keys_mask = locations
        .iter()
        .map(|l| if let &Location::Key(k) = l { 1 << k } else { 0 })
        .sum();
    let start_index = locations.iter().position(|&l| l == start).unwrap();
    let mut visited = HashMap::<(usize, u32), usize>::new();
    let mut pending = BinaryHeap::new();
    pending.push((Reverse(0), start_index, 0_u32));
    while let Some((Reverse(dist), index, mut keys)) = pending.pop() {
        match visited.entry((index, keys)) {
            Entry::Occupied(o) if *o.get() <= dist => {
                continue;
            }
            Entry::Occupied(mut o) => {
                o.insert(dist);
            }
            Entry::Vacant(v) => {
                v.insert(dist);
            }
        }
        if let Location::Key(key) = locations[index] {
            keys |= 1 << key;
        }
        if keys == all_keys_mask {
            return Some(dist);
        }
        for &(next, delta) in &neighbors[index] {
            if let Location::Door(key) = next
                && (keys & (1 << key)) == 0
            {
                continue;
            }
            let next_ix = locations.iter().position(|&l| l == next).unwrap();
            if let Some(&prev_dist) = visited.get(&(next_ix, keys))
                && dist + delta >= prev_dist
            {
                continue;
            }
            pending.push((Reverse(dist + delta), next_ix, keys));
        }
    }
    None
}

fn find_all_keys_parallel(
    starts: [Location; 4],
    locations: &[Location],
    neighbors: &[Vec<(Location, usize)>],
) -> Option<usize> {
    let all_keys_mask = locations
        .iter()
        .map(|l| if let &Location::Key(k) = l { 1 << k } else { 0 })
        .sum();
    let start_indices = starts.map(|start| locations.iter().position(|&l| l == start).unwrap());
    let mut visited = HashMap::<([usize; 4], u32), usize>::new();
    let mut pending = BinaryHeap::new();
    pending.push((Reverse(0), start_indices, 0_u32));
    while let Some((Reverse(dist), indices, mut keys)) = pending.pop() {
        match visited.entry((indices, keys)) {
            Entry::Occupied(o) if *o.get() <= dist => {
                continue;
            }
            Entry::Occupied(mut o) => {
                o.insert(dist);
            }
            Entry::Vacant(v) => {
                v.insert(dist);
            }
        }
        for index in indices {
            if let Location::Key(key) = locations[index] {
                keys |= 1 << key;
            }
        }
        if keys == all_keys_mask {
            return Some(dist);
        }
        for (ix, index) in indices.into_iter().enumerate() {
            for &(next, delta) in &neighbors[index] {
                if let Location::Door(key) = next
                    && (keys & (1 << key)) == 0
                {
                    continue;
                }
                let next_ix = locations.iter().position(|&l| l == next).unwrap();
                let mut new_indices = indices;
                new_indices[ix] = next_ix;
                if let Some(&prev_dist) = visited.get(&(new_indices, keys))
                    && dist + delta >= prev_dist
                {
                    continue;
                }
                pending.push((Reverse(dist + delta), new_indices, keys));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    const EXAMPLE1: &str = "\
        #########\n\
        #b.A.@.a#\n\
        #########\
    ";

    const EXAMPLE2: &str = "\
        ########################\n\
        #f.D.E.e.C.b.A.@.a.B.c.#\n\
        ######################.#\n\
        #d.....................#\n\
        ########################\
    ";

    const EXAMPLE3: &str = "\
        ########################\n\
        #...............b.C.D.f#\n\
        #.######################\n\
        #.....@.a.B.c.d.A.e.F.g#\n\
        ########################\
    ";

    const EXAMPLE4: &str = "\
        #################\n\
        #i.G..c...e..H.p#\n\
        ########.########\n\
        #j.A..b...f..D.o#\n\
        ########@########\n\
        #k.E..a...g..B.n#\n\
        ########.########\n\
        #l.F..d...h..C.m#\n\
        #################\
    ";

    const EXAMPLE5: &str = "\
        ########################\n\
        #@..............ac.GI.b#\n\
        ###d#e#f################\n\
        ###A#B#C################\n\
        ###g#h#i################\n\
        ########################\
    ";

    const EXAMPLE6: &str = "\
        #######\n\
        #a.#Cd#\n\
        ##...##\n\
        ##.@.##\n\
        ##...##\n\
        #cB#Ab#\n\
        #######\
    ";

    const EXAMPLE6_ALT: &str = "\
        #######\n\
        #a.#Cd#\n\
        ##@#@##\n\
        #######\n\
        ##@#@##\n\
        #cB#Ab#\n\
        #######\
    ";

    const EXAMPLE7: &str = "\
        ###############\n\
        #d.ABC.#.....a#\n\
        ######@#@######\n\
        ###############\n\
        ######@#@######\n\
        #b.....#.....c#\n\
        ###############\
    ";

    const EXAMPLE8: &str = "\
        #############\n\
        #DcBa.#.GhKl#\n\
        #.###@#@#I###\n\
        #e#d#####j#k#\n\
        ###C#@#@###J#\n\
        #fEbA.#.FgHi#\n\
        #############\
    ";

    const EXAMPLE9: &str = "\
        #############\n\
        #g#f.D#..h#l#\n\
        #F###e#E###.#\n\
        #dCba@#@BcIJ#\n\
        #############\n\
        #nK.L@#@G...#\n\
        #M###N#H###.#\n\
        #o#m..#i#jk.#\n\
        #############\
    ";

    #[test_case(EXAMPLE1 => 8)]
    #[test_case(EXAMPLE2 => 86)]
    #[test_case(EXAMPLE3 => 132)]
    #[test_case(EXAMPLE4 => 136)]
    #[test_case(EXAMPLE5 => 81)]
    fn test_part_1(input: &str) -> usize {
        let map = parse(input).unwrap();
        part_1(&map)
    }

    #[test_case(EXAMPLE6 => 8)]
    #[test_case(EXAMPLE6_ALT => 8)]
    #[test_case(EXAMPLE7 => 24)]
    #[test_case(EXAMPLE8 => 32)]
    #[test_case(EXAMPLE9 => 72)]
    fn test_part_2(input: &str) -> usize {
        let map = parse(input).unwrap();
        part_2(&map)
    }
}
