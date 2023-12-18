use snafu::prelude::*;
use std::{
    cmp,
    collections::{btree_map::Entry, BTreeMap, BinaryHeap},
    str::FromStr,
};

const INPUT: &str = include_str!("../input");

fn main() -> Result<(), Error> {
    let heat_loss = minimal_heat_loss(INPUT)?;
    // Part 1: 758
    println!("{heat_loss}");

    Ok(())
}

fn minimal_heat_loss(s: &str) -> Result<u32, Error> {
    use direction_shorthand::*;

    let grid = s.parse::<Grid>()?;

    #[derive(Debug)]
    struct Step {
        coord: Coord,
        dir: Direction,
        steps: usize,
        cost: u32,
    }

    impl PartialEq for Step {
        fn eq(&self, other: &Self) -> bool {
            self.cost == other.cost
        }
    }

    impl Eq for Step {}

    impl PartialOrd for Step {
        fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Ord for Step {
        fn cmp(&self, other: &Self) -> cmp::Ordering {
            self.cost.cmp(&other.cost).reverse()
        }
    }

    let start = grid.start();
    let end = grid.end();

    let mut queue = BinaryHeap::from_iter([
        Step {
            coord: start,
            dir: R,
            steps: 0,
            cost: 0,
        },
        Step {
            coord: start,
            dir: D,
            steps: 0,
            cost: 0,
        },
    ]);

    let mut visited = BTreeMap::new();

    while let Some(Step {
        coord,
        dir,
        steps,
        cost,
    }) = queue.pop()
    {
        if coord == end {
            return Ok(cost);
        }

        let visited = visited.entry(coord).or_insert_with(BTreeMap::new);
        let visited = visited.entry(dir).or_insert_with(BTreeMap::new);
        match visited.entry(steps) {
            Entry::Vacant(e) => {
                e.insert(cost);
            }
            Entry::Occupied(mut e) => {
                let old_cost = *e.get();
                if cost < old_cost {
                    e.insert(cost);
                } else {
                    continue;
                }
            }
        }

        let left = dir.left_turn();
        if let Some(coord) = grid.step(coord, left) {
            let cost = cost + grid.map[&coord];

            queue.push(Step {
                coord,
                dir: left,
                steps: 1,
                cost,
            });
        }

        let right = dir.right_turn();
        if let Some(coord) = grid.step(coord, right) {
            let cost = cost + grid.map[&coord];

            queue.push(Step {
                coord,
                dir: right,
                steps: 1,
                cost,
            });
        }

        if steps < 3 {
            if let Some(coord) = grid.step(coord, dir) {
                let cost = cost + grid.map[&coord];

                queue.push(Step {
                    coord,
                    dir,
                    steps: steps + 1,
                    cost,
                });
            }
        }
    }

    unreachable!("Grid had no solution");
}

type Coord = (usize, usize);

struct Grid {
    map: BTreeMap<Coord, u32>,
    x_max: usize,
    y_max: usize,
}

impl Grid {
    fn start(&self) -> Coord {
        (0, 0)
    }

    fn end(&self) -> Coord {
        let Self { x_max, y_max, .. } = *self;
        (x_max, y_max)
    }

    fn step(&self, coord: Coord, dir: Direction) -> Option<Coord> {
        use direction_shorthand::*;

        let Self { x_max, y_max, .. } = *self;

        let (x, y) = coord;

        let checked_inc = |v: usize, max: usize| {
            let v = v.checked_add(1)?;
            if v > max {
                None
            } else {
                Some(v)
            }
        };

        match dir {
            U => Some((x, y.checked_sub(1)?)),
            R => Some((checked_inc(x, x_max)?, y)),
            D => Some((x, checked_inc(y, y_max)?)),
            L => Some((x.checked_sub(1)?, y)),
        }
    }
}

impl FromStr for Grid {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut map = BTreeMap::new();

        for (y, line) in s.lines().enumerate() {
            for (x, c) in line.chars().enumerate() {
                let d = c.to_digit(10).context(DigitSnafu { x, y, c })?;
                map.insert((x, y), d);
            }
        }

        let &(x_max, y_max) = map.keys().last().context(EmptySnafu)?;

        Ok(Self { map, x_max, y_max })
    }
}

#[derive(Debug, Snafu)]
enum Error {
    Digit { x: usize, y: usize, c: char },

    Empty,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Direction {
    fn left_turn(self) -> Self {
        use direction_shorthand::*;

        match self {
            R => U,
            U => L,
            L => D,
            D => R,
        }
    }

    fn right_turn(self) -> Self {
        use direction_shorthand::*;

        match self {
            R => D,
            U => R,
            L => U,
            D => L,
        }
    }
}

mod direction_shorthand {
    pub(super) use super::Direction::{Down as D, Left as L, Right as R, Up as U};
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_INPUT_1: &str = include_str!("../example-input-1");

    #[test]
    #[snafu::report]
    fn example_1() -> Result<(), Error> {
        assert_eq!(102, minimal_heat_loss(EXAMPLE_INPUT_1)?);

        Ok(())
    }
}
