use itertools::Itertools;
use snafu::prelude::*;
use std::{collections::BTreeMap, rc::Rc};

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

#[derive(Debug, Clone)]
enum Component {
    Symbol(char),
    Number(Rc<u64>),
}

impl Component {
    fn is_symbol(&self) -> bool {
        matches!(self, Self::Symbol(..))
    }

    fn as_number(&self) -> Option<&u64> {
        match self {
            Self::Number(n) => Some(n),
            _ => None,
        }
    }
}

fn sum_of_part_numbers(s: &str) -> Result<u64, ParseComponentMapError> {
    let components = parse_component_map(s)?;

    let symbol_positions = components
        .iter()
        .flat_map(|(pos, c)| c.is_symbol().then_some(pos));

    let possible_numbers = symbol_positions
        .flat_map(|&sym_pos| fringe(sym_pos).flat_map(|pos| components.get(&pos)?.as_number()));

    let mut possible_numbers = possible_numbers.collect::<Vec<_>>();

    // Unique by reference equality
    possible_numbers.sort_by_key(|n| &**n as *const u64);
    possible_numbers.dedup_by_key(|n| &**n as *const u64);

    Ok(possible_numbers.into_iter().sum())
}

fn fringe((x, y): Coordinate) -> impl Iterator<Item = Coordinate> {
    let x_start = x.checked_sub(1);
    let x_end = x.checked_add(1);

    let y_start = y.checked_sub(1);
    let y_end = y.checked_add(1);

    [
        // left edge
        (x_start, y_start),
        (x_start, Some(y)),
        (x_start, y_end),
        // middle
        (Some(x), y_start),
        (Some(x), y_end),
        // right edge
        (x_end, y_start),
        (x_end, Some(y)),
        (x_end, y_end),
    ]
    .into_iter()
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

                    for (w, num) in itertools::repeat_n(Rc::new(num), width).enumerate() {
                        components.insert((x + w, y), Component::Number(num));
                    }

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
        let found = fringe((0, 0)).collect::<BTreeSet<_>>();
        let expected = BTreeSet::from_iter([(1, 0), (1, 1), (0, 1)]);

        assert_eq!(found, expected);
    }
}
