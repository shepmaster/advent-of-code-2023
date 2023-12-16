use snafu::prelude::*;
use std::{
    cmp,
    collections::{BTreeMap, BTreeSet},
    str::FromStr,
};

const INPUT: &str = include_str!("../input");

#[snafu::report]
fn main() -> Result<(), ParseMapError> {
    let tiles = energized_tiles(INPUT)?;
    // Part 1: 7562
    println!("{tiles}");

    Ok(())
}

fn energized_tiles(s: &str) -> Result<usize, ParseMapError> {
    let map: Map = s.parse()?;

    use direction_shorthands::*;
    use square_shorthands::*;

    let mut queue = vec![((0, 0), R)];
    let mut visited = BTreeMap::new();

    while let Some((coord, direction)) = queue.pop() {
        let visited = visited.entry(coord).or_insert_with(BTreeSet::new);
        if !visited.insert(direction) {
            // Already visited this, no need to re-visit
            continue;
        }

        match (map.squares.get(&coord), direction) {
            (Some(Vs), R | L) => {
                for d in [U, D] {
                    queue.extend(map.cast(coord, d));
                }
            }

            (Some(Hs), U | D) => {
                for d in [L, R] {
                    queue.extend(map.cast(coord, d));
                }
            }

            (Some(Dr), U) => queue.extend(map.cast(coord, L)),
            (Some(Dr), R) => queue.extend(map.cast(coord, D)),
            (Some(Dr), D) => queue.extend(map.cast(coord, R)),
            (Some(Dr), L) => queue.extend(map.cast(coord, U)),

            (Some(Dl), U) => queue.extend(map.cast(coord, R)),
            (Some(Dl), R) => queue.extend(map.cast(coord, U)),
            (Some(Dl), D) => queue.extend(map.cast(coord, L)),
            (Some(Dl), L) => queue.extend(map.cast(coord, D)),

            (Some(Vs), U | D) | (Some(Hs), L | R) | (None, _) => {
                queue.extend(map.cast(coord, direction));
            }
        }
    }

    Ok(visited.len())
}

type Coord = (usize, usize);

struct Map {
    squares: BTreeMap<Coord, Square>,
    x_max: usize,
    y_max: usize,
}

impl Map {
    fn cast(&self, start: Coord, dir: Direction) -> Option<(Coord, Direction)> {
        self.go(start, dir).map(|c| (c, dir))
    }

    fn go(&self, start: Coord, dir: Direction) -> Option<Coord> {
        use direction_shorthands::*;

        let Self { x_max, y_max, .. } = *self;
        let (x, y) = start;

        let cap = |v, max| {
            if v > max {
                None
            } else {
                Some(v)
            }
        };

        match dir {
            U => Some((x, y.checked_sub(1)?)),
            R => Some((cap(x.checked_add(1)?, x_max)?, y)),
            D => Some((x, cap(y.checked_add(1)?, y_max)?)),
            L => Some((x.checked_sub(1)?, y)),
        }
    }
}

impl FromStr for Map {
    type Err = ParseMapError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut squares = BTreeMap::new();
        let mut x_max = 0;
        let mut y_max = 0;

        for (y, line) in s.lines().enumerate() {
            for (x, c) in line.chars().enumerate() {
                if let Some(s) = Square::try_from(c).context(SquareSnafu { x, y, c })? {
                    squares.insert((x, y), s);
                }
                x_max = cmp::max(x_max, x);
            }
            y_max = cmp::max(y_max, y);
        }

        Ok(Self {
            squares,
            x_max,
            y_max,
        })
    }
}

#[derive(Debug, Snafu)]
enum ParseMapError {
    Square {
        source: ParseSquareError,
        x: usize,
        y: usize,
        c: char,
    },
}

#[derive(Debug, Copy, Clone)]
enum Square {
    VerticalSplit,   // |
    HorizontalSplit, // -
    MirrorDownRight, // \
    MirrorDownLeft,  // /
}

impl Square {
    fn try_from(value: char) -> Result<Option<Self>, ParseSquareError> {
        use Square::*;

        Ok(Some(match value {
            '|' => VerticalSplit,
            '-' => HorizontalSplit,
            '\\' => MirrorDownRight,
            '/' => MirrorDownLeft,
            '.' => return Ok(None),
            _ => return Err(ParseSquareError),
        }))
    }
}

#[derive(Debug, Snafu)]
struct ParseSquareError;

mod square_shorthands {
    pub(super) use super::Square::{
        HorizontalSplit as Hs, MirrorDownLeft as Dl, MirrorDownRight as Dr, VerticalSplit as Vs,
    };
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Direction {
    Up,
    Right,
    Down,
    Left,
}

mod direction_shorthands {
    pub(super) use super::Direction::{Down as D, Left as L, Right as R, Up as U};
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_INPUT_1: &str = include_str!("../example-input-1");

    #[test]
    #[snafu::report]
    fn example_1() -> Result<(), ParseMapError> {
        assert_eq!(46, energized_tiles(EXAMPLE_INPUT_1)?);

        Ok(())
    }
}
