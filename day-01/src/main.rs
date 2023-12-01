use snafu::prelude::*;

const INPUT: &str = include_str!("../input");

#[snafu::report]
fn main() -> Result<(), CalibrationError> {
    let sum = calibration_sum(INPUT)?;
    // Part 1: 53080
    println!("{sum}");

    Ok(())
}

fn calibration_sum(s: &str) -> Result<u32, CalibrationError> {
    s.lines()
        .map(|line| {
            let mut forward_digits = line.chars().flat_map(|c| c.to_digit(10));
            let mut backward_digits = forward_digits.clone();

            let first = forward_digits.next().context(FirstMissingSnafu { line })?;
            let last = backward_digits
                .next_back()
                .context(LastMissingSnafu { line })?;

            Ok(first * 10 + last)
        })
        .sum()
}

#[derive(Debug, Snafu)]
enum CalibrationError {
    #[snafu(display("There was no first digit in '{line}'"))]
    FirstMissing { line: String },

    #[snafu(display("There was no last digit in '{line}'"))]
    LastMissing { line: String },
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_INPUT_1: &str = r#"1abc2
pqr3stu8vwx
a1b2c3d4e5f
treb7uchet"#;

    #[test]
    #[snafu::report]
    fn example_1() -> Result<(), CalibrationError> {
        assert_eq!(142, calibration_sum(EXAMPLE_INPUT_1)?);

        Ok(())
    }
}
