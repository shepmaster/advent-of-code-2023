use petgraph::graphmap::UnGraphMap;
use std::{
    collections::{BTreeMap, BTreeSet},
    ops::RangeInclusive,
};

const INPUT: &str = include_str!("../input");

fn main() {
    let length = longest_hike(INPUT);
    assert_eq!(2298, length);
    println!("{length}");

    let length = longest_hike_even_uphill(INPUT);
    assert!(length > 6066);
    // Was computing shortest path between the intersections, not the
    // point-to-point distance
    assert_eq!(6602, length);
    println!("{length}");
}

fn longest_hike(s: &str) -> usize {
    use Tile::*;

    let (map, bounds) = build_map(s);

    let (start, end) = find_entry_and_exit(&map, &bounds);

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

fn longest_hike_even_uphill(s: &str) -> usize {
    let (map, bounds) = build_map(s);
    let (start, end) = find_entry_and_exit(&map, &bounds);

    let mut g = UnGraphMap::new();

    for (&coord, tile) in &map {
        if !tile.passable() {
            continue;
        }

        let me = g.add_node(coord);

        let neighbors = [
            bounds.up(coord),
            bounds.right(coord),
            bounds.down(coord),
            bounds.left(coord),
        ]
        .into_iter()
        .flatten();

        for neighbor in neighbors {
            if map.get(&neighbor).map_or(false, |t| t.passable()) {
                let n = g.add_node(neighbor);
                g.add_edge(me, n, 1);
            }
        }
    }

    // let dot = format!("{:?}", petgraph::dot::Dot::new(&g));
    // std::fs::write("./graph.dot", dot).unwrap();

    // Find every node that has two edges. Delete that node, summing
    // up the weights (steps) from each of the two edges and
    // connecting the outsides.
    //
    // A --1-- B --3-- C
    //
    // becomes
    //
    // A --4-- C
    //
    // This is probably the least efficient way of doing this
    loop {
        let q = g
            .nodes()
            .flat_map(|n| {
                let e = g.edges(n).collect::<Vec<_>>();
                <[_; 2]>::try_from(e).map(|es| (n, es))
            })
            .next();

        let Some((n, [a, b])) = q else { break };

        let (aa, ab, &aw) = a;
        let (ba, bb, &bw) = b;

        // What order are these nodes in?
        let a = if aa == n { ab } else { aa };
        let b = if ba == n { bb } else { ba };

        g.remove_node(n);
        g.add_edge(a, b, aw + bw);
    }

    // let dot = format!("{:?}", petgraph::dot::Dot::new(&g2));
    // std::fs::write("./graph2.dot", dot).unwrap();

    // Use the same brute force algorithm, but with the compressed graph

    let mut queue = vec![(start, vec![(start, 0)])];
    let mut successes = vec![];

    while let Some((coord, visited)) = queue.pop() {
        // Reached the end
        if coord == end {
            successes.push(visited);
            continue;
        }

        for (a, b, &steps) in g.edges(coord) {
            let other = if a == coord { b } else { a };

            if visited.iter().any(|&(c, _)| c == other) {
                // Already been here
                continue;
            }

            let mut visited = visited.clone();
            visited.push((other, steps));

            queue.push((other, visited));
        }
    }

    let max_len = successes
        .iter()
        .map(|s| s.iter().map(|(_, p)| p).sum::<usize>())
        .max()
        .expect("No paths found");

    max_len
}

type Dim = usize;
type Coord = (Dim, Dim);
type Map = BTreeMap<Coord, Tile>;

fn build_map(s: &str) -> (Map, Bounds) {
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

    let bounds = Bounds {
        x: 0..=x_max,
        y: 0..=y_max,
    };

    (map, bounds)
}

fn find_entry_and_exit(map: &Map, bounds: &Bounds) -> (Coord, Coord) {
    use Tile::*;

    let entry_y = *bounds.y.start();
    let entry_x = bounds
        .x
        .clone()
        .find(|&x| map[&(x, entry_y)] == Path)
        .expect("Entry tile missing");
    let entry = (entry_x, entry_y);

    let exit_y = *bounds.y.end();
    let exit_x = bounds
        .x
        .clone()
        .find(|&x| map[&(x, exit_y)] == Path)
        .expect("Exit tile missing");
    let exit = (exit_x, exit_y);

    (entry, exit)
}

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

impl Tile {
    fn passable(self) -> bool {
        !matches!(self, Tile::Forest)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_INPUT_1: &str = include_str!("../example-input-1");

    #[test]
    fn example_1() {
        assert_eq!(94, longest_hike(EXAMPLE_INPUT_1));
    }

    #[test]
    fn example_2() {
        assert_eq!(154, longest_hike_even_uphill(EXAMPLE_INPUT_1));
    }
}
