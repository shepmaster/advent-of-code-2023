use snafu::prelude::*;
use std::{cmp, collections::BTreeSet};

const INPUT: &str = include_str!("../input");

#[snafu::report]
fn main() -> Result<(), Error> {
    let load = total_load(INPUT)?;
    // Part 1: 108641
    println!("{load}");

    Ok(())
}

fn total_load(s: &str) -> Result<usize, Error> {
    let mut cubes = BTreeSet::new();
    let mut balls = BTreeSet::new();

    let mut x_max = 0;
    let mut y_max = 0;

    for (y, line) in s.lines().enumerate() {
        for (x, c) in line.chars().enumerate() {
            let set = match c {
                'O' => &mut balls,
                '#' => &mut cubes,
                _ => continue,
            };
            set.insert((x, y));

            x_max = cmp::max(x_max, x);
        }

        y_max = cmp::max(y_max, y);
    }

    // Walk down each column. If we see a ball, move it to the
    // furthest spot avaiable. If we see a cube, update where the
    // furthest spot would be.

    let sum = (0..=x_max)
        .map(|x| {
            let mut sum = 0;
            let mut dest = 0;

            for y in 0..=y_max {
                let coord = (x, y);

                if balls.contains(&coord) {
                    // Adding one as we are counting from the *edge* of
                    // the platform
                    sum += y_max - dest + 1;
                    dest += 1;
                } else if cubes.contains(&coord) {
                    // A cube is at this spot, so the balls will slide to
                    // the *next* spot
                    dest = y + 1;
                }
            }

            sum
        })
        .sum();

    Ok(sum)
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
        assert_eq!(136, total_load(EXAMPLE_INPUT_1)?);

        Ok(())
    }
}
