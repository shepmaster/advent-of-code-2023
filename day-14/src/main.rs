use core::fmt;
use snafu::prelude::*;
use std::{
    cmp,
    collections::{btree_map::Entry, BTreeMap, BTreeSet},
};

const INPUT: &str = include_str!("../input");
const CYCLES: usize = 1_000_000_000;

#[snafu::report]
fn main() -> Result<(), Error> {
    let load = total_load(INPUT)?;
    // Part 1: 108641
    println!("{load}");

    let load = total_load_after_spin_cycles(INPUT, CYCLES)?;
    // Part 2: 84328
    println!("{load}");

    Ok(())
}

fn total_load(s: &str) -> Result<usize, Error> {
    let mut board = Board::new(s);

    board.tilt_north();

    Ok(board.total_load())
}

fn total_load_after_spin_cycles(s: &str, n_cycles: usize) -> Result<usize, Error> {
    let mut board = Board::new(s);

    let mut last_states = BTreeMap::new();

    for cycle in 0..n_cycles {
        board.spin_cycle();

        match last_states.entry(board.balls.clone()) {
            Entry::Vacant(v) => v.insert(cycle),
            Entry::Occupied(o) => {
                let &prev_cycle = o.get();

                let cycle_len = cycle - prev_cycle;

                let cycles_remaining = n_cycles - cycle;
                // Skip over all the repeated work
                let cycles_remaining = cycles_remaining % cycle_len;
                // We've already done this cycle, don't count it again.
                let cycles_remaining = cycles_remaining - 1;

                for _ in 0..cycles_remaining {
                    board.spin_cycle();
                }

                break;
            }
        };
    }

    Ok(board.total_load())
}

type Map = BTreeSet<(usize, usize)>;

#[derive(Debug, PartialEq)]
struct Board {
    cubes: Map,
    balls: Map,
    x_max: usize,
    y_max: usize,
}

impl Board {
    fn new(s: &str) -> Self {
        let mut cubes = BTreeSet::new();
        let mut balls = BTreeSet::new();

        let mut x_max = 0;
        let mut y_max = 0;

        for (y, line) in s.lines().enumerate() {
            for (x, c) in line.chars().enumerate() {
                let set = match c {
                    'O' => &mut balls,
                    '#' => &mut cubes,
                    _ => continue,
                };
                set.insert((x, y));

                x_max = cmp::max(x_max, x);
            }

            y_max = cmp::max(y_max, y);
        }

        Self {
            cubes,
            balls,
            x_max,
            y_max,
        }
    }

    fn total_load(&self) -> usize {
        let Self { balls, y_max, .. } = self;
        // Adding one as we are counting from the *edge* of
        // the platform
        balls.iter().map(|&(_, y)| y_max - y + 1).sum()
    }

    fn spin_cycle(&mut self) {
        self.tilt_north();
        self.tilt_west();
        self.tilt_south();
        self.tilt_east();
    }

    /// Walk down each column. If we see a ball, move it to the
    /// furthest spot avaiable. If we see a cube, update where the
    /// furthest spot would be.
    fn tilt_north(&mut self) {
        let Self {
            ref cubes,
            x_max,
            y_max,
            ..
        } = *self;

        for x in 0..=x_max {
            let mut dest = 0;

            for y in 0..=y_max {
                let coord = (x, y);

                if self.balls.remove(&coord) {
                    self.balls.insert((x, dest));
                    dest += 1;
                } else if cubes.contains(&coord) {
                    // A cube is at this spot, so the balls will slide to
                    // the *next* spot
                    dest = y + 1;
                }
            }
        }
    }

    fn tilt_west(&mut self) {
        let Self {
            ref cubes,
            x_max,
            y_max,
            ..
        } = *self;

        for y in 0..=y_max {
            let mut dest = 0;

            for x in 0..=x_max {
                let coord = (x, y);

                if self.balls.remove(&coord) {
                    self.balls.insert((dest, y));
                    dest += 1;
                } else if cubes.contains(&coord) {
                    // A cube is at this spot, so the balls will slide to
                    // the *next* spot
                    dest = x + 1;
                }
            }
        }
    }

    fn tilt_south(&mut self) {
        let Self {
            ref cubes,
            x_max,
            y_max,
            ..
        } = *self;

        for x in 0..=x_max {
            let mut dest = y_max;

            for y in (0..=y_max).rev() {
                let coord = (x, y);

                if self.balls.remove(&coord) {
                    self.balls.insert((x, dest));
                    dest = dest.saturating_sub(1);
                } else if cubes.contains(&coord) {
                    dest = y.saturating_sub(1); // can be zero as we will exit the loop anyway
                }
            }
        }
    }

    fn tilt_east(&mut self) {
        let Self {
            ref cubes,
            x_max,
            y_max,
            ..
        } = *self;

        for y in 0..=y_max {
            let mut dest = x_max;

            for x in (0..=x_max).rev() {
                let coord = (x, y);

                if self.balls.remove(&coord) {
                    self.balls.insert((dest, y));
                    dest = dest.saturating_sub(1);
                } else if cubes.contains(&coord) {
                    dest = x.saturating_sub(1); // can be zero as we will exit the loop anyway
                }
            }
        }
    }

    #[allow(dead_code)]
    fn assert_consistent(&self) {
        let Self {
            ref cubes,
            ref balls,
            x_max,
            y_max,
        } = *self;

        assert_eq!(0, cubes.intersection(balls).take(1).count());

        let x_range = 0..=x_max;
        let y_range = 0..=y_max;

        for map in [cubes, balls] {
            for (x, y) in map {
                assert!(x_range.contains(x));
                assert!(y_range.contains(y));
            }
        }
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            ref cubes,
            ref balls,
            x_max,
            y_max,
        } = *self;
        for y in 0..=y_max {
            for x in 0..=x_max {
                let coord = (x, y);
                let c = if cubes.contains(&coord) {
                    '#'
                } else if balls.contains(&coord) {
                    'O'
                } else {
                    '.'
                };

                c.fmt(f)?;
            }
            "\n".fmt(f)?;
        }
        Ok(())
    }
}

#[derive(Debug, Snafu)]
enum Error {}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_INPUT_1: &str = include_str!("../example-input-1");
    const EXAMPLE_INPUT_1_CYCLE_1: &str = include_str!("../example-input-1-cycle-1");
    const EXAMPLE_INPUT_1_CYCLE_2: &str = include_str!("../example-input-1-cycle-2");
    const EXAMPLE_INPUT_1_CYCLE_3: &str = include_str!("../example-input-1-cycle-3");

    #[test]
    #[snafu::report]
    fn example_1() -> Result<(), Error> {
        assert_eq!(136, total_load(EXAMPLE_INPUT_1)?);

        Ok(())
    }

    #[test]
    #[snafu::report]
    fn example_2() -> Result<(), Error> {
        assert_eq!(64, total_load_after_spin_cycles(EXAMPLE_INPUT_1, CYCLES)?);

        Ok(())
    }

    #[test]
    #[snafu::report]
    fn example_2_spin_cycles() -> Result<(), Error> {
        let mut b = Board::new(EXAMPLE_INPUT_1);

        b.spin_cycle();
        let b1 = Board::new(EXAMPLE_INPUT_1_CYCLE_1);
        assert_eq!(b1, b);

        b.spin_cycle();
        let b2 = Board::new(EXAMPLE_INPUT_1_CYCLE_2);
        assert_eq!(b2, b);

        b.spin_cycle();
        let b3 = Board::new(EXAMPLE_INPUT_1_CYCLE_3);
        assert_eq!(b3, b);

        Ok(())
    }
}
