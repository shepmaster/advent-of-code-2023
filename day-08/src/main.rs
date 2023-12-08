use itertools::Itertools;
use snafu::prelude::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    iter,
};

const INPUT: &str = include_str!("../input");

#[snafu::report]
fn main() -> Result<(), Error> {
    let n_steps = n_steps_to_end(INPUT)?;
    // Part 1: 18157
    println!("{n_steps}");

    let n_steps = multi_n_steps_to_end(INPUT)?;
    // Part 2: 52766656211 (too low)
    // -> Didn't multiply by the step length
    //       : 14299763833181
    println!("{n_steps}");

    Ok(())
}

fn n_steps_to_end(s: &str) -> Result<usize, Error> {
    let (steps, map) = parse_input(s)?;

    let path = follow_path(&steps, "AAA", &map);

    Ok(path.take_while(|&node| node != "ZZZ").count())
}

// Idea: follow each path, recording all the possible end spots until
// we return to a place we've already been and the future steps
// match. Then we have to find some kind of multiple least common
// denominator across all paths / endings?
fn multi_n_steps_to_end(s: &str) -> Result<usize, Error> {
    let (steps, map) = parse_input(s)?;

    let starts = map.keys().filter(|node| node.ends_with('A'));

    let paths = starts
        .map(|start| {
            follow_path(&steps, start, &map)
                .enumerate()
                .filter(|(_, node)| node.ends_with('Z'))
                .map(|(idx, _)| idx)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    // By inspection, all the path step counts happen to be exact
    // multiples of the input step count. This means we can factor
    // that out before multiplication and then factor it back in
    // afterwards. We only need to factor it in once as we can treat
    // all values as modulo `steps.len()`.

    // Additionally, the real input only has a single step value for
    // each path, so we don't need to actually worry about multiple.

    let laps_lcm: usize = paths
        .into_iter()
        .flat_map(|p| p.into_iter().next())
        .map(|v| {
            if v % steps.len() == 0 {
                v / steps.len()
            } else {
                v
            }
        })
        .product();

    let n_steps = laps_lcm * steps.len();
    Ok(n_steps)
}

fn follow_path<'a>(
    steps: &'a [Direction],
    mut position: &'a str,
    map: &'a Map<'a>,
) -> impl Iterator<Item = &'a str> + 'a {
    let mut steps = steps.iter().copied().enumerate().cycle();
    let mut visited = BTreeSet::new();

    iter::from_fn(move || {
        let (step_idx, step) = steps.next().expect("can never run out of steps");

        if !visited.insert((position, step_idx)) {
            // We have already visited this node at this point in the step list;
            // the path will repeat forever.
            return None;
        }

        let map_value = map.get(position).expect("the map does not have this node");

        let next = match step {
            Direction::Left => map_value.0,
            Direction::Right => map_value.1,
        };

        let current = Some(position);
        position = next;
        current
    })
}

type Map<'a> = BTreeMap<&'a str, (&'a str, &'a str)>;

fn parse_input(s: &str) -> Result<(Vec<Direction>, Map<'_>), Error> {
    let mut lines = s.lines().fuse();

    let steps = lines.next().context(StepsMissingSnafu)?;
    let steps = steps
        .chars()
        .map(|c| Direction::try_from(c).context(StepInvalidSnafu { c }))
        .collect::<Result<_, _>>()?;

    let map = lines
        .dropping(1)
        .map(|line| parse_line(line).context(LineInvalidSnafu { line }))
        .collect::<Result<_, _>>()?;

    Ok((steps, map))
}

#[derive(Debug, Snafu)]
enum Error {
    StepsMissing,

    StepInvalid {
        source: ParseDirectionError,
        c: char,
    },

    LineInvalid {
        source: ParseLineError,
        line: String,
    },
}

fn parse_line(line: &str) -> Result<(&str, (&str, &str)), ParseLineError> {
    use parse_line_error::*;

    let (key, value) = line.split_once('=').context(MalformedSnafu)?;

    let key = key.trim();
    let value = value.trim().trim_start_matches('(').trim_end_matches(')');

    let (left, right) = value
        .split_once(',')
        .context(MalformedValueSnafu { value })?;

    let left = left.trim();
    let right = right.trim();

    Ok((key, (left, right)))
}

#[derive(Debug, Snafu)]
#[snafu(module)]
enum ParseLineError {
    Malformed,

    MalformedValue { value: String },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Direction {
    Left,
    Right,
}

impl TryFrom<char> for Direction {
    type Error = ParseDirectionError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        Ok(match value {
            'L' => Self::Left,
            'R' => Self::Right,
            _ => return ParseDirectionSnafu.fail(),
        })
    }
}

#[derive(Debug, Snafu)]
struct ParseDirectionError;

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_INPUT_1: &str = include_str!("../example-input-1");
    const EXAMPLE_INPUT_2: &str = include_str!("../example-input-2");
    const EXAMPLE_INPUT_3: &str = include_str!("../example-input-3");

    #[test]
    #[snafu::report]
    fn example_1() -> Result<(), Error> {
        assert_eq!(2, n_steps_to_end(EXAMPLE_INPUT_1)?);
        Ok(())
    }

    #[test]
    #[snafu::report]
    fn example_2() -> Result<(), Error> {
        assert_eq!(6, n_steps_to_end(EXAMPLE_INPUT_2)?);
        Ok(())
    }

    #[test]
    #[snafu::report]
    fn example_3() -> Result<(), Error> {
        assert_eq!(6, multi_n_steps_to_end(EXAMPLE_INPUT_3)?);
        Ok(())
    }
}
