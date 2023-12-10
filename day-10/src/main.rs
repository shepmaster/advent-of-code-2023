use snafu::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

const INPUT: &str = include_str!("../input");

#[snafu::report]
fn main() -> Result<(), Error> {
    let distance = furthest_distance_from_start(INPUT)?;
    // Part 1: 6697
    println!("{distance}");

    Ok(())
}

type Coord = (usize, usize);

fn furthest_distance_from_start(s: &str) -> Result<usize, Error> {
    let mut map = BTreeMap::new();

    for (y, line) in s.lines().enumerate() {
        for (x, c) in line.chars().enumerate() {
            if let Some(p) = Pipe::try_from_char(c).context(PipeSnafu { x, y, c })? {
                map.insert((x, y), p);
            }
        }
    }

    let (&start, &p) = map
        .iter()
        .find(|&(_, &p)| p == Pipe::Start)
        .context(MissingStartSnafu)?;

    let mut to_visit = BTreeSet::new();
    let mut visited = BTreeSet::new();

    to_visit.insert((start, p));

    while let Some((start, p)) = to_visit.pop_first() {
        for (c, d) in p.outgoing(start) {
            if let Some(&p) = map.get(&c) {
                if p.compatible(d) && visited.insert(c) {
                    to_visit.insert((c, p));
                }
            }
        }
    }

    Ok(visited.len() / 2)
}

#[derive(Debug, Snafu)]
enum Error {
    Pipe {
        source: ParsePipeError,
        x: usize,
        y: usize,
        c: char,
    },

    MissingStart,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Direction {
    Up,
    Right,
    Down,
    Left,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Pipe {
    Start,
    NorthSouth,
    EastWest,
    NorthEast,
    NorthWest,
    SouthWest,
    SouthEast,
}

impl Pipe {
    fn try_from_char(value: char) -> Result<Option<Self>, ParsePipeError> {
        use Pipe::*;

        Ok(Some(match value {
            'S' => Start,
            '|' => NorthSouth,
            '-' => EastWest,
            'L' => NorthEast,
            'J' => NorthWest,
            '7' => SouthWest,
            'F' => SouthEast,
            '.' => return Ok(None),
            _ => return ParsePipeSnafu.fail(),
        }))
    }

    fn outgoing(self, (x, y): Coord) -> impl Iterator<Item = (Coord, Direction)> {
        use Direction::*;
        use Pipe::*;

        let xl = x.checked_sub(1);
        let xr = x.checked_add(1);
        let yu = y.checked_sub(1);
        let yd = y.checked_add(1);

        let xx = Some(x);
        let yy = Some(y);

        let u = (xx, yu, Up);
        let r = (xr, yy, Right);
        let d = (xx, yd, Down);
        let l = (xl, yy, Left);

        let choices = match self {
            Start => vec![u, r, d, l],
            NorthSouth => vec![u, d],
            EastWest => vec![l, r],
            NorthEast => vec![u, r],
            NorthWest => vec![u, l],
            SouthWest => vec![d, l],
            SouthEast => vec![d, r],
        };

        choices
            .into_iter()
            .flat_map(|(x, y, d)| Some(((x?, y?), d)))
    }

    fn compatible(self, dir: Direction) -> bool {
        use Direction::*;
        use Pipe::*;

        // The direction we take coming into the pipe
        #[allow(clippy::match_like_matches_macro)] // I like the current spacing
        match (self, dir) {
            (Start, _) => true,

            (NorthSouth, Up | Down) => true,

            (EastWest, Right | Left) => true,

            (NorthEast, Down | Left) => true,

            (NorthWest, Right | Down) => true,

            (SouthWest, Up | Right) => true,

            (SouthEast, Up | Left) => true,

            _ => false,
        }
    }
}

#[derive(Debug, Snafu)]
struct ParsePipeError;

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_INPUT_1: &str = include_str!("../example-input-1");
    const EXAMPLE_INPUT_1B: &str = include_str!("../example-input-1b");
    const EXAMPLE_INPUT_2: &str = include_str!("../example-input-2");
    const EXAMPLE_INPUT_2B: &str = include_str!("../example-input-2b");

    #[test]
    #[snafu::report]
    fn example_1() -> Result<(), Error> {
        assert_eq!(4, furthest_distance_from_start(EXAMPLE_INPUT_1)?);
        assert_eq!(4, furthest_distance_from_start(EXAMPLE_INPUT_1B)?);

        Ok(())
    }

    #[test]
    #[snafu::report]
    fn example_2() -> Result<(), Error> {
        assert_eq!(8, furthest_distance_from_start(EXAMPLE_INPUT_2)?);
        assert_eq!(8, furthest_distance_from_start(EXAMPLE_INPUT_2B)?);

        Ok(())
    }
}
