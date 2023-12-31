use itertools::Itertools;
use snafu::prelude::*;

const INPUT: &str = include_str!("../input");

#[snafu::report]
fn main() -> Result<(), Error> {
    let sum = sum_of_extrapolated_histories(INPUT)?;
    // Part 1: 2008960228
    println!("{sum}");

    let sum = sum_of_backwards_extrapolated_histories(INPUT)?;
    // Part 2: 1097
    println!("{sum}");

    Ok(())
}

fn sum_of_extrapolated_histories(s: &str) -> Result<i64, Error> {
    s.lines().map(extrapolated_history).sum()
}

fn sum_of_backwards_extrapolated_histories(s: &str) -> Result<i64, Error> {
    s.lines().map(backwards_extrapolated_history).sum()
}

fn extrapolated_history(line: &str) -> Result<i64, Error> {
    let all_numbers = build_differences(line)?;

    #[allow(clippy::unnecessary_fold)] // parallel code with backwards impl
    let next_value = all_numbers
        .iter()
        .rev()
        .flat_map(|ns| ns.last())
        .fold(0, |acc, v| v + acc);

    Ok(next_value)
}

fn backwards_extrapolated_history(line: &str) -> Result<i64, Error> {
    let all_numbers = build_differences(line)?;

    let prev_value = all_numbers
        .iter()
        .rev()
        .flat_map(|ns| ns.first())
        .fold(0, |acc, v| v - acc);

    Ok(prev_value)
}

fn build_differences(line: &str) -> Result<Vec<Vec<i64>>, Error> {
    let numbers = line
        .split_ascii_whitespace()
        .map(|number| number.parse().context(InvalidNumberSnafu { number }))
        .collect::<Result<Vec<i64>, _>>()?;

    let mut last = numbers;
    let mut all_numbers = Vec::new();

    loop {
        let next: Vec<_> = last.iter().tuple_windows().map(|(l, r)| r - l).collect();

        all_numbers.push(last);

        if next.iter().all(|&n| n == 0) {
            break;
        }

        last = next;
    }

    Ok(all_numbers)
}

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("Could not parse '{number}'"))]
    InvalidNumber {
        source: std::num::ParseIntError,
        number: String,
    },
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_INPUT_1: &str = include_str!("../example-input-1");

    #[test]
    #[snafu::report]
    fn example_1() -> Result<(), Error> {
        assert_eq!(114, sum_of_extrapolated_histories(EXAMPLE_INPUT_1)?);

        Ok(())
    }

    #[test]
    #[snafu::report]
    fn example_2() -> Result<(), Error> {
        assert_eq!(2, sum_of_backwards_extrapolated_histories(EXAMPLE_INPUT_1)?);

        Ok(())
    }
}
