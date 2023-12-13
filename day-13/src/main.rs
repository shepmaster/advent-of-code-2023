use itertools::Itertools;
use snafu::prelude::*;
use std::collections::BTreeMap;

const INPUT: &str = include_str!("../input");

#[snafu::report]
fn main() -> Result<(), Error> {
    let summary = summarize(INPUT)?;
    // Part 1: 34821
    println!("{summary}");

    let summary = summarize_with_smudges(INPUT)?;
    // Part 2: 37005 (too high)
    // -> Wasn't ensuring that it was a different fold index.
    //         28945 (too low)
    // -> Wasn't getting all possible original fold indexes before
    //    discarding the original index.
    //         36919
    println!("{summary}");

    Ok(())
}

fn summarize(s: &str) -> Result<usize, Error> {
    let sum = s
        .split("\n\n")
        .map(|raw_grid| {
            let grid = build_grid(raw_grid);

            let &(x_max, y_max) = grid.keys().last().expect("Grid has no values");

            // Iterate over the grid horizontally and vertically,
            // reducing each row or column down to a pattern of
            // booleans, then assigning each unique pattern an ID.

            let (_unique_column_patterns, columns) =
                build_patterns(&grid, 0..=x_max, 0..=y_max, |x, y| (x, y));

            let (_unique_row_patterns, rows) =
                build_patterns(&grid, 0..=y_max, 0..=x_max, |y, x| (x, y));

            // We now have two one-dimensional views of the
            // grid. Potential fold points occur whenever two pattern
            // IDs occur next to each other.

            let column_fold_idx = fold_index(&columns);
            let row_fold_idx = fold_index(&rows);

            to_score(raw_grid, column_fold_idx, row_fold_idx)
        })
        .sum();

    Ok(sum)
}

fn summarize_with_smudges(s: &str) -> Result<usize, Error> {
    let sum = s
        .split("\n\n")
        .map(|raw_grid| {
            let grid = build_grid(raw_grid);

            let &(x_max, y_max) = grid.keys().last().expect("Grid has no values");

            // Iterate over the grid horizontally and vertically,
            // reducing each row or column down to a pattern of
            // booleans, then assigning each unique pattern an ID.

            let (unique_column_patterns, columns) =
                build_patterns(&grid, 0..=x_max, 0..=y_max, |x, y| (x, y));

            let (unique_row_patterns, rows) =
                build_patterns(&grid, 0..=y_max, 0..=x_max, |y, x| (x, y));

            // Find potential swaps by looking at patterns that differ
            // by only one element.

            let column_swaps = potential_swaps(&unique_column_patterns);
            let row_swaps = potential_swaps(&unique_row_patterns);

            // Walk through each pattern, replacing each possible
            // pattern swap at a time, then evaluate if the modified
            // pattern could be a fold.

            let column_fold_idx = fold_index_with_swaps(&columns, &column_swaps);
            let row_fold_idx = fold_index_with_swaps(&rows, &row_swaps);

            to_score(raw_grid, column_fold_idx, row_fold_idx)
        })
        .sum();

    Ok(sum)
}

type Coord = (usize, usize);

type Grid = BTreeMap<Coord, bool>;

type UniquePatternMap = BTreeMap<Vec<bool>, usize>;

type Patterns = Vec<usize>;

type Swap = [usize; 2];

fn build_grid(grid: &str) -> Grid {
    grid.lines()
        .enumerate()
        .flat_map(|(y, line)| {
            line.chars()
                .enumerate()
                .map(move |(x, c)| ((x, y), c == '#'))
        })
        .collect()
}

fn build_patterns(
    grid: &Grid,
    major: impl Iterator<Item = usize> + Clone,
    minor: impl Iterator<Item = usize> + Clone,
    f: impl Fn(usize, usize) -> Coord,
) -> (UniquePatternMap, Patterns) {
    let mut unique_patterns = BTreeMap::new();
    let patterns = major
        .map(|ma| {
            let pattern = minor
                .clone()
                .flat_map(|mi| grid.get(&f(ma, mi)).copied())
                .collect();

            let next_id = unique_patterns.len();
            *unique_patterns.entry(pattern).or_insert(next_id)
        })
        .collect();

    (unique_patterns, patterns)
}

fn fold_index(patterns: &Patterns) -> Option<usize> {
    fold_indices(patterns).next()
}

fn fold_indices(patterns: &Patterns) -> impl Iterator<Item = usize> + '_ {
    let potential_fold_idxs = patterns
        .iter()
        .tuple_windows()
        .enumerate()
        .filter(|(_, (a, b))| a == b)
        .map(|(i, _)| i + 1);

    potential_fold_idxs.filter(|&fold_idx| {
        let (a, b) = patterns.split_at(fold_idx);

        // Using `zip` instead of `eq` to ignore mismatched lengths
        a.iter().rev().zip(b).all(|(a, b)| a == b)
    })
}

fn to_score(grid: &str, column_fold_idx: Option<usize>, row_fold_idx: Option<usize>) -> usize {
    match (column_fold_idx, row_fold_idx) {
        (Some(c), None) => c,
        (None, Some(r)) => r * 100,
        (c, r) => {
            eprintln!("Grid had impossible solution");
            eprintln!();
            eprintln!("{grid}");
            eprintln!();
            eprintln!("{c:?} / {r:?}");
            panic!();
        }
    }
}

fn potential_swaps(unique_patterns: &UniquePatternMap) -> Vec<Swap> {
    unique_patterns
        .iter()
        .map(|(k, &v)| (&k[..], v))
        .combinations(2)
        .map(|v| <[_; 2]>::try_from(v).expect("Must have two"))
        .filter(|[a, b]| a.0.iter().zip(b.0).filter(|(a, b)| a != b).count() == 1)
        .map(|[a, b]| [a.1, b.1])
        .collect()
}

fn fold_index_with_swaps(patterns: &Patterns, swaps: &[Swap]) -> Option<usize> {
    let original_idx = fold_index(patterns);

    patterns
        .iter()
        .enumerate()
        .flat_map(|(i, &pattern)| {
            let fold_idx_with_swap = move |src, dst| {
                if pattern == src {
                    let mut swapped = patterns.clone();
                    swapped[i] = dst;
                    fold_indices(&swapped).collect()
                } else {
                    vec![]
                }
            };

            swaps.iter().flat_map(move |&[a, b]| {
                let idx_a = fold_idx_with_swap(a, b);
                let idx_b = fold_idx_with_swap(b, a);

                itertools::chain(idx_a, idx_b)
            })
        })
        .find(|&idx| Some(idx) != original_idx)
}

#[derive(Debug, Snafu)]
enum Error {}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_INPUT_1: &str = include_str!("../example-input-1");
    const REPRO_INPUT_1: &str = include_str!("../repro-input-1");

    #[test]
    #[snafu::report]
    fn example_1() -> Result<(), Error> {
        assert_eq!(405, summarize(EXAMPLE_INPUT_1)?);

        Ok(())
    }

    #[test]
    #[snafu::report]
    fn example_2() -> Result<(), Error> {
        assert_eq!(400, summarize_with_smudges(EXAMPLE_INPUT_1)?);

        Ok(())
    }

    #[test]
    #[snafu::report]
    fn repro_1() -> Result<(), Error> {
        assert_eq!(5, summarize_with_smudges(REPRO_INPUT_1)?);

        Ok(())
    }
}
