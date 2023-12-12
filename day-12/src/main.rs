use core::fmt;
use itertools::Itertools;
use snafu::prelude::*;
use std::collections::BTreeMap;

const INPUT: &str = include_str!("../input");

#[snafu::report]
fn main() -> Result<(), Error> {
    let arrangements = sum_of_possible_arrangements(INPUT)?;
    // Part 1: 7916
    println!("{arrangements}");

    let arrangements = sum_of_unfolded_possible_arrangements(INPUT)?;
    // Part 2: 37366887898686
    println!("{arrangements}");

    Ok(())
}

fn sum_of_possible_arrangements(s: &str) -> Result<usize, Error> {
    lines(s).map(|line| Ok(line?.possible_arrangements())).sum()
}

fn sum_of_unfolded_possible_arrangements(s: &str) -> Result<usize, Error> {
    lines(s)
        .map(|line| {
            let mut line = line?;
            line.unfold();
            Ok(line.possible_arrangements())
        })
        .sum()
}

fn lines(s: &str) -> impl Iterator<Item = Result<Line, Error>> + '_ {
    s.lines()
        .map(|line| Line::try_from(line).context(LineSnafu { line }))
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

impl Line {
    fn unfold(&mut self) {
        use std::mem;

        const UNFOLD_COUNT: usize = 5;

        let conditions = mem::take(&mut self.conditions);
        let a = itertools::repeat_n(conditions, UNFOLD_COUNT);
        let a = Itertools::intersperse(a, vec![Condition::Unknown]);
        self.conditions = a.flatten().collect();

        let group_sizes = mem::take(&mut self.group_sizes);
        let a = itertools::repeat_n(group_sizes, UNFOLD_COUNT);
        self.group_sizes = a.flatten().collect();
    }

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

        type Cache<'a> = BTreeMap<(&'a [Condition], &'a [usize]), usize>;

        fn core<'a>(
            cache: &mut Cache<'a>,
            conditions: &'a [Condition],
            group_sizes: &'a [usize],
        ) -> usize {
            let Some((condition, next_conditions)) = conditions.split_first() else {
                // No more conditions; only a success when the groups are empty
                if group_sizes.is_empty() {
                    return 1;
                } else {
                    return 0;
                }
            };

            let Some((&group_size, next_group_sizes)) = group_sizes.split_first() else {
                // No more groups; only a success when the remaining conditions are operational
                if conditions.iter().all(|c| c.acts_as_operational()) {
                    return 1;
                } else {
                    return 0;
                }
            };

            if let Some(&successes) = cache.get(&(conditions, group_sizes)) {
                return successes;
            }

            fn treat_as_operational<'a>(
                next_conditions: &'a [Condition],
                group_sizes: &'a [usize],
            ) -> (&'a [Condition], &'a [usize]) {
                (next_conditions, group_sizes)
            }

            fn treat_as_damaged<'a>(
                conditions: &'a [Condition],
                group_size: usize,
                next_group_sizes: &'a [usize],
            ) -> Option<(&'a [Condition], &'a [usize])> {
                if conditions.len() < group_size {
                    return None;
                };

                let (head, body) = conditions.split_at(group_size);

                // We need to start with N damaged conditions
                if !head.iter().all(|c| c.acts_as_damaged()) {
                    return None;
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
                    return None;
                }

                Some((tail, next_group_sizes))
            }

            // NEXT: grab the value, recursively call back into ourselves, cache the result
            // also check the cache at the beginning of function.
            let n_successes = match condition {
                Operational => {
                    let (cs, gs) = treat_as_operational(next_conditions, group_sizes);
                    core(cache, cs, gs)
                }

                Damaged => match treat_as_damaged(conditions, group_size, next_group_sizes) {
                    Some((cs, gs)) => core(cache, cs, gs),
                    None => 0,
                },

                Unknown => {
                    let (cs, gs) = treat_as_operational(next_conditions, group_sizes);
                    let operational_successes = core(cache, cs, gs);

                    let damaged_successes =
                        match treat_as_damaged(conditions, group_size, next_group_sizes) {
                            Some((cs, gs)) => core(cache, cs, gs),
                            None => 0,
                        };

                    operational_successes + damaged_successes
                }
            };

            cache.insert((conditions, group_sizes), n_successes);
            n_successes
        }

        let mut cache = BTreeMap::new();
        core(&mut cache, &self.conditions, &self.group_sizes)
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
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

    #[test]
    #[snafu::report]
    fn example_2() -> Result<(), Error> {
        assert_eq!(
            525152,
            sum_of_unfolded_possible_arrangements(EXAMPLE_INPUT_1)?
        );

        Ok(())
    }
}
