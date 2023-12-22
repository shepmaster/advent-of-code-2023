use core::fmt;
use std::{
    cmp,
    collections::{BTreeMap, BTreeSet},
};

use itertools::Itertools;
use petgraph::{graphmap::DiGraphMap, Direction};

const INPUT: &str = include_str!("../input");

fn main() {
    let bricks = safely_disintegratable_bricks(INPUT);
    // Part 1: 23 (wrong)
    // -> limited to 26 via `Iterator::zip`
    // -> Wasn't using `max`!
    //       : 573 (too high)
    // -> Child bricks may be one of multiple siblings of one parent
    // -> while being the only child of another parent.
    //       : 492
    println!("{bricks}");
}

fn safely_disintegratable_bricks(s: &str) -> usize {
    let mut id = 0;

    let mut input = s
        .lines()
        .map(|line| {
            let (s, e) = line.split_once('~').expect("Malformed");

            let parse_coord = |s: &str| -> Coord {
                let mut p = s.split(',');

                let parse_digit = |p: &mut std::str::Split<'_, _>| -> Dim {
                    let d = p.next().unwrap();
                    d.parse().unwrap()
                };

                let x = parse_digit(&mut p);
                let y = parse_digit(&mut p);
                let z = parse_digit(&mut p);

                (x, y, z)
            };

            let s = parse_coord(s);
            let e = parse_coord(e);

            let b = Brick { id, s, e };
            id += 1;
            b
        })
        .collect::<Vec<_>>();

    // eprintln!("[");
    // for i in &input {
    //     eprintln!("  {i:?},");
    // }
    // eprintln!("]");
    // eprintln!("=== Initial {}\n\n{}", input.len(), TowerViewX(&input));

    // Checking assumptions
    // for Brick { s, e } in &input {
    //     assert!(s.0 <= e.0);
    //     assert!(s.1 <= e.1);
    //     assert!(s.2 <= e.2);
    // }

    // Process the bricks from the bottom-up
    input.sort_by_key(|b| cmp::min(b.s.2, b.e.2));

    // eprintln!("[");
    // for i in &input {
    //     eprintln!("  {i:?},");
    // }
    // eprintln!("]");
    // eprintln!("=== Sorted {}\n\n{}", input.len(), TowerViewX(&input));

    let mut stacked = Vec::<Brick>::new();

    for to_stack in input {
        let mut max = 0;

        for coord in to_stack.footprint() {
            for sitting_brick in &stacked {
                if let Some(z) = sitting_brick.overlap_intersection(coord) {
                    max = cmp::max(z, max);
                }
            }
        }

        let fallen = to_stack.translate_to(max + 1); // 1 because everything has one height
        stacked.push(fallen);
    }

    // eprintln!("=== Stacked\n\n{}", TowerViewX(&stacked));

    // Build a tree of every brick. Child nodes support the
    // parents. The topmost parent can be removed; nothing relies on
    // them. Any node with two children can have the children removed;
    // the other child would continue supporting.

    let v = Volume::new(&stacked);

    let mut g = DiGraphMap::new();

    for (p_coord, p_id) in v.all() {
        let p = g.add_node(p_id);

        if let Some(c_id) = v.below(p_coord) {
            if p_id == c_id {
                // This is a tall piece and it sits atop itself
                continue;
            }

            let c = g.add_node(c_id);
            g.add_edge(p, c, "");
        }
    }

    // let dot = format!("{}", petgraph::dot::Dot::new(&g));
    // std::fs::write("./graph.dot", dot).unwrap();

    // Bricks at the top of the pile can always be removed
    let mut no_parent_bricks = BTreeSet::new();

    // Not just > 1 children... if one of those children is the sole
    // supporter of another, it cannot be removed. For example, C
    // cannot be removed because then Z would fall.
    //
    //  A      Z
    //  |      |
    //  |--    |
    //  | |    |
    //  B |- C-|
    let mut has_siblings = BTreeSet::new();
    let mut has_no_siblings = BTreeSet::new();

    for id in g.nodes() {
        use Direction::*;

        let n_parents = g.edges_directed(id, Incoming).take(1).count();

        if n_parents == 0 {
            no_parent_bricks.insert(id);
        }

        let children = g
            .edges_directed(id, Outgoing)
            .map(|(_, c, _)| c)
            .collect::<Vec<_>>();
        match &children[..] {
            &[] => {}
            &[c] => {
                has_no_siblings.insert(c);
            }
            c => has_siblings.extend(c),
        }
    }

    let n_always_siblings = has_siblings.difference(&has_no_siblings).count();

    no_parent_bricks.len() + n_always_siblings
}

type Id = u16;
type Dim = u16;
type Coord = (Dim, Dim, Dim);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Brick {
    s: Coord,
    e: Coord,
    id: Id,
}

// Always a line... hmm
impl Brick {
    #[cfg(test)]
    fn new(id: Id, s: Coord, e: Coord) -> Self {
        Self { id, s, e }
    }

    fn cubes(self) -> impl Iterator<Item = Coord> {
        let x = self.s.0..=self.e.0;
        let y = self.s.1..=self.e.1;
        let z = self.s.2..=self.e.2;

        x.cartesian_product(y)
            .cartesian_product(z)
            .map(|((x, y), z)| (x, y, z))
    }

    fn footprint(self) -> impl Iterator<Item = Coord> {
        let x = self.s.0..=self.e.0;
        let y = self.s.1..=self.e.1;
        let z = cmp::min(self.s.2, self.e.2);

        x.cartesian_product(y).map(move |(x, y)| (x, y, z))
    }

    fn overlap_intersection(self, coord: Coord) -> Option<Dim> {
        let xs = self.s.0..=self.e.0;
        let ys = self.s.1..=self.e.1;

        let (x, y, _) = coord;
        if xs.contains(&x) && ys.contains(&y) {
            let z = cmp::max(self.s.2, self.e.2);
            Some(z)
        } else {
            None
        }
    }

    fn translate_to(mut self, z: Dim) -> Brick {
        let lowest_z = cmp::min(self.s.2, self.e.2);
        let distance = lowest_z - z;

        self.s.2 -= distance;
        self.e.2 -= distance;

        self
    }
}

#[test]
fn brick_cubes() {
    let b = Brick::new(0, (1, 1, 8), (1, 1, 9));
    assert_eq!(2, b.cubes().count());

    let b = Brick::new(0, (2, 0, 5), (2, 2, 5));
    assert_eq!(3, b.cubes().count());
}

#[test]
fn brick_footprint() {
    let b = Brick::new(0, (1, 1, 8), (1, 1, 9));
    assert_eq!(1, b.footprint().count());

    let b = Brick::new(0, (2, 0, 5), (2, 2, 5));
    assert_eq!(3, b.footprint().count());
}

#[test]
fn brick_translate_to() {
    let b = Brick::new(0, (1, 1, 8), (1, 1, 9)).translate_to(5);
    assert_eq!(Brick::new(0, (1, 1, 5), (1, 1, 6)), b);
}

struct Volume {
    volume: BTreeMap<Coord, Id>,
    x_max: Dim,
    y_max: Dim,
    z_max: Dim,
}

impl Volume {
    fn new(bricks: &[Brick]) -> Self {
        let mut x_max = 0;
        let mut y_max = 0;
        let mut z_max = 0;

        let mut volume = BTreeMap::new();

        for b in bricks {
            let Brick { id, .. } = *b;

            for coord in b.cubes() {
                if let Some(old_id) = volume.insert(coord, id) {
                    let old_b = bricks.iter().find(|b| b.id == old_id).unwrap();
                    dbg!(old_b, b);
                    panic!("replaced {old_id} with {id} at {coord:?}");
                }

                x_max = cmp::max(x_max, coord.0);
                y_max = cmp::max(y_max, coord.1);
                z_max = cmp::max(z_max, coord.2);
            }
        }

        Self {
            volume,
            x_max,
            y_max,
            z_max,
        }
    }

    fn get(&self, coord: Coord) -> Option<Id> {
        self.volume.get(&coord).copied()
    }

    fn all(&self) -> impl Iterator<Item = (Coord, Id)> + '_ {
        let Self {
            x_max,
            y_max,
            z_max,
            ..
        } = *self;

        let xs = 0..=x_max;
        let ys = 0..=y_max;
        let zs = 0..=z_max;

        let coords = xs
            .cartesian_product(ys)
            .cartesian_product(zs)
            .map(|((x, y), z)| (x, y, z));

        coords.flat_map(|c| Some((c, self.get(c)?)))
    }

    fn below(&self, (x, y, z): Coord) -> Option<Id> {
        let z = z.checked_sub(1)?;

        let coord = (x, y, z);

        self.get(coord)
    }
}

struct TowerViewX<'a>(&'a [Brick]);

impl fmt::Display for TowerViewX<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let volume = Volume::new(self.0);

        for z in (0..=volume.z_max).rev() {
            for x in 0..=volume.x_max {
                let mut ys = (0..=volume.y_max)
                    .flat_map(|y| volume.get((x, y, z)))
                    .collect::<Vec<_>>();

                ys.sort();
                ys.dedup();

                match &ys[..] {
                    [] => '.'.fmt(f)?,
                    &[l] => id_to_char(l).fmt(f)?,
                    _ => '?'.fmt(f)?,
                }
            }
            "\n".fmt(f)?;
        }

        Ok(())
    }
}

fn id_to_char(id: Id) -> char {
    let id = u8::try_from(id)
        .ok()
        .and_then(|id| if id < 26 { Some(id) } else { None });

    match id {
        Some(id) => char::from(b'A' + id),
        None => '*',
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_INPUT_1: &str = include_str!("../example-input-1");

    #[test]
    fn example_1() {
        assert_eq!(5, safely_disintegratable_bricks(EXAMPLE_INPUT_1));
    }
}
