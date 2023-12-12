use core::fmt;
use snafu::prelude::*;

const INPUT: &str = include_str!("../input");

#[snafu::report]
fn main() -> Result<(), Error> {
    let arrangements = sum_of_possible_arrangements(INPUT)?;
    // Part 1: 7916
    println!("{arrangements}");

    Ok(())
}

fn sum_of_possible_arrangements(s: &str) -> Result<usize, Error> {
    s.lines()
        .map(|line| {
            let line = Line::try_from(line).context(LineSnafu { line })?;
            Ok(line.possible_arrangements())
        })
        .sum()
}

#[derive(Debug, Snafu)]
enum Error {
    Line {
        source: ParseLineError,
        line: String,
    },
}

#[derive(Debug)]
struct Line {
    conditions: Vec<Condition>,
    group_sizes: Vec<usize>,
}

impl TryFrom<&str> for Line {
    type Error = ParseLineError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use parse_line_error::*;

        let (conditions, group_sizes) = value.split_once(' ').context(MalformedSnafu)?;

        let conditions = conditions
            .chars()
            .map(|c| c.try_into().context(ConditionSnafu { c }))
            .collect::<Result<_, _>>()?;
        let group_sizes = group_sizes
            .split(',')
            .map(|s| s.parse().context(GroupSizeSnafu { s }))
            .collect::<Result<_, _>>()?;

        Ok(Self {
            conditions,
            group_sizes,
        })
    }
}

type Queue<'a> = Vec<(&'a [Condition], &'a [usize])>;

impl Line {
    /// The general idea is to look at the head of `conditions`. If
    /// it's damaged or unknown, try to fit (head of `group_sizes`)
    /// damaged pieces. We then need to leave one operational piece
    /// (or the end of conditions!) and then recur.
    ///
    /// If the head of conditions is operational or unknown, slide
    /// down the conditions by one and recur.
    ///
    /// If the sum of `group_sizes` (plus the space inbetween!) ever
    /// exceeds the length of conditions, then it's an invalid possibility.
    fn possible_arrangements(&self) -> usize {
        use Condition::*;

        let conditions = &self.conditions[..];
        let group_sizes = &self.group_sizes[..];

        let mut success = 0;
        let mut queue: Queue<'_> = vec![(conditions, group_sizes)];

        while let Some((conditions, group_sizes)) = queue.pop() {
            let Some((condition, next_conditions)) = conditions.split_first() else {
                // No more conditions; only a success when the groups are empty
                if group_sizes.is_empty() {
                    success += 1;
                }
                continue;
            };

            let Some((&group_size, next_group_sizes)) = group_sizes.split_first() else {
                // No more groups; only a success when the remaining conditions are operational
                if conditions.iter().all(|c| c.acts_as_operational()) {
                    success += 1;
                }

                continue;
            };

            fn treat_as_operational<'a>(
                queue: &mut Queue<'a>,
                next_conditions: &'a [Condition],
                group_sizes: &'a [usize],
            ) {
                queue.push((next_conditions, group_sizes));
            }

            fn treat_as_damaged<'a>(
                queue: &mut Queue<'a>,
                conditions: &'a [Condition],
                group_size: usize,
                next_group_sizes: &'a [usize],
            ) {
                if conditions.len() < group_size {
                    return;
                };

                let (head, body) = conditions.split_at(group_size);

                // We need to start with N damaged conditions
                if !head.iter().all(|c| c.acts_as_damaged()) {
                    return;
                }

                // If we have a next element, check to see if it
                // counts as operational. If we don't have a next
                // element, that's fine, we are just at the end of
                // the line.
                let (followed_by_operational, tail) = body
                    .split_first()
                    .map(|(b, t)| (b.acts_as_operational(), t))
                    .unwrap_or((true, &[]));

                if !followed_by_operational {
                    return;
                }

                queue.push((tail, next_group_sizes));
            }

            match condition {
                Operational => {
                    treat_as_operational(&mut queue, next_conditions, group_sizes);
                }

                Damaged => {
                    treat_as_damaged(&mut queue, conditions, group_size, next_group_sizes);
                }

                Unknown => {
                    treat_as_operational(&mut queue, next_conditions, group_sizes);
                    treat_as_damaged(&mut queue, conditions, group_size, next_group_sizes);
                }
            }
        }

        success
    }
}

#[derive(Debug, Snafu)]
#[snafu(module)]
enum ParseLineError {
    Malformed,

    Condition {
        source: ParseConditionError,
        c: String,
    },

    GroupSize {
        source: std::num::ParseIntError,
        s: String,
    },
}

struct ConditionView<'a>(&'a [Condition]);

impl fmt::Display for ConditionView<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in self.0 {
            c.fmt(f)?
        }

        Ok(())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Condition {
    Operational,
    Damaged,
    Unknown,
}

impl Condition {
    fn acts_as_damaged(&self) -> bool {
        use Condition::*;

        matches!(self, Damaged | Unknown)
    }

    fn acts_as_operational(&self) -> bool {
        use Condition::*;

        matches!(self, Operational | Unknown)
    }
}

impl TryFrom<char> for Condition {
    type Error = ParseConditionError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        use Condition::*;

        Ok(match value {
            '.' => Operational,
            '#' => Damaged,
            '?' => Unknown,
            _ => return ParseConditionSnafu.fail(),
        })
    }
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Condition::*;

        let c = match self {
            Operational => '.',
            Damaged => '#',
            Unknown => '?',
        };
        c.fmt(f)
    }
}

#[derive(Debug, Snafu)]
struct ParseConditionError;

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_INPUT_1: &str = include_str!("../example-input-1");

    #[test]
    #[snafu::report]
    fn example_1() -> Result<(), Error> {
        assert_eq!(21, sum_of_possible_arrangements(EXAMPLE_INPUT_1)?);

        Ok(())
    }
}
