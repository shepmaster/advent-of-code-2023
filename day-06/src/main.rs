use itertools::Either;
use snafu::prelude::*;
use std::iter;

const INPUT: &str = include_str!("../input");

#[snafu::report]
fn main() -> Result<(), Error> {
    let product = product_of_number_of_possible_wins(INPUT)?;
    // Part 1: 800280
    println!("{product}");

    let number = number_of_possible_wins_fixed_kerning(INPUT)?;
    // Part 1: 45128024
    println!("{number}");

    Ok(())
}

fn product_of_number_of_possible_wins(s: &str) -> Result<u64, Error> {
    let mut lines = s.lines();

    let times = lines.next().context(TimesMissingSnafu)?;
    let distances = lines.next().context(DistancesMissingSnafu)?;

    let times = parse_sequence(times);
    let distances = parse_sequence(distances);

    times
        .zip(distances)
        .map(|(time, distance)| {
            let time = time.context(TimesInvalidSnafu)?;
            let distance = distance.context(DistancesInvalidSnafu)?;

            Ok(number_of_possible_wins(time, distance))
        })
        .product()
}

fn number_of_possible_wins_fixed_kerning(s: &str) -> Result<u64, Error> {
    let mut lines = s.lines();

    let times = lines.next().context(TimesMissingSnafu)?;
    let distances = lines.next().context(DistancesMissingSnafu)?;

    let smush = |s: &str| {
        s.chars()
            .filter(|c| c.is_ascii_digit())
            .collect::<String>()
            .parse()
    };

    let time = smush(times).context(TimeInvalidSnafu)?;
    let distance = smush(distances).context(DistanceInvalidSnafu)?;

    Ok(number_of_possible_wins(time, distance))
}

// `t` is total time
// `p` is time spent pressing button
// `d` is the distance to beat
//
// solve for `p`
// (t-p)*p > d
// tp - p^2 > d
// -p^2 + tp - d > 0
//
// quadratic formula
// (-b ± sqrt(b^2 - 4ac)) / 2a
// a = -1
// b = t
// c = -d
//
// (-t ± sqrt(t^2 - 4d)) / -2
// (t ± sqrt(t^2 - 4d)) / 2
// t/2 ± sqrt(t^2 - 4d)/2
fn number_of_possible_wins(time: f64, distance: f64) -> u64 {
    let pt1 = time / 2.0;
    let pt2 = (time.powi(2) - 4.0 * distance).sqrt() / 2.0;

    let lower = pt1 - pt2;
    let upper = pt1 + pt2;

    let next_lower = lower.ceil();
    let prev_upper = upper.floor();

    // If we matched the integer value exactly, we need to nudge
    // towards the middle so that we *win* instead of tie.
    let lower = if lower == next_lower {
        next_lower + 1.0
    } else {
        next_lower
    };

    let upper = if upper == prev_upper {
        prev_upper - 1.0
    } else {
        prev_upper
    };

    let lower = lower as u64;
    let upper = upper as u64;

    // Adding one to account for the fencepost
    upper - lower + 1
}

#[derive(Debug, Snafu)]
enum Error {
    TimesMissing,

    DistancesMissing,

    TimesInvalid { source: ParseSequenceError },

    DistancesInvalid { source: ParseSequenceError },

    TimeInvalid { source: std::num::ParseFloatError },

    DistanceInvalid { source: std::num::ParseFloatError },
}

fn parse_sequence(s: &str) -> impl Iterator<Item = Result<f64, ParseSequenceError>> + '_ {
    use parse_sequence_error::*;

    match s.split_once(':').context(MalformedSnafu) {
        Ok((_, values)) => Either::Left(
            values
                .split_ascii_whitespace()
                .map(|value| value.parse().context(InvalidSnafu { value })),
        ),
        Err(err) => Either::Right(iter::once(Err(err))),
    }
}

#[derive(Debug, Snafu)]
#[snafu(module)]
enum ParseSequenceError {
    Malformed,

    Invalid {
        source: std::num::ParseFloatError,
        value: String,
    },
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_INPUT_1: &str = include_str!("../example-input-1");

    #[test]
    #[snafu::report]
    fn example_1() -> Result<(), Error> {
        assert_eq!(288, product_of_number_of_possible_wins(EXAMPLE_INPUT_1)?);

        Ok(())
    }

    #[test]
    #[snafu::report]
    fn example_2() -> Result<(), Error> {
        assert_eq!(
            71503,
            number_of_possible_wins_fixed_kerning(EXAMPLE_INPUT_1)?
        );

        Ok(())
    }
}
