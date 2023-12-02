use snafu::prelude::*;
use std::{cmp, str::FromStr};

const INPUT: &str = include_str!("../input");

#[snafu::report]
fn main() -> Result<(), Error> {
    let sum = sum_of_possible_game_ids(INPUT)?;
    // Part 1: 2283
    println!("{sum}");

    let sum = sum_of_power_of_minimum_cubes(INPUT)?;
    // Part 2: 78669
    println!("{sum}");

    Ok(())
}

type GameId = u64;

const MAX: Draw = Draw {
    red: 12,
    green: 13,
    blue: 14,
};

fn sum_of_possible_game_ids(s: &str) -> Result<u64, Error> {
    let validated_games = parse_games(s).map(|g| {
        let (id, draws) = g?;
        let valid =
            itertools::process_results(draws, |mut draws| draws.all(|draw| MAX.can_fit(draw)))?;

        Ok((id, valid))
    });

    itertools::process_results(validated_games, |games| {
        games.filter_map(|(id, valid)| valid.then_some(id)).sum()
    })
}

fn sum_of_power_of_minimum_cubes(s: &str) -> Result<u64, Error> {
    let minimum_games = parse_games(s).map(|g| {
        let (id, draws) = g?;
        let minimum = itertools::process_results(draws, |draws| draws.reduce(Draw::minimum))?
            .unwrap_or_default();

        Ok((id, minimum))
    });

    itertools::process_results(minimum_games, |games| games.map(|(_, d)| d.power()).sum())
}

#[allow(clippy::needless_lifetimes)]
fn parse_games<'a>(
    s: &'a str,
) -> impl Iterator<Item = Result<(GameId, impl Iterator<Item = Result<Draw, Error>> + 'a), Error>> + 'a
{
    s.lines().map(|line| {
        let mut parts = line.splitn(2, ':');
        let id = parts.next().context(MissingIdSnafu { line })?;
        let draws = parts.next().context(MissingDrawsSnafu { line })?;

        let id = id.trim_start_matches("Game ");
        let id = id.parse().context(InvalidIdSnafu { line, id })?;

        let draws = draws
            .split(';')
            .map(move |draw| Draw::from_str(draw).context(InvalidDrawSnafu { line, draw }));

        Ok((id, draws))
    })
}

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display(r#"The line "{line}" had no game ID"#))]
    MissingId { line: String },

    #[snafu(display(r#"The line "{line}" had no color draws"#))]
    MissingDraws { line: String },

    #[snafu(display(r#"The line "{line}" had an invalid game ID "{id}""#))]
    InvalidId {
        source: std::num::ParseIntError,
        line: String,
        id: String,
    },

    #[snafu(display(r#"The line "{line}" had an invalid color draw "{draw}""#))]
    InvalidDraw {
        source: ParseDrawError,
        line: String,
        draw: String,
    },
}

#[derive(Debug, Copy, Clone, Default)]
struct Draw {
    red: u64,
    green: u64,
    blue: u64,
}

impl Draw {
    fn can_fit(&self, subset: Draw) -> bool {
        self.red >= subset.red && self.green >= subset.green && self.blue >= subset.blue
    }

    fn minimum(self, other: Self) -> Self {
        Self {
            red: cmp::max(self.red, other.red),
            green: cmp::max(self.green, other.green),
            blue: cmp::max(self.blue, other.blue),
        }
    }

    fn power(self) -> u64 {
        self.red * self.green * self.blue
    }
}

impl FromStr for Draw {
    type Err = ParseDrawError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use parse_draw_error::*;

        let mut this = Self::default();

        for component in s.split(',') {
            let mut parts = component.trim().splitn(2, ' ');

            let count = parts.next().context(MissingCountSnafu { component })?;
            let color = parts.next().context(MissingColorSnafu { component })?;

            let count = count
                .parse()
                .context(InvalidCountSnafu { component, count })?;
            match color {
                "red" => this.red = count,
                "blue" => this.blue = count,
                "green" => this.green = count,
                color => return InvalidColorSnafu { component, color }.fail(),
            }
        }

        Ok(this)
    }
}

#[derive(Debug, Snafu)]
#[snafu(module)]
enum ParseDrawError {
    #[snafu(display(r#"The component "{component}" was missing the count"#))]
    MissingCount { component: String },

    #[snafu(display(r#"The component "{component}" was missing the color"#))]
    MissingColor { component: String },

    #[snafu(display(r#"The component "{component}" had an invalid count "{count}""#))]
    InvalidCount {
        source: std::num::ParseIntError,
        component: String,
        count: String,
    },

    #[snafu(display(r#"The component "{component}" had an invalid color "{color}""#))]
    InvalidColor { component: String, color: String },
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_INPUT_1: &str = include_str!("../example-input-1");

    #[test]
    #[snafu::report]
    fn example_1() -> Result<(), Error> {
        assert_eq!(8, sum_of_possible_game_ids(EXAMPLE_INPUT_1)?);

        Ok(())
    }

    #[test]
    #[snafu::report]
    fn example_2() -> Result<(), Error> {
        assert_eq!(2286, sum_of_power_of_minimum_cubes(EXAMPLE_INPUT_1)?);

        Ok(())
    }
}
