use itertools::Itertools;
use snafu::prelude::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    str::FromStr,
};

const INPUT: &str = include_str!("../input");

#[snafu::report]
fn main() -> Result<(), Error> {
    let volume = lava_volume(INPUT)?;
    // Part 1: 60557
    // -> Too low (wasn't properly handing corners)
    //         61865
    println!("{volume}");

    let volume = lava_volume_fixed(INPUT)?;
    // Part 2: 40343619199142
    println!("{volume}");

    Ok(())
}

fn lava_volume(s: &str) -> Result<i64, Error> {
    let steps: Vec<_> = s
        .lines()
        .map(|step| step.try_into().context(StepSnafu { step }))
        .collect::<Result<_, _>>()?;

    Ok(points_of_interest_volume(steps))
}

fn lava_volume_fixed(s: &str) -> Result<i64, Error> {
    let steps: Vec<_> = s
        .lines()
        .map(|step| {
            step.try_into()
                .context(StepSnafu { step })
                .map(|s: Step<'_>| s.fixup())
        })
        .collect::<Result<_, _>>()?;

    Ok(points_of_interest_volume(steps))
}

#[derive(Debug, Copy, Clone)]
enum Corner {
    Lower,
    Upper,
}

// Idea: Find all corners in the graph. That point, and the 8
// immediate neighbors, are the only relevant points in the entire
// graph.
//
// Take those unique X and Y coordinates and build a number of
// rectanges that cover the entire graph.
//
// For each of those rectangles, pick a representative point inside
// and check to see if it is a corner or a line. If so, they are
// filled. Otherwise, cast a ray from the left, counting how many
// vertical walls are intersected. An odd number means we are inside,
// even means outside. We also have to track the number and kind of
// corners to know if the twist mans we are inside or not.
fn points_of_interest_volume(steps: Vec<Step<'_>>) -> i64 {
    use direction_shorthand::*;
    use Corner::*;

    let prev_dirs = steps.iter().map(|s| s.dir).cycle().skip(steps.len() - 1);

    let mut x = 0;
    let mut y = 0;

    let mut linear = 0;

    let mut corners = BTreeMap::new(); // coord -> kind of corner
    let mut v_lines = BTreeMap::new(); // x -> y0..=y1
    let mut h_lines = BTreeMap::new(); // y -> x0..=x1

    for (step, prev_dir) in steps.iter().zip(prev_dirs) {
        let Step { dir, count, .. } = step;

        // Count how long the outline is to check our math.
        linear += count;

        // Add corners.
        let corner = match (prev_dir, dir) {
            (U, R) => Lower,
            (U, L) => Lower,

            (R, U) => Upper,
            (R, D) => Lower,

            (D, R) => Upper,
            (D, L) => Upper,

            (L, U) => Upper,
            (L, D) => Lower,

            (U, U) | (R, R) | (D, D) | (L, L) => unreachable!("Lines cannot continue"),

            (U, D) | (R, L) | (D, U) | (L, R) => unreachable!("Lines cannot reverse"),
        };
        corners.insert((x, y), corner);

        let start_c = (x, y);

        // Move our coordinate
        match dir {
            U => y -= count,
            R => x += count,
            D => y += count,
            L => x -= count,
        }

        let end_c = (x, y);

        // Take two endpoints of a line and return a range
        // representing the range of the line exclusing the corners.
        let inner_line = |v0, v1| {
            let mut vs = [v0, v1];
            vs.sort();
            let [v0, v1] = vs;

            (v0 + 1)..=(v1 - 1)
        };

        match dir {
            U | D => {
                v_lines
                    .entry(x)
                    .or_insert_with(Vec::new)
                    .push(inner_line(start_c.1, end_c.1));
            }
            L | R => {
                h_lines
                    .entry(y)
                    .or_insert_with(Vec::new)
                    .push(inner_line(start_c.0, end_c.0));
            }
        }
    }

    // The locations of the corners and all 8 immediate neighbors.
    const DELTAS: [i64; 3] = [-1, 0, -1];

    let pois = corners.keys().flat_map(|&(x, y)| {
        DELTAS
            .into_iter()
            .cartesian_product(DELTAS)
            .map(move |(dx, dy)| (x + dx, y + dy))
    });

    let (xs, ys): (BTreeSet<_>, BTreeSet<_>) = pois.unzip();

    // Pair up consecutive points to create the top-left and
    // bottom-right points of each square.
    let xs = xs.iter().copied().tuple_windows::<(_, _)>();
    let ys = ys.iter().copied().tuple_windows::<(_, _)>();
    let squares = xs.cartesian_product(ys);

    let mut outline_filled = 0;
    let mut inside_filled = 0;

    for ((x0, x1), (y0, y1)) in squares {
        let w = x1 - x0;
        let h = y1 - y0;
        let a = w * h;

        // let top_left = (x0, y0);
        let bottom_right = (x1, y1);

        // The top-left point doesn't work, but the bottom-right does?
        // A fencepost problem?
        let rep = bottom_right;

        let is_on_corner = || corners.contains_key(&rep);

        let is_on_v_line = || {
            v_lines
                .iter()
                .any(|(&x, ys)| x == rep.0 && ys.iter().any(|ys| ys.contains(&rep.1)))
        };

        let is_on_h_line = || {
            h_lines
                .iter()
                .any(|(&y, xs)| y == rep.1 && xs.iter().any(|xs| xs.contains(&rep.0)))
        };

        if is_on_corner() {
            assert_eq!(1, a, "a corner should only ever be exactly one square");
            outline_filled += a;
        } else if is_on_v_line() || is_on_h_line() {
            outline_filled += a;
        } else {
            use Intersection::*;

            #[derive(Debug, Copy, Clone)]
            enum Intersection {
                Corn(Corner),
                Wind,
            }

            // Find all corners (and their kinds) and walls that occur
            // at this Y coordinate and before this X
            // coordinate. Those are the only intersecting points that
            // matter if the element is inside or not.

            let c = corners
                .iter()
                .filter(|(&(x, y), _)| y == rep.1 && x < rep.0)
                .map(|((x, _), &c)| (x, Corn(c)));

            let w = v_lines
                .range(..=rep.0)
                .filter(|(_, ys)| ys.iter().any(|ys| ys.contains(&rep.1)))
                .map(|(x, _)| (x, Wind));

            let mut intersections = c.chain(w).collect::<BTreeMap<_, _>>();

            // Walk through the intersections two elements at a
            // time. If we see a wall, toggle `inside` and put the
            // second element back. If we see two corners that match,
            // the shape doubled back and we don't change `inside`. If
            // they are different, the shape kept going so toggle
            // `inside`.
            let mut inside = false;

            loop {
                match (intersections.pop_first(), intersections.pop_first()) {
                    (None, None) => break,

                    (Some((_, Wind)), v) => {
                        inside = !inside;
                        intersections.extend(v);
                    }

                    (Some((_, Corn(Upper))), Some((_, Corn(Upper))))
                    | (Some((_, Corn(Lower))), Some((_, Corn(Lower)))) => {
                        // no-op
                    }

                    (Some((_, Corn(Upper))), Some((_, Corn(Lower))))
                    | (Some((_, Corn(Lower))), Some((_, Corn(Upper)))) => {
                        inside = !inside;
                    }

                    o => unreachable!("Didn't handle {o:?}"),
                }
            }

            if inside {
                inside_filled += a;
            }
        }
    }

    assert_eq!(linear, outline_filled);

    outline_filled + inside_filled
}

#[derive(Debug, Snafu)]
enum Error {
    Step {
        source: ParseStepError,
        step: String,
    },
}

#[derive(Debug)]
struct Step<'a> {
    dir: Direction,
    count: i64,
    color: &'a str,
}

impl Step<'_> {
    fn fixup(self) -> Step<'static> {
        use direction_shorthand::*;

        let (h, t) = self.color.split_at(5);

        let count = i64::from_str_radix(h, 16).unwrap();
        let dir = match t {
            "0" => R,
            "1" => D,
            "2" => L,
            "3" => U,
            _ => panic!(),
        };

        Step {
            dir,
            count,
            color: "",
        }
    }
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
        let count = count.parse().context(InvalidCountSnafu { count })?;

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

    #[test]
    #[snafu::report]
    fn example_2() -> Result<(), Error> {
        assert_eq!(952408144115, lava_volume_fixed(EXAMPLE_INPUT_1)?);

        Ok(())
    }
}
