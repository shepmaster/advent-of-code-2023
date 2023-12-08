use itertools::Itertools;
use snafu::prelude::*;
use std::{collections::BTreeMap, iter};

const INPUT: &str = include_str!("../input");

#[snafu::report]
fn main() -> Result<(), Error> {
    let n_steps = n_steps_to_end(INPUT)?;
    println!("{n_steps}");

    Ok(())
}

fn n_steps_to_end(s: &str) -> Result<usize, Error> {
    let mut lines = s.lines().fuse();

    let steps = lines.next().context(StepsMissingSnafu)?;
    let steps = steps
        .chars()
        .map(|c| Direction::try_from(c).context(StepInvalidSnafu { c }))
        .collect::<Result<Vec<_>, _>>()?;

    let map = lines
        .dropping(1)
        .map(|line| parse_line(line).context(LineInvalidSnafu { line }))
        .collect::<Result<BTreeMap<_, _>, _>>()?;

    let mut position = "AAA";

    let mut steps = steps.iter().copied().cycle();

    let path = iter::from_fn(|| {
        let current = Some(position);

        let map_value = map.get(position)?;

        let next = match steps.next()? {
            Direction::Left => map_value.0,
            Direction::Right => map_value.1,
        };

        position = next;
        current
    });

    Ok(path.take_while(|&node| node != "ZZZ").count())
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

#[derive(Debug, Copy, Clone)]
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
}
