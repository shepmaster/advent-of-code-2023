use snafu::prelude::*;

const INPUT: &str = include_str!("../input");

#[snafu::report]
fn main() -> Result<(), CalibrationError> {
    let sum = calibration_sum(INPUT)?;
    // Part 1: 53080
    // Part 2: 53268
    println!("{sum}");

    Ok(())
}

fn calibration_sum(s: &str) -> Result<u32, CalibrationError> {
    s.lines()
        .map(|line| {
            let line_bytes = line.as_bytes();
            let mut sublines_fwd = (0..line_bytes.len())
                .map(|i| &line_bytes[i..])
                .flat_map(parse_digit);
            let mut sublines_rev = sublines_fwd.clone();

            let first = sublines_fwd.next().context(FirstMissingSnafu { line })?;
            let last = sublines_rev
                .next_back()
                .context(LastMissingSnafu { line })?;

            Ok(first * 10 + last)
        })
        .sum()
}

fn parse_digit(s: &[u8]) -> Option<u32> {
    let table: [(u32, [&[u8]; 2]); 10] = [
        (0, [b"0", b"zero"]),
        (1, [b"1", b"one"]),
        (2, [b"2", b"two"]),
        (3, [b"3", b"three"]),
        (4, [b"4", b"four"]),
        (5, [b"5", b"five"]),
        (6, [b"6", b"six"]),
        (7, [b"7", b"seven"]),
        (8, [b"8", b"eight"]),
        (9, [b"9", b"nine"]),
    ];

    for &(value, ref matchers) in &table {
        for &matcher in matchers {
            if s.starts_with(matcher) {
                return Some(value);
            }
        }
    }

    None
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

    const EXAMPLE_INPUT_2: &str = r#"two1nine
eightwothree
abcone2threexyz
xtwone3four
4nineeightseven2
zoneight234
7pqrstsixteen"#;

    #[test]
    #[snafu::report]
    fn example_2() -> Result<(), CalibrationError> {
        assert_eq!(281, calibration_sum(EXAMPLE_INPUT_2)?);

        Ok(())
    }
}
