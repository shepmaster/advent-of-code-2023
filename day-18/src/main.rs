use core::fmt;
use snafu::prelude::*;
use std::{cmp, collections::BTreeSet, str::FromStr};

const INPUT: &str = include_str!("../input");

#[snafu::report]
fn main() -> Result<(), Error> {
    let volume = lava_volume(INPUT)?;
    // Part 1: 60557
    // -> Too low (wasn't properly handing corners)
    //         61865
    println!("{volume}");

    Ok(())
}

fn lava_volume(s: &str) -> Result<usize, Error> {
    let steps = s
        .lines()
        .map(|step| Step::try_from(step).context(StepSnafu { step }));

    let map = itertools::process_results(steps, |steps| build_outline(steps)).unwrap();

    let bounds = Bounds::find(&map).context(BoundsSnafu)?;

    let next = fill_in_outline(&map, bounds);

    Ok(next.len())
}

#[derive(Debug, Snafu)]
enum Error {
    Step {
        source: ParseStepError,
        step: String,
    },

    Bounds,
}

fn fill_in_outline(map: &Map, bounds: Bounds) -> BTreeSet<(i32, i32)> {
    #[derive(Debug, Copy, Clone)]
    enum Inside {
        Yes,
        No,
    }

    impl Inside {
        fn toggle(&self) -> Inside {
            match self {
                Inside::Yes => Inside::No,
                Inside::No => Inside::Yes,
            }
        }
    }

    #[derive(Debug, Copy, Clone)]
    enum State {
        Known(Inside),
        Pending(Wall, Inside),
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    enum Wall {
        UpperBend,
        Flat,
        LowerBend,
    }

    let Bounds {
        x_min,
        x_max,
        y_min,
        y_max,
    } = bounds;

    let mut next = map.clone();

    // eprintln!("{}", MapView(&next, bounds));

    for y in y_min..=y_max {
        let mut state = State::Known(Inside::No);

        for x in x_min..=x_max {
            let us = map.contains(&(x, y));

            if us {
                let above = map.contains(&(x, y - 1));
                let below = map.contains(&(x, y + 1));

                let w = match (above, below) {
                    (true, true) | (false, false) => Wall::Flat,
                    (true, false) => Wall::UpperBend,
                    (false, true) => Wall::LowerBend,
                };

                // eprint!("{state:?} => {w:?} => ");

                state = match (state, w) {
                    (State::Known(i), Wall::Flat) => State::Known(i.toggle()),
                    (State::Known(i), w) => State::Pending(w, i),

                    (State::Pending(..), Wall::Flat) => state,
                    (State::Pending(ow, i), nw) => {
                        if ow == nw {
                            State::Known(i)
                        } else {
                            State::Known(i.toggle())
                        }
                    }
                };

                // eprintln!("{state:?}");
            }

            if let State::Known(Inside::Yes) = state {
                next.insert((x, y));
            }
        }

        // eprintln!("QQQ\n");
        // eprintln!("{}", MapView(&next, bounds));
    }

    next
}

struct MapView<'a>(&'a Map, Bounds);

impl fmt::Display for MapView<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Bounds {
            x_min,
            x_max,
            y_min,
            y_max,
        } = self.1;

        for y in y_min..=y_max {
            for x in x_min..=x_max {
                let c = if self.0.contains(&(x, y)) { '#' } else { '.' };
                c.fmt(f)?;
            }
            "\n".fmt(f)?
        }

        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
struct Bounds {
    x_min: i32,
    x_max: i32,
    y_min: i32,
    y_max: i32,
}

impl Bounds {
    fn initial(x: i32, y: i32) -> Self {
        Self {
            x_min: x,
            x_max: x,
            y_min: y,
            y_max: y,
        }
    }

    fn find(map: &Map) -> Option<Self> {
        let mut b = None;

        for &(x, y) in map {
            let Bounds {
                x_min,
                x_max,
                y_min,
                y_max,
            } = b.get_or_insert(Bounds::initial(x, y));

            *x_min = cmp::min(x, *x_min);
            *x_max = cmp::max(x, *x_max);
            *y_min = cmp::min(y, *y_min);
            *y_max = cmp::max(y, *y_max);
        }

        b
    }
}

type Coord = (i32, i32);

type Map = BTreeSet<Coord>;

fn build_outline<'a>(steps: impl Iterator<Item = Step<'a>>) -> Map {
    use direction_shorthand::*;

    let mut x = 0;
    let mut y = 0;

    let mut map: Map = BTreeSet::new();

    for Step { dir, count, .. } in steps {
        match dir {
            U => {
                let next_y = y - count;
                for c in (next_y..=y).map(|y| (x, y)) {
                    map.insert(c);
                }
                y = next_y;
            }

            R => {
                let next_x = x + count;
                for c in (x..=next_x).map(|x| (x, y)) {
                    map.insert(c);
                }
                x = next_x;
            }

            D => {
                let next_y = y + count;
                for c in (y..=next_y).map(|y| (x, y)) {
                    map.insert(c);
                }
                y = next_y;
            }

            L => {
                let next_x = x - count;
                for c in (next_x..=x).map(|x| (x, y)) {
                    map.insert(c);
                }
                x = next_x;
            }
        }
    }

    map
}

struct Step<'a> {
    dir: Direction,
    count: i32,
    #[allow(dead_code)]
    color: &'a str,
}

impl<'a> TryFrom<&'a str> for Step<'a> {
    type Error = ParseStepError;

    fn try_from(l: &'a str) -> Result<Self, Self::Error> {
        use parse_step_error::*;

        let (dir, l) = l.split_once(' ').context(MalformedDirectionSnafu)?;
        let (count, l) = l.split_once(' ').context(MalformedCountSnafu)?;
        let color = l.trim_matches(&['#', '(', ')']);

        let dir = dir
            .parse::<Direction>()
            .context(InvalidDirectionSnafu { dir })?;
        let count = count.parse::<i32>().context(InvalidCountSnafu { count })?;

        Ok(Self { dir, count, color })
    }
}

#[derive(Debug, Snafu)]
#[snafu(module)]
enum ParseStepError {
    MalformedDirection,
    MalformedCount,
    InvalidDirection {
        source: ParseDirectionError,
        dir: String,
    },
    InvalidCount {
        source: std::num::ParseIntError,
        count: String,
    },
}

#[derive(Debug, Copy, Clone)]
enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl FromStr for Direction {
    type Err = ParseDirectionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use direction_shorthand::*;

        Ok(match s {
            "U" => U,
            "R" => R,
            "D" => D,
            "L" => L,
            _ => return ParseDirectionSnafu.fail(),
        })
    }
}

#[derive(Debug, Snafu)]
struct ParseDirectionError;

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
        assert_eq!(62, lava_volume(EXAMPLE_INPUT_1)?);

        Ok(())
    }
}
