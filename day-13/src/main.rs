use itertools::Itertools;
use snafu::prelude::*;
use std::collections::BTreeMap;

const INPUT: &str = include_str!("../input");

#[snafu::report]
fn main() -> Result<(), Error> {
    let summary = summarize(INPUT)?;
    // Part 1: 34821
    println!("{summary}");

    Ok(())
}

fn summarize(s: &str) -> Result<usize, Error> {
    let grids = s.split("\n\n").map(|grid| {
        grid.lines()
            .enumerate()
            .flat_map(|(y, line)| {
                line.chars()
                    .enumerate()
                    .map(move |(x, c)| ((x, y), c == '#'))
            })
            .collect::<BTreeMap<_, _>>()
    });

    let sum = grids
        .map(|g| {
            let &(x_max, y_max) = g.keys().last().expect("Grid has no values");

            // Iterate over the grid horizontally and vertically,
            // reducing each row or column down to a pattern of
            // booleans, then assigning each unique pattern an ID.

            let mut unique_column_patterns = BTreeMap::new();
            let columns = (0..=x_max)
                .map(|x| {
                    let column = (0..=y_max)
                        .flat_map(|y| g.get(&(x, y)).copied())
                        .collect::<Vec<_>>();

                    let next_id = unique_column_patterns.len();
                    *unique_column_patterns.entry(column).or_insert(next_id)
                })
                .collect::<Vec<_>>();

            let mut unique_row_patterns = BTreeMap::new();
            let rows = (0..=y_max)
                .map(|y| {
                    let row = (0..=x_max)
                        .flat_map(|x| g.get(&(x, y)).copied())
                        .collect::<Vec<_>>();

                    let next_id = unique_row_patterns.len();
                    *unique_row_patterns.entry(row).or_insert(next_id)
                })
                .collect::<Vec<_>>();

            // We now have two one-dimensional views of the
            // grid. Potential fold points occur whenever two pattern
            // IDs occur next to each other.

            let mut potential_column_fold_idxs = columns
                .iter()
                .tuple_windows()
                .enumerate()
                .filter(|(_, (a, b))| a == b)
                .map(|(i, _)| i + 1);

            let column_fold_idx = potential_column_fold_idxs.find(|&fold_idx| {
                let (a, b) = columns.split_at(fold_idx);

                // Using `zip` instead of `eq` to ignore mismatched lengths
                a.iter().rev().zip(b).all(|(a, b)| a == b)
            });

            let mut potential_row_fold_idxs = rows
                .iter()
                .tuple_windows()
                .enumerate()
                .filter(|(_, (a, b))| a == b)
                .map(|(i, _)| i + 1);

            let row_fold_idx = potential_row_fold_idxs.find(|&fold_idx| {
                let (a, b) = rows.split_at(fold_idx);

                // Using `zip` instead of `eq` to ignore mismatched lengths
                a.iter().rev().zip(b).all(|(a, b)| a == b)
            });

            column_fold_idx.unwrap_or(0) + row_fold_idx.unwrap_or(0) * 100
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
        assert_eq!(405, summarize(EXAMPLE_INPUT_1)?);

        Ok(())
    }
}
