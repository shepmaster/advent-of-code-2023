use snafu::prelude::*;
use std::{
    cmp,
    collections::{BTreeMap, BTreeSet},
    ops::RangeInclusive,
};

const INPUT: &str = include_str!("../input");

#[snafu::report]
fn main() -> Result<(), Error> {
    let distance = furthest_distance_from_start(INPUT)?;
    // Part 1: 6697
    println!("{distance}");

    let area = area_inside_loop(INPUT)?;
    // Part 1: 423
    println!("{area}");

    Ok(())
}

type Coord = (usize, usize);
type Map = BTreeMap<Coord, Pipe>;

fn furthest_distance_from_start(s: &str) -> Result<usize, Error> {
    let map = build_map(s)?;

    let visited = build_path(&map)?;

    Ok(visited.len() / 2)
}

fn area_inside_loop(s: &str) -> Result<usize, Error> {
    let map = build_map(s)?;
    let path = build_path(&map)?;

    let (x_range, y_range) = find_bounds(&map).expect("The map had no entries");

    #[derive(Debug, Copy, Clone)]
    enum Space {
        Inside,
        Outside,
    }

    impl Space {
        fn toggle(self) -> Self {
            match self {
                Outside => Inside,
                Inside => Outside,
            }
        }
    }

    #[derive(Debug, Copy, Clone)]
    enum State {
        Known(Space),
        Wall(
            /// Which pipe we are looking for to toggle the state
            Pipe,
            Space,
        ),
    }

    use Pipe::*;
    use Space::*;
    use State::*;

    let mut count = 0;

    for y in y_range {
        let mut state = State::Known(Space::Outside);

        for x in x_range.clone() {
            let c = (x, y);
            let v = path.get(&c);

            match (v, state) {
                // Vertical wall
                (Some(&NorthSouth), Known(s)) => {
                    state = Known(s.toggle());
                }

                // Horizontal wall
                (Some(&EastWest), _) => {
                    // Do nothing
                }

                // Entering a wall
                (Some(&SouthEast), Known(s)) => {
                    state = Wall(NorthWest, s);
                }
                (Some(&NorthEast), Known(s)) => {
                    state = Wall(SouthWest, s);
                }

                // Exiting a wall that keeps going
                //        |    |
                // --> ┌--┘ or └--┐
                //     |          |
                (Some(&SouthWest), Wall(SouthWest, s)) => {
                    state = Known(s.toggle());
                }
                (Some(&NorthWest), Wall(NorthWest, s)) => {
                    state = Known(s.toggle());
                }

                // Exiting a wall that doubles back
                //     |  |
                // --> └--┘ or ┌--┐
                //             |  |
                (Some(&SouthWest), Wall(NorthWest, s)) => {
                    state = Known(s);
                }
                (Some(&NorthWest), Wall(SouthWest, s)) => {
                    state = Known(s);
                }

                // Blank space
                (None, Known(Inside)) => {
                    count += 1;
                }
                (None, Known(Outside)) => {
                    // Do nothing
                }

                o => panic!("Bad logic {o:?}"),
            }
        }
    }

    Ok(count)
}

fn build_map(s: &str) -> Result<Map, Error> {
    let mut map = BTreeMap::new();

    for (y, line) in s.lines().enumerate() {
        for (x, c) in line.chars().enumerate() {
            if let Some(p) = Pipe::try_from_char(c).context(PipeSnafu { x, y, c })? {
                map.insert((x, y), p);
            }
        }
    }

    Ok(map)
}

fn find_bounds(map: &Map) -> Option<(RangeInclusive<usize>, RangeInclusive<usize>)> {
    let mut x_min = None;
    let mut x_max = None;
    let mut y_min = None;
    let mut y_max = None;

    for &(x, y) in map.keys() {
        x_min = Some(x_min.map_or(x, |o| cmp::min(x, o)));
        x_max = Some(x_max.map_or(x, |o| cmp::max(x, o)));
        y_min = Some(y_min.map_or(y, |o| cmp::min(y, o)));
        y_max = Some(y_max.map_or(y, |o| cmp::max(y, o)));
    }

    Some((x_min?..=x_max?, y_min?..=y_max?))
}

fn build_path(map: &Map) -> Result<Map, Error> {
    let (&start_c, _) = map
        .iter()
        .find(|&(_, &p)| p == Pipe::Start)
        .context(MissingStartSnafu)?;

    let mut to_visit = BTreeSet::new();
    let mut visited = BTreeMap::new();

    let start_p = calculate_start_pipe(start_c, map);

    to_visit.insert((start_c, start_p));

    while let Some((start_c, start_p)) = to_visit.pop_first() {
        for (c, d) in start_p.outgoing(start_c) {
            if let Some(&p) = map.get(&c) {
                if p.compatible(d) && visited.insert(c, p).is_none() {
                    to_visit.insert((c, p));
                }
            }
        }
    }

    // Overwrite the start pipe with the concrete one we determined
    visited.insert(start_c, start_p);

    Ok(visited)
}

fn calculate_start_pipe(coord: Coord, map: &Map) -> Pipe {
    use Direction::*;
    use Pipe::*;

    let u = up(coord);
    let r = right(coord);
    let d = down(coord);
    let l = left(coord);

    let neighbors = [u, r, d, l]
        .into_iter()
        .flat_map(|x| {
            let (c, d) = x?;
            let &p = map.get(&c)?;

            if p.compatible(d) {
                Some(d)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    match &neighbors[..] {
        [Up, Down] | [Down, Up] => NorthSouth,
        [Left, Right] | [Right, Left] => EastWest,
        [Up, Right] | [Right, Up] => NorthEast,
        [Up, Left] | [Left, Up] => NorthWest,
        [Down, Left] | [Left, Down] => SouthWest,
        [Down, Right] | [Right, Down] => SouthEast,
        _ => unreachable!(),
    }
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

    fn outgoing(self, coord: Coord) -> impl Iterator<Item = (Coord, Direction)> {
        use Pipe::*;

        let u = up(coord);
        let r = right(coord);
        let d = down(coord);
        let l = left(coord);

        let choices = match self {
            Start => vec![u, r, d, l],
            NorthSouth => vec![u, d],
            EastWest => vec![l, r],
            NorthEast => vec![u, r],
            NorthWest => vec![u, l],
            SouthWest => vec![d, l],
            SouthEast => vec![d, r],
        };

        choices.into_iter().flatten()
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

fn up((x, y): Coord) -> Option<(Coord, Direction)> {
    Some(((x, y.checked_sub(1)?), Direction::Up))
}

fn down((x, y): Coord) -> Option<(Coord, Direction)> {
    Some(((x, y.checked_add(1)?), Direction::Down))
}

fn left((x, y): Coord) -> Option<(Coord, Direction)> {
    Some(((x.checked_sub(1)?, y), Direction::Left))
}

fn right((x, y): Coord) -> Option<(Coord, Direction)> {
    Some(((x.checked_add(1)?, y), Direction::Right))
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_INPUT_1: &str = include_str!("../example-input-1");
    const EXAMPLE_INPUT_1B: &str = include_str!("../example-input-1b");
    const EXAMPLE_INPUT_2: &str = include_str!("../example-input-2");
    const EXAMPLE_INPUT_2B: &str = include_str!("../example-input-2b");
    const EXAMPLE_INPUT_3: &str = include_str!("../example-input-3");
    const EXAMPLE_INPUT_3B: &str = include_str!("../example-input-3b");
    const EXAMPLE_INPUT_4: &str = include_str!("../example-input-4");
    const EXAMPLE_INPUT_5: &str = include_str!("../example-input-5");

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

    #[test]
    #[snafu::report]
    fn example_3() -> Result<(), Error> {
        assert_eq!(4, area_inside_loop(EXAMPLE_INPUT_3)?);
        assert_eq!(4, area_inside_loop(EXAMPLE_INPUT_3B)?);

        Ok(())
    }

    #[test]
    #[snafu::report]
    fn example_4() -> Result<(), Error> {
        assert_eq!(8, area_inside_loop(EXAMPLE_INPUT_4)?);

        Ok(())
    }

    #[test]
    #[snafu::report]
    fn example_5() -> Result<(), Error> {
        assert_eq!(10, area_inside_loop(EXAMPLE_INPUT_5)?);

        Ok(())
    }
}
