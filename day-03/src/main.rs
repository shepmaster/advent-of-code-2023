use itertools::Itertools;
use snafu::prelude::*;
use std::collections::BTreeMap;

const INPUT: &str = include_str!("../input");

type Coordinate = (usize, usize);
type ComponentMap = BTreeMap<(usize, usize), Component>;

#[snafu::report]
fn main() -> Result<(), ParseComponentMapError> {
    let sum = sum_of_part_numbers(INPUT)?;
    // Part 1: 530849
    println!("{sum}");

    Ok(())
}

#[derive(Debug, Copy, Clone)]
enum Component {
    Symbol(char),
    Number(Number),
}

#[derive(Debug, Copy, Clone)]
struct Number {
    num: u64,
    width: usize,
}

impl Component {
    fn is_symbol(&self) -> bool {
        matches!(self, Self::Symbol(..))
    }

    fn into_number(self) -> Option<Number> {
        match self {
            Self::Number(n) => Some(n),
            _ => None,
        }
    }
}

fn sum_of_part_numbers(s: &str) -> Result<u64, ParseComponentMapError> {
    let components = parse_component_map(s)?;

    let numbers = components
        .iter()
        .flat_map(|(pos, c)| c.into_number().map(|n| (*pos, n)));

    Ok(numbers
        .filter(|&(pos, n)| {
            fringe(pos, n.width)
                .any(|coord| components.get(&coord).map_or(false, |c| c.is_symbol()))
        })
        .map(|(_pos, n)| n.num)
        .sum())
}

fn fringe((x, y): Coordinate, width: usize) -> impl Iterator<Item = Coordinate> {
    let x_start = x.checked_sub(1);
    let x_end = x.checked_add(width);

    let left_edge = [
        (x_start, y.checked_sub(1)),
        (x_start, Some(y)),
        (x_start, y.checked_add(1)),
    ];

    let middle = (0..width)
        .flat_map(move |d| x.checked_add(d))
        .flat_map(move |x| [(Some(x), y.checked_sub(1)), (Some(x), y.checked_add(1))]);

    let right_edge = [
        (x_end, y.checked_sub(1)),
        (x_end, Some(y)),
        (x_end, y.checked_add(1)),
    ];

    left_edge
        .into_iter()
        .chain(middle)
        .chain(right_edge)
        .flat_map(|x| Some((x.0?, x.1?)))
}

fn parse_component_map(s: &str) -> Result<ComponentMap, ParseComponentMapError> {
    let mut components = BTreeMap::new();

    for (y, line) in s.lines().enumerate() {
        let mut head = line;
        let mut x = 0;

        loop {
            let number_position = head
                .char_indices()
                .take_while_ref(|(_, c)| c.is_ascii_digit())
                .last();
            match number_position {
                Some((idx, c)) => {
                    let end_idx = idx + c.len_utf8();
                    let (num, rest) = head.split_at(end_idx);

                    let width = num.len();
                    let num = num.parse().context(InvalidNumberSnafu { num })?;

                    components.insert((x, y), Component::Number(Number { num, width }));

                    head = rest;
                    x += width;
                }
                None => {
                    match head.chars().next() {
                        Some(c) => {
                            match c {
                                '.' => { /* blank space */ }
                                s => {
                                    components.insert((x, y), Component::Symbol(s));
                                }
                            }

                            head = &head[1..];
                            x += 1;
                        }
                        None => {
                            // End of line
                            break;
                        }
                    }
                }
            }
        }
    }

    Ok(components)
}

#[derive(Debug, Snafu)]
enum ParseComponentMapError {
    InvalidNumber {
        source: std::num::ParseIntError,
        num: String,
    },
}

#[cfg(test)]
mod test {
    use std::collections::BTreeSet;

    use super::*;

    const EXAMPLE_INPUT_1: &str = include_str!("../example-input-1");

    #[test]
    #[snafu::report]
    fn example_1() -> Result<(), ParseComponentMapError> {
        assert_eq!(4361, sum_of_part_numbers(EXAMPLE_INPUT_1)?);

        Ok(())
    }

    #[test]
    fn fringe_handles_edges() {
        let found = fringe((0, 0), 1).collect::<BTreeSet<_>>();
        let expected = BTreeSet::from_iter([(1, 0), (1, 1), (0, 1)]);

        assert_eq!(found, expected);
    }
}
