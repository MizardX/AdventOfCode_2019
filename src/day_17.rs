use std::fmt::{Display, Write};
use std::num::ParseIntError;
use std::ops::{Add, AddAssign, Index};

use crate::machine::{parse_program, Machine, MachineError, Value};

#[aoc_generator(day17)]
fn parse(input: &str) -> Result<Vec<Value>, ParseIntError> {
    parse_program(input)
}

#[aoc(day17, part1)]
fn part_1(program: &[Value]) -> usize {
    let mut machine = Machine::new(program);
    let map = read_map(&mut machine).unwrap();
    sum_alignment_parameters(&map)
}

fn read_map(machine: &mut Machine) -> Result<Map<u8>, MachineError> {
    match machine.run_until_stopped() {
        Ok(()) | Err(MachineError::EmptyInput) => {}
        err => err?,
    }
    let mut output = Vec::new();
    let mut line_len = 0;
    while let Some(x) = machine.outputs.pop_front() {
        let ch = u8::try_from(x).unwrap();
        if ch == b'\n' {
            if line_len == 0 {
                break;
            }
            line_len = 0;
        } else {
            line_len += 1;
        }
        output.push(ch);
    }
    Ok(Map::new(output, |&ch| ch == b'\n', b' '))
}

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

fn sum_alignment_parameters(map: &Map<u8>) -> usize {
    let mut alignment_sum = 0;
    for y in 1..map.height - 1 {
        for x in 1..map.width - 1 {
            let pos = Position::new(i64::try_from(x).unwrap(), i64::try_from(y).unwrap());
            if map[pos] == b'#'
                && map[pos + Direction::Up] == b'#'
                && map[pos + Direction::Left] == b'#'
                && map[pos + Direction::Right] == b'#'
                && map[pos + Direction::Down] == b'#'
            {
                alignment_sum += x * y;
            }
        }
    }
    alignment_sum
}

#[aoc(day17, part2)]
fn part_2(program: &[Value]) -> Value {
    let mut machine = Machine::new(program);
    machine.write(0, 2);

    let map = read_map(&mut machine).unwrap();

    let path = collect_path(&map);

    let subdiv = PathSubdivision::subdivide_path(&path).unwrap();
    let mut program_text = subdiv.to_string();
    program_text.push_str("n\n");

    machine.inputs.extend(program_text.bytes().map(Value::from));

    machine.run_until_stopped().unwrap();

    machine.outputs.pop_back().unwrap()
}

fn collect_path(map: &Map<u8>) -> Vec<Action> {
    const fn is_open(ch: u8) -> bool {
        matches!(ch, b'#' | b'<' | b'^' | b'>' | b'v')
    }
    let (mut dir, mut pos) = map
        .data
        .iter()
        .enumerate()
        .find_map(|(ix, &ch)| Some((Direction::try_from(ch).ok()?, map.index_to_pos(ix))))
        .unwrap();
    let mut path = Vec::new();
    loop {
        let mut forward_count = 0;
        while is_open(map[pos + dir]) {
            pos += dir;
            forward_count += 1;
        }
        if forward_count > 0 {
            path.push(Action::Forward(forward_count));
        }
        if is_open(map[pos + dir.turn_left()]) {
            dir = dir.turn_left();
            path.push(Action::Left);
        } else if is_open(map[pos + dir.turn_right()]) {
            dir = dir.turn_right();
            path.push(Action::Right);
        } else {
            // End of the path
            break;
        }
    }
    path
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Subroutine {
    A,
    B,
    C,
}

impl Subroutine {
    const fn all() -> [Self; 3] {
        [Self::A, Self::B, Self::C]
    }
}

impl Display for Subroutine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char(match self {
            Self::A => 'A',
            Self::B => 'B',
            Self::C => 'C',
        })
    }
}

#[derive(Debug, Clone, Default)]
struct PathSubdivision {
    main: Vec<Subroutine>,
    subroutines: [Vec<Action>; Subroutine::all().len()],
}

impl PathSubdivision {
    fn walk(&mut self, path: &[Action]) -> bool {
        if path.is_empty() {
            return self.main.len() * 2 - 1 <= 20
                && self
                    .subroutines
                    .iter()
                    .all(|s| s.iter().map(|a| a.len() + 1).sum::<usize>() - 1 <= 20);
        }
        for sub in Subroutine::all() {
            let sub_ix = sub as usize;
            if self.subroutines[sub_ix].is_empty() {
                self.main.push(sub);
                for (path_ix, &action) in path.iter().enumerate() {
                    self.subroutines[sub_ix].push(action);
                    if self.walk(&path[path_ix + 1..]) {
                        return true;
                    }
                }
                self.subroutines[sub_ix].clear();
                self.main.pop();
                return false;
            }
            if path.starts_with(&self.subroutines[sub_ix]) {
                self.main.push(sub);
                if self.walk(&path[self.subroutines[sub_ix].len()..]) {
                    return true;
                }
                self.main.pop();
            }
        }
        false
    }

    fn subdivide_path(path: &[Action]) -> Option<Self> {
        let mut subdiv = Self::default();
        subdiv.walk(path).then_some(subdiv)
    }
}

impl Display for PathSubdivision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, sub) in self.main.iter().enumerate() {
            if i > 0 {
                f.write_char(',')?;
            }
            write!(f, "{sub}")?;
        }
        writeln!(f)?;
        for sub in &self.subroutines {
            for (i, action) in sub.iter().enumerate() {
                if i > 0 {
                    f.write_char(',')?;
                }
                write!(f, "{action}")?;
            }
            writeln!(f)?;
        }
        Ok(())
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
    const fn turn_left(self) -> Self {
        match self {
            Self::Up => Self::Left,
            Self::Right => Self::Up,
            Self::Down => Self::Right,
            Self::Left => Self::Down,
        }
    }
    const fn turn_right(self) -> Self {
        match self {
            Self::Up => Self::Right,
            Self::Right => Self::Down,
            Self::Down => Self::Left,
            Self::Left => Self::Up,
        }
    }
}

impl TryFrom<u8> for Direction {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            b'<' => Self::Left,
            b'^' => Self::Up,
            b'>' => Self::Right,
            b'v' => Self::Down,
            _ => return Err(()),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    Left,
    Right,
    Forward(u8),
}

impl Action {
    const fn len(self) -> usize {
        match self {
            Self::Left | Self::Right | Self::Forward(0..=9) => 1,
            Self::Forward(..) => 2,
        }
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Left => f.write_char('L'),
            Self::Right => f.write_char('R'),
            Self::Forward(n) => write!(f, "{n}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE1: &str = "\
        ..#..........\n\
        ..#..........\n\
        #######...###\n\
        #.#...#...#.#\n\
        #############\n\
        ..#...#...#..\n\
        ..#####...^..\
    ";

    const EXAMPLE2: &str = "\
        #######...#####\n\
        #.....#...#...#\n\
        #.....#...#...#\n\
        ......#...#...#\n\
        ......#...###.#\n\
        ......#.....#.#\n\
        ^########...#.#\n\
        ......#.#...#.#\n\
        ......#########\n\
        ........#...#..\n\
        ....#########..\n\
        ....#...#......\n\
        ....#...#......\n\
        ....#...#......\n\
        ....#####......\
    ";

    #[test]
    fn test_part_1() {
        let map = Map::new(EXAMPLE1.as_bytes().to_vec(), |&ch| ch == b'\n', b' ');
        let result = sum_alignment_parameters(&map);
        assert_eq!(result, 76);
    }

    #[test]
    fn test_find_path() {
        let map = Map::new(EXAMPLE2.as_bytes().to_vec(), |&ch| ch == b'\n', b' ');
        let path = collect_path(&map);
        let mut displayed = String::new();
        for action in path {
            if !displayed.is_empty() {
                displayed.push(',');
            }
            write!(&mut displayed, "{action}").unwrap();
        }

        assert_eq!(
            displayed,
            "R,8,R,8,R,4,R,4,R,8,L,6,L,2,R,4,R,4,R,8,R,8,R,8,L,6,L,2"
        );
    }

    #[test]
    fn test_subdivide() {
        let map = Map::new(EXAMPLE2.as_bytes().to_vec(), |&ch| ch == b'\n', b' ');
        let path = collect_path(&map);
        let subdiv = PathSubdivision::subdivide_path(&path).unwrap();
        let text = subdiv.to_string();
        for line in text.lines() {
            assert!(line.len() <= 20, "len <= 20: {line:?}");
        }
        let mut reconstucted = Vec::new();
        for &sub in &subdiv.main {
            reconstucted.extend_from_slice(&subdiv.subroutines[sub as usize]);
        }
        assert_eq!(path, reconstucted);
    }
}
