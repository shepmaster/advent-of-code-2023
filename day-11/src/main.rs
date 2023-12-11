use snafu::prelude::*;
use std::collections::BTreeSet;

const INPUT: &str = include_str!("../input");

#[snafu::report]
fn main() -> Result<(), Error> {
    let sum = sum_of_shortest_paths(INPUT)?;
    // Part 1: 9522407
    println!("{sum}");

    Ok(())
}

fn sum_of_shortest_paths(s: &str) -> Result<usize, Error> {
    let mut galaxies = Vec::new();
    let mut seen_columns = BTreeSet::new();
    let mut seen_rows = BTreeSet::new();

    for (y, line) in s.lines().enumerate() {
        for (x, c) in line.chars().enumerate() {
            if c == '#' {
                galaxies.push((x, y));

                seen_columns.insert(x);
                seen_rows.insert(y);
            }
        }
    }

    let mut q = &galaxies[..];
    let mut distance_sum = 0;

    while let Some((head, tails)) = q.split_first() {
        for tail in tails {
            let [x0, x1] = order(head.0, tail.0);
            let [y0, y1] = order(head.1, tail.1);

            let x_base = x1 - x0;
            let y_base = y1 - y0;

            let x_expansion = (x0..=x1).filter(|x| !seen_columns.contains(x)).count();
            let y_expansion = (y0..=y1).filter(|y| !seen_rows.contains(y)).count();

            let x = x_base + x_expansion;
            let y = y_base + y_expansion;

            distance_sum += x + y;
        }
        q = tails;
    }

    Ok(distance_sum)
}

fn order(a: usize, b: usize) -> [usize; 2] {
    let mut x = [a, b];
    x.sort();
    x
}

#[derive(Debug, Snafu)]
enum Error {}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_INPUT_1: &str = include_str!("../example-input-1");

    #[test]
    #[snafu::report]
    fn example_1() -> Result<(), Error> {
        assert_eq!(374, sum_of_shortest_paths(EXAMPLE_INPUT_1)?);

        Ok(())
    }
}
