#![allow(unused)]

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::{Display, Write};
use std::mem::MaybeUninit;
use std::ops::{Add, AddAssign, Index, IndexMut, RangeInclusive};
use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(C)]
struct Grid<T> {
    tiles: Box<[T]>,
    rows: usize,
    cols: usize,
    fallback: T,
}

impl<T> Grid<T> {
    fn new(rows: usize, cols: usize, cb: impl Fn(usize, usize) -> T, fallback: T) -> Self {
        let tiles = (0..rows * cols)
            .map(|ix| cb(ix / cols, ix % cols))
            .collect();
        Self {
            tiles,
            rows,
            cols,
            fallback,
        }
    }

    fn to_index(&self, pos: Position) -> Option<usize> {
        if let Ok(row) = usize::try_from(pos.y)
            && let Ok(col) = usize::try_from(pos.x)
            && row < self.rows
            && col < self.cols
        {
            Some(row * self.cols + col)
        } else {
            None
        }
    }
}

impl<T> Index<Position> for Grid<T> {
    type Output = T;

    fn index(&self, pos: Position) -> &Self::Output {
        self.to_index(pos)
            .map_or(&self.fallback, |index| &self.tiles[index])
    }
}

impl<T> IndexMut<Position> for Grid<T> {
    fn index_mut(&mut self, pos: Position) -> &mut Self::Output {
        self.to_index(pos).map_or_else(
            || {
                panic!("outside: {pos:?}");
            },
            |index| &mut self.tiles[index],
        )
    }
}

impl<T> Display for Grid<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in 0..self.rows {
            if row > 0 {
                writeln!(f)?;
            }
            for cell in &self.tiles[row * self.cols..(row + 1) * self.cols] {
                cell.fmt(f)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Position {
    x: i32,
    y: i32,
}

impl Position {
    const fn new(x: i32, y: i32) -> Self {
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

#[derive(Debug, Error)]
enum ParseError {
    #[error("Invalid tile: {0:?}")]
    InvalidTile(char),
    #[error("Portal not outside an open position")]
    InvalidPortalPosition,
    #[error("Portal does not have exactly one partner")]
    UnmatchedPortal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tile {
    Open,
    Wall,
    Void,
    Portal(char, char),
}

impl Tile {
    const fn is_passable(self) -> bool {
        matches!(self, Self::Open | Self::Portal(..))
    }
}

impl TryFrom<u8> for Tile {
    type Error = ParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            b'.' => Self::Open,
            b'#' => Self::Wall,
            b' ' | b'A'..=b'Z' => Self::Void,
            _ => return Err(ParseError::InvalidTile(value as char)),
        })
    }
}

impl Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char(match *self {
            Self::Open => '.',
            Self::Wall => '#',
            Self::Void => ' ',
            Self::Portal(ch1, _) => ch1,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Measurement {
    full_width: usize,
    full_height: usize,
    grid_width: usize,
    grid_height: usize,
    hole_offset_x: usize,
    hole_offset_y: usize,
    hole_width: usize,
    hole_height: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MeasurePosition {
    GridBorder(Direction, Position),
    Grid(Position),
    HoleBorder(Direction, Position),
    Void,
}

impl Measurement {
    fn measure(input: &str) -> Self {
        let full_height = input.lines().count();
        let mut lines = input.lines();
        let first_line = lines.next().unwrap();
        let full_width = first_line.len();
        let grid_width = full_width - 4;
        let grid_height = full_height - 4;

        let hole_offset_y = input
            .lines()
            .skip(2)
            .position(|l| l[2..grid_width + 2].contains(' '))
            .unwrap()
            + 2;
        let hole_height = full_height
            - input
                .lines()
                .rev()
                .skip(2)
                .position(|l| l[2..grid_width + 2].contains(' '))
                .unwrap()
            - 2
            - hole_offset_y;
        let hole_first_line: &str = input.lines().nth(hole_offset_y).unwrap();
        let hole_offset_x = hole_first_line[2..grid_width + 2]
            .bytes()
            .position(|ch| ch == b' ')
            .unwrap()
            + 2;
        let hole_width = hole_first_line[2..grid_width + 2]
            .bytes()
            .rposition(|ch| ch == b' ')
            .unwrap()
            + 3
            - hole_offset_x;
        Self {
            full_width,
            full_height,
            grid_width,
            grid_height,
            hole_offset_x,
            hole_offset_y,
            hole_width,
            hole_height,
        }
    }

    fn locate(&self, row: usize, col: usize) -> MeasurePosition {
        let x_region: u8 = match col {
            _ if col < 2 => 0,
            _ if col < self.hole_offset_x => 1,
            _ if col < self.hole_offset_x + 2 => 2,
            _ if col < self.hole_offset_x + self.hole_width - 2 => 3,
            _ if col < self.hole_offset_x + self.hole_width => 4,
            _ if col < 2 + self.grid_width => 5,
            _ => 6,
        };
        let y_region: u8 = match row {
            _ if row < 2 => 0,
            _ if row < self.hole_offset_y => 1,
            _ if row < self.hole_offset_y + 2 => 2,
            _ if row < self.hole_offset_y + self.hole_height - 2 => 3,
            _ if row < self.hole_offset_y + self.hole_height => 4,
            _ if row < 2 + self.grid_height => 5,
            _ => 6,
        };
        match (x_region, y_region) {
            (0 | 6, 0 | 6) | (3, 3) => MeasurePosition::Void,
            (0, _) => MeasurePosition::GridBorder(
                Direction::Left,
                Position::new(0, i32::try_from(row - 2).unwrap()),
            ),
            (_, 0) => MeasurePosition::GridBorder(
                Direction::Up,
                Position::new(i32::try_from(col - 2).unwrap(), 0),
            ),
            (_, 6) => MeasurePosition::GridBorder(
                Direction::Down,
                Position::new(
                    i32::try_from(col - 2).unwrap(),
                    i32::try_from(self.grid_height - 1).unwrap(),
                ),
            ),
            (6, _) => MeasurePosition::GridBorder(
                Direction::Right,
                Position::new(
                    i32::try_from(self.grid_width - 1).unwrap(),
                    i32::try_from(row - 2).unwrap(),
                ),
            ),
            (_, 1 | 5) | (1 | 5, _) => MeasurePosition::Grid(Position::new(
                i32::try_from(col - 2).unwrap(),
                i32::try_from(row - 2).unwrap(),
            )),
            // Unsymmetric: The hole corners are assigned to the sides
            (2, _) => MeasurePosition::HoleBorder(
                Direction::Left,
                Position::new(
                    i32::try_from(self.hole_offset_x - 3).unwrap(),
                    i32::try_from(row - 2).unwrap(),
                ),
            ),
            (3, 2) => MeasurePosition::HoleBorder(
                Direction::Up,
                Position::new(
                    i32::try_from(col - 2).unwrap(),
                    i32::try_from(self.hole_offset_y - 3).unwrap(),
                ),
            ),
            (3, 4) => MeasurePosition::HoleBorder(
                Direction::Down,
                Position::new(
                    i32::try_from(col - 2).unwrap(),
                    i32::try_from(self.hole_offset_y + self.hole_height - 2).unwrap(),
                ),
            ),
            // Unsymmetric: The hole corners are assigned to the sides
            (4, _) => MeasurePosition::HoleBorder(
                Direction::Right,
                Position::new(
                    i32::try_from(self.hole_offset_x + self.hole_width - 2).unwrap(),
                    i32::try_from(row - 2).unwrap(),
                ),
            ),
            _ => MeasurePosition::Void,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Maze {
    grid: Grid<Tile>,
    warps: HashMap<Position, (Position, i32)>,
    start: Option<Position>,
    goal: Option<Position>,
}

impl FromStr for Maze {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let measurement = Measurement::measure(input);

        let mut grid = Grid::new(
            measurement.grid_height,
            measurement.grid_width,
            |_, _| Tile::Void,
            Tile::Void,
        );

        // First pass, just fill in the grid
        for (r, line) in input.lines().enumerate() {
            for (c, ch) in line.bytes().enumerate() {
                if let MeasurePosition::Grid(pos) = measurement.locate(r, c) {
                    grid[pos] = ch.try_into()?;
                }
            }
        }

        // Second pass, fill in the portals
        let mut portals = HashMap::<(char, char), Vec<(Position, i32)>>::new();
        for (r, line) in input.lines().enumerate() {
            for (c, ch) in line.bytes().enumerate() {
                match measurement.locate(r, c) {
                    MeasurePosition::GridBorder(_, pos) => {
                        if ch.is_ascii_uppercase() {
                            match &mut grid[pos] {
                                Tile::Portal(a, b) => {
                                    *b = ch as char;
                                    portals.entry((*a, *b)).or_default().push((pos, -1));
                                }
                                tile @ Tile::Open => *tile = Tile::Portal(ch as char, '_'),
                                tile => {
                                    return Err(ParseError::InvalidPortalPosition);
                                }
                            }
                        }
                    }
                    MeasurePosition::HoleBorder(_, pos) => {
                        if ch.is_ascii_uppercase() {
                            match &mut grid[pos] {
                                Tile::Portal(a, b) => {
                                    *b = ch as char;
                                    portals.entry((*a, *b)).or_default().push((pos, 1));
                                }
                                tile @ Tile::Open => *tile = Tile::Portal(ch as char, '_'),
                                tile => {
                                    return Err(ParseError::InvalidPortalPosition);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        let mut warps = HashMap::new();
        let mut unmatched = Vec::new();
        for group in portals.values() {
            if let &[(a, delta_a), (b, delta_b)] = group.as_slice() {
                warps.insert(a, (b, delta_a));
                warps.insert(b, (a, delta_b));
            } else {
                unmatched.extend_from_slice(group);
            }
        }

        let mut start = None;
        let mut goal = None;

        for &(pos, _) in &unmatched {
            match grid[pos] {
                Tile::Portal('A', 'A') => start = Some(pos),
                Tile::Portal('Z', 'Z') => goal = Some(pos),
                _ => return Err(ParseError::UnmatchedPortal),
            }
        }

        Ok(Self {
            grid,
            warps,
            start,
            goal,
        })
    }
}

#[aoc_generator(day20)]
fn parse(input: &str) -> Result<Maze, ParseError> {
    input.parse()
}

#[aoc(day20, part1)]
fn part_1(maze: &Maze) -> usize {
    let mut pending: VecDeque<(Position, usize)> = [(maze.start.unwrap(), 0)].into();
    let mut visited = HashSet::new();
    while let Some((pos, dist)) = pending.pop_front() {
        if !visited.insert(pos) {
            continue;
        }
        if pos == maze.goal.unwrap() {
            return dist;
        }
        if let Some(&(twin, _)) = maze.warps.get(&pos)
            && !visited.contains(&twin)
        {
            pending.push_back((twin, dist + 1));
        }
        for dir in Direction::all() {
            let next = pos + dir;
            if maze.grid[next].is_passable() && !visited.contains(&next) {
                pending.push_back((next, dist + 1));
            }
        }
    }
    0
}

#[aoc(day20, part2)]
fn part_2(maze: &Maze) -> usize {
    let mut pending: VecDeque<(Position, u32, usize)> = [(maze.start.unwrap(), 0, 0)].into();
    let mut visited = HashSet::new();
    let mut max_depth = 0;
    while let Some((pos, depth, dist)) = pending.pop_front() {
        if !visited.insert((pos, depth)) {
            continue;
        }
        max_depth = max_depth.max(depth);
        if (pos, depth) == (maze.goal.unwrap(), 0) {
            println!("Max depth {max_depth}");
            return dist;
        }
        if let Some(&(twin, delta)) = maze.warps.get(&pos)
            && let Some(twindepth) = depth.checked_add_signed(delta)
            && !visited.contains(&(twin, twindepth))
        {
            pending.push_back((twin, twindepth, dist + 1));
        }
        for dir in Direction::all() {
            let next = pos + dir;
            if maze.grid[next].is_passable() && !visited.contains(&(next, depth)) {
                pending.push_back((next, depth, dist + 1));
            }
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    const EXAMPLE1: &str = "\
        `````````A```````````\n\
        `````````A```````````\n\
        ``#######.#########``\n\
        ``#######.........#``\n\
        ``#######.#######.#``\n\
        ``#######.#######.#``\n\
        ``#######.#######.#``\n\
        ``#####``B````###.#``\n\
        BC...##``C````###.#``\n\
        ``##.##```````###.#``\n\
        ``##...DE``F``###.#``\n\
        ``#####````G``###.#``\n\
        ``#########.#####.#``\n\
        DE..#######...###.#``\n\
        ``#.#########.###.#``\n\
        FG..#########.....#``\n\
        ``###########.#####``\n\
        `````````````Z```````\n\
        `````````````Z```````\
    ";

    const EXAMPLE2: &str = "\
        ```````````````````A```````````````\n\
        ```````````````````A```````````````\n\
        ``#################.#############``\n\
        ``#.#...#...................#.#.#``\n\
        ``#.#.#.###.###.###.#########.#.#``\n\
        ``#.#.#.......#...#.....#.#.#...#``\n\
        ``#.#########.###.#####.#.#.###.#``\n\
        ``#.............#.#.....#.......#``\n\
        ``###.###########.###.#####.#.#.#``\n\
        ``#.....#````````A```C````#.#.#.#``\n\
        ``#######````````S```P````#####.#``\n\
        ``#.#...#`````````````````#......VT\n\
        ``#.#.#.#`````````````````#.#####``\n\
        ``#...#.#```````````````YN....#.#``\n\
        ``#.###.#`````````````````#####.#``\n\
        DI....#.#`````````````````#.....#``\n\
        ``#####.#`````````````````#.###.#``\n\
        ZZ......#```````````````QG....#..AS\n\
        ``###.###`````````````````#######``\n\
        JO..#.#.#`````````````````#.....#``\n\
        ``#.#.#.#`````````````````###.#.#``\n\
        ``#...#..DI`````````````BU....#..LF\n\
        ``#####.#`````````````````#.#####``\n\
        YN......#```````````````VT..#....QG\n\
        ``#.###.#`````````````````#.###.#``\n\
        ``#.#...#`````````````````#.....#``\n\
        ``###.###````J`L`````J````#.#.###``\n\
        ``#.....#````O`F`````P````#.#...#``\n\
        ``#.###.#####.#.#####.#####.###.#``\n\
        ``#...#.#.#...#.....#.....#.#...#``\n\
        ``#.#####.###.###.#.#.#########.#``\n\
        ``#...#.#.....#...#.#.#.#.....#.#``\n\
        ``#.###.#####.###.###.#.#.#######``\n\
        ``#.#.........#...#.............#``\n\
        ``#########.###.###.#############``\n\
        ```````````B```J```C```````````````\n\
        ```````````U```P```P```````````````\
    ";

    const EXAMPLE3: &str = "\
        `````````````Z`L`X`W```````C`````````````````\n\
        `````````````Z`P`Q`B```````K`````````````````\n\
        ``###########.#.#.#.#######.###############``\n\
        ``#...#.......#.#.......#.#.......#.#.#...#``\n\
        ``###.#.#.#.#.#.#.#.###.#.#.#######.#.#.###``\n\
        ``#.#...#.#.#...#.#.#...#...#...#.#.......#``\n\
        ``#.###.#######.###.###.#.###.###.#.#######``\n\
        ``#...#.......#.#...#...#.............#...#``\n\
        ``#.#########.#######.#.#######.#######.###``\n\
        ``#...#.#````F```````R`I```````Z````#.#.#.#``\n\
        ``#.###.#````D```````E`C```````H````#.#.#.#``\n\
        ``#.#...#```````````````````````````#...#.#``\n\
        ``#.###.#```````````````````````````#.###.#``\n\
        ``#.#....OA```````````````````````WB..#.#..ZH\n\
        ``#.###.#```````````````````````````#.#.#.#``\n\
        CJ......#```````````````````````````#.....#``\n\
        ``#######```````````````````````````#######``\n\
        ``#.#....CK`````````````````````````#......IC\n\
        ``#.###.#```````````````````````````#.###.#``\n\
        ``#.....#```````````````````````````#...#.#``\n\
        ``###.###```````````````````````````#.#.#.#``\n\
        XF....#.#`````````````````````````RF..#.#.#``\n\
        ``#####.#```````````````````````````#######``\n\
        ``#......CJ```````````````````````NM..#...#``\n\
        ``###.#.#```````````````````````````#.###.#``\n\
        RE....#.#```````````````````````````#......RF\n\
        ``###.###````````X```X```````L``````#.#.#.#``\n\
        ``#.....#````````F```Q```````P``````#.#.#.#``\n\
        ``###.###########.###.#######.#########.###``\n\
        ``#.....#...#.....#.......#...#.....#.#...#``\n\
        ``#####.#.###.#######.#######.###.###.#.#.#``\n\
        ``#.......#.......#.#.#.#.#...#...#...#.#.#``\n\
        ``#####.###.#####.#.#.#.#.###.###.#.###.###``\n\
        ``#.......#.....#.#...#...............#...#``\n\
        ``#############.#.#.###.###################``\n\
        ```````````````A`O`F```N`````````````````````\n\
        ```````````````A`A`D```M`````````````````````\
    ";

    fn fix_example(input: &str) -> String {
        input.replace('`', " ")
    }

    #[test]
    fn test_measure() {
        let measure = Measurement::measure(&fix_example(EXAMPLE1));
        assert_eq!(measure.full_width, 21);
        assert_eq!(measure.grid_width, 17);
        assert_eq!(measure.hole_offset_x, 7);
        assert_eq!(measure.hole_width, 7);
        assert_eq!(measure.full_height, 19);
        assert_eq!(measure.grid_height, 15);
        assert_eq!(measure.hole_offset_y, 7);
        assert_eq!(measure.hole_height, 5);
        let mut count_grid = 0;
        let mut count_grid_border = 0;
        let mut count_hole_border = 0;
        let mut count_void = 0;
        for row in 0..measure.full_height {
            for col in 0..measure.full_width {
                match measure.locate(row, col) {
                    MeasurePosition::GridBorder(..) => count_grid_border += 1,
                    MeasurePosition::Grid(..) => count_grid += 1,
                    MeasurePosition::HoleBorder(..) => count_hole_border += 1,
                    MeasurePosition::Void => count_void += 1,
                }
            }
        }
        assert_eq!(count_grid_border, (17 + 15) * 2 * 2); // 4 sides, except corners
        assert_eq!(count_grid, 17 * 15 - 7 * 5); // grid minus hole
        assert_eq!(count_hole_border, (3 + 1) * 2 * 2 + 2 * 2 * 4); // two spaces in hole closest to the maze wall, including 4 corners
        assert_eq!(count_void, 2 * 2 * 4 + 3); // 4 outside corners + center
    }

    #[test]
    fn test_parse() {
        let res = parse(&fix_example(EXAMPLE1)).unwrap();
        assert_eq!(res.start, Some(Position::new(7, 0)));
        assert_eq!(res.goal, Some(Position::new(11, 14)));
        let expected = [
            (Position::new(0, 6), Position::new(7, 4)),
            (Position::new(0, 13), Position::new(9, 10)),
            (Position::new(9, 10), Position::new(0, 13)),
            (Position::new(0, 11), Position::new(4, 8)),
            (Position::new(4, 8), Position::new(0, 11)),
            (Position::new(7, 4), Position::new(0, 6)),
        ];
        assert_eq!(res.warps.len(), expected.len());
        for (p1, p2) in expected {
            assert_eq!(res.warps[&p1].0, p2);
            assert_eq!(res.warps[&p2].0, p1);
        }
    }

    #[test_case(EXAMPLE1 => 23)]
    #[test_case(EXAMPLE2 => 58)]
    fn test_part_1(input: &str) -> usize {
        let maze = parse(&fix_example(input)).unwrap();
        part_1(&maze)
    }

    #[test_case(EXAMPLE1 => 26)]
    // EXAMPLE2 would never finish, and would eventually run out of memory
    #[test_case(EXAMPLE3 => 396)]
    fn test_part_2(input: &str) -> usize {
        let maze = parse(&fix_example(input)).unwrap();
        part_2(&maze)
    }
}
