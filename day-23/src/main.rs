use std::{
    collections::{BTreeMap, BTreeSet},
    ops::RangeInclusive,
};

const INPUT: &str = include_str!("../input");

fn main() {
    let length = longest_hike(INPUT);
    assert_eq!(2298, length);
    println!("{length}");
}

fn longest_hike(s: &str) -> usize {
    use Tile::*;

    let mut map = BTreeMap::new();

    for (y, line) in s.lines().enumerate() {
        for (x, c) in line.chars().enumerate() {
            let coord = (x, y);

            let tile = match c {
                '.' => Path,
                '#' => Forest,
                '^' => Up,
                '>' => Right,
                'v' => Down,
                '<' => Left,
                oth => unreachable!("Invalid input '{oth}'"),
            };

            map.insert(coord, tile);
        }
    }

    let &(x_max, y_max) = map.keys().last().expect("Should have one tile");

    let start_x = (0..=x_max)
        .find(|&x| map[&(x, 0)] == Path)
        .expect("Start tile missing");
    let start = (start_x, 0);

    let end_x = (0..=x_max)
        .find(|&x| map[&(x, y_max)] == Path)
        .expect("End tile missing");
    let end = (end_x, y_max);

    let bounds = Bounds {
        x: 0..=x_max,
        y: 0..=y_max,
    };

    let mut queue = vec![(start, BTreeSet::new())];
    let mut successes = vec![];

    while let Some((coord, mut visited)) = queue.pop() {
        // Reached the end
        if coord == end {
            successes.push(visited);
            continue;
        }

        // Already been here
        if !visited.insert(coord) {
            continue;
        }

        let nexts = match &map[&coord] {
            Path => vec![
                bounds.up(coord),
                bounds.right(coord),
                bounds.left(coord),
                bounds.down(coord),
            ],
            Forest => continue,
            Up => vec![bounds.up(coord)],
            Right => vec![bounds.right(coord)],
            Left => vec![bounds.left(coord)],
            Down => vec![bounds.down(coord)],
        };

        for next in nexts.into_iter().flatten() {
            queue.push((next, visited.clone()));
        }
    }

    let max_len = successes
        .iter()
        .map(|s| s.len())
        .max()
        .expect("No paths found");

    max_len
}

type Dim = usize;
type Coord = (Dim, Dim);

#[derive(Debug)]
struct Bounds {
    x: RangeInclusive<Dim>,
    y: RangeInclusive<Dim>,
}

impl Bounds {
    fn up(&self, (x, y): Coord) -> Option<Coord> {
        Some((x, y.checked_sub(1)?))
    }

    fn right(&self, (x, y): Coord) -> Option<Coord> {
        if x == *self.x.end() {
            None
        } else {
            Some((x + 1, y))
        }
    }

    fn down(&self, (x, y): Coord) -> Option<Coord> {
        if y == *self.y.end() {
            None
        } else {
            Some((x, y + 1))
        }
    }

    fn left(&self, (x, y): Coord) -> Option<Coord> {
        Some((x.checked_sub(1)?, y))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Tile {
    Path,
    Forest,
    Up,
    Right,
    Left,
    Down,
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_INPUT_1: &str = include_str!("../example-input-1");

    #[test]
    fn example_1() {
        assert_eq!(94, longest_hike(EXAMPLE_INPUT_1));
    }
}
