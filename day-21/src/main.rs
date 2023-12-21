use std::{
    cmp,
    collections::{btree_map::Entry, BTreeMap, BTreeSet, VecDeque},
    fmt,
    ops::RangeInclusive,
};

const INPUT: &str = include_str!("../input");

fn main() {
    let plots = unique_reachable_plots(INPUT, 64);
    // Part 1: 3872 (too high)
    // -> TYPO?!?!?!
    //       : 3782
    println!("{plots}");
}

fn unique_reachable_plots(s: &str, n_steps: usize) -> usize {
    let mut start = None;
    let mut rocks = BTreeSet::new();

    let mut x_max = 0;
    let mut y_max = 0;

    for (y, line) in s.lines().enumerate() {
        for (x, c) in line.chars().enumerate() {
            match c {
                '#' => {
                    rocks.insert((x, y));
                }
                'S' => start = Some((x, y)),
                _ => {}
            }

            x_max = cmp::max(x_max, x);
        }
        y_max = cmp::max(y_max, y);
    }

    let start = start.expect("Input malformed; no starting position");
    let rocks = rocks;
    let _bounds = Bounds::new(x_max, y_max);

    let mut queue = VecDeque::from_iter([(start, n_steps)]);
    let mut visited = BTreeMap::new();

    // let mut frontier = n_steps;

    while let Some((coord, n_steps_left)) = queue.pop_front() {
        // if frontier != n_steps_left {
        //     let gv = GridView {
        //         bounds: &_bounds,
        //         rocks: &rocks,
        //         visited: &visited,
        //     };
        //     eprintln!("-xxx- {frontier}\n\n{gv}");
        //     frontier = n_steps_left;
        // }

        match visited.entry(coord) {
            Entry::Vacant(e) => e.insert(n_steps - n_steps_left),
            Entry::Occupied(_e) => {
                // Never toggle between even or odd, right?
                assert_eq!(_e.get() % 2 == 0, (n_steps - n_steps_left) % 2 == 0);
                continue;
            }
        };

        if let Some(next_n_steps_left) = n_steps_left.checked_sub(1) {
            let (x, y) = coord;
            let nexts = [(x, y - 1), (x + 1, y), (x, y + 1), (x - 1, y)];

            for next in nexts {
                if !rocks.contains(&next) {
                    queue.push_back((next, next_n_steps_left));
                }
            }
        }
    }

    // let gv = GridView {
    //     bounds: &_bounds,
    //     rocks: &rocks,
    //     visited: &visited,
    // };
    // eprintln!("-xxx- 0\n\n{gv}");

    let is_even = n_steps % 2 == 0;
    visited.retain(|_, n_steps| (*n_steps % 2 == 0) == is_even);

    // let gv = GridView {
    //     bounds: &_bounds,
    //     rocks: &rocks,
    //     visited: &visited,
    // };
    // eprintln!("{gv}");

    visited.len()
}

type Dim = usize;
type Coord = (Dim, Dim);

#[derive(Debug)]
struct Bounds {
    x: RangeInclusive<Dim>,
    y: RangeInclusive<Dim>,
}

impl Bounds {
    fn new(x: Dim, y: Dim) -> Self {
        Self { x: 0..=x, y: 0..=y }
    }
}

struct GridView<'a> {
    bounds: &'a Bounds,
    rocks: &'a BTreeSet<Coord>,
    visited: &'a BTreeMap<Coord, usize>,
}

impl fmt::Display for GridView<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in self.bounds.y.clone() {
            for x in self.bounds.x.clone() {
                let coord = (x, y);
                let rock = self.rocks.contains(&coord);
                let visit = self.visited.get(&coord);

                match (rock, visit) {
                    (true, Some(_)) => '?'.fmt(f)?,
                    (true, None) => '#'.fmt(f)?,
                    (false, Some(s)) => {
                        if *s < 10 {
                            s.fmt(f)?;
                        } else {
                            'O'.fmt(f)?;
                        }
                    }
                    (false, None) => '.'.fmt(f)?,
                }
            }
            "\n".fmt(f)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_INPUT_1: &str = include_str!("../example-input-1");

    #[test]
    fn example_1() {
        let cases = [(1, 2), (2, 4), (3, 6), (4, 9), (5, 13), (6, 16)];

        for (steps, plots) in cases {
            assert_eq!(
                plots,
                unique_reachable_plots(EXAMPLE_INPUT_1, steps),
                "In {steps} steps"
            );
        }
    }
}
