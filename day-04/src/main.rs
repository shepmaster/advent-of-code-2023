use snafu::prelude::*;
use std::collections::BTreeSet;

const INPUT: &str = include_str!("../input");

type Num = u8;

#[snafu::report]
fn main() -> Result<u64, ParseError> {
    let sum = sum_of_winning_points(INPUT)?;
    // Part 1: 25004
    println!("{sum}");

    Ok(())
}

fn sum_of_winning_points(s: &str) -> Result<u64, ParseError> {
    s.lines()
        .map(|line| -> Result<_, ParseError> {
            let mut parts = line.splitn(3, &[':', '|']);
            let _id = parts.next().context(MissingIdSnafu { line })?;
            let winners = parts.next().context(MissingWinnersSnafu { line })?;
            let numbers = parts.next().context(MissingNumbersSnafu { line })?;

            let winners = winners
                .split_ascii_whitespace()
                .map(|winner| winner.parse::<Num>().context(ParseWinnerSnafu { winner }))
                .collect::<Result<BTreeSet<_>, _>>()
                .context(InvalidWinnerSnafu { line })?;

            let numbers = numbers
                .split_ascii_whitespace()
                .map(|number| number.parse::<Num>().context(ParseNumberSnafu { number }));

            let n_matches = itertools::process_results(numbers, |numbers| {
                numbers.filter(|n| winners.contains(n)).count()
            })
            .context(InvalidNumberSnafu { line })?;

            let points = if n_matches == 0 {
                0
            } else {
                1 << (n_matches - 1)
            };

            Ok(points)
        })
        .sum()
}

#[derive(Debug, Snafu)]
enum ParseError {
    MissingId {
        line: String,
    },

    MissingWinners {
        line: String,
    },

    MissingNumbers {
        line: String,
    },

    InvalidWinner {
        source: ParseWinnerError,
        line: String,
    },

    InvalidNumber {
        source: ParseNumberError,
        line: String,
    },
}

#[derive(Debug, Snafu)]
struct ParseWinnerError {
    source: std::num::ParseIntError,
    winner: String,
}

#[derive(Debug, Snafu)]
struct ParseNumberError {
    source: std::num::ParseIntError,
    number: String,
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_INPUT_1: &str = include_str!("../example-input-1");

    #[test]
    #[snafu::report]
    fn example_1() -> Result<(), ParseError> {
        assert_eq!(13, sum_of_winning_points(EXAMPLE_INPUT_1)?);

        Ok(())
    }
}
