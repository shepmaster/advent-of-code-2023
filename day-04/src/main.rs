use snafu::prelude::*;
use std::collections::BTreeSet;

const INPUT: &str = include_str!("../input");

type Num = u8;

#[snafu::report]
fn main() -> Result<u64, ParseError> {
    let sum = sum_of_winning_points(INPUT)?;
    // Part 1: 25004
    println!("{sum}");

    let number = number_of_scratchcards(INPUT)?;
    // Part 2: 14427616
    println!("{number}");

    Ok(())
}

fn sum_of_winning_points(s: &str) -> Result<u64, ParseError> {
    s.lines()
        .map(|line| {
            let n_matches = n_matches(line)?;

            let points = if n_matches == 0 {
                0
            } else {
                1 << (n_matches - 1)
            };

            Ok(points)
        })
        .sum()
}

fn number_of_scratchcards(s: &str) -> Result<usize, ParseError> {
    let matches = s.lines().map(n_matches).collect::<Result<Vec<_>, _>>()?;
    let mut total_count = vec![1; matches.len()];

    for (idx, &matches) in matches.iter().enumerate() {
        let current_count = total_count[idx];

        for idx in (idx..).take(matches) {
            // Adding one to get the card after the current
            if let Some(total) = total_count.get_mut(idx + 1) {
                *total += current_count;
            }
        }
    }

    Ok(total_count.into_iter().sum())
}

fn n_matches(line: &str) -> Result<usize, ParseError> {
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

    itertools::process_results(numbers, |numbers| {
        numbers.filter(|n| winners.contains(n)).count()
    })
    .context(InvalidNumberSnafu { line })
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

    #[test]
    #[snafu::report]
    fn example_2() -> Result<(), ParseError> {
        assert_eq!(30, number_of_scratchcards(EXAMPLE_INPUT_1)?);

        Ok(())
    }
}
