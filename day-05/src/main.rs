use itertools::Itertools;
use snafu::prelude::*;
use std::collections::BTreeMap;

const INPUT: &str = include_str!("../input");

#[snafu::report]
fn main() -> Result<(), Error> {
    let lowest = lowest_seed_location(INPUT)?;
    // Part 1: 251346198
    println!("{lowest}");

    let lowest = lowest_seed_range_location(INPUT)?;
    // Part 2: 72263011
    println!("{lowest}");

    Ok(())
}

fn lowest_seed_location(s: &str) -> Result<u64, Error> {
    let input = parse_input(s)?;

    input
        .seeds
        .iter()
        .map(|&seed| input.follow_through_maps(seed))
        .min()
        .context(NoSeedsSnafu)
}

fn lowest_seed_range_location(s: &str) -> Result<u64, Error> {
    let input = parse_input(s)?;

    input
        .seeds
        .chunks_exact(2)
        .flat_map(|range| range[0]..(range[0] + range[1]))
        .map(|seed| input.follow_through_maps(seed))
        .min()
        .context(NoSeedsSnafu)
}

#[derive(Debug, Snafu)]
enum Error {
    // TODO[SNAFU]: transparent
    #[snafu(context(false))]
    Parsing {
        source: ParseInputError,
    },

    NoSeeds,
}

struct Input {
    seeds: Vec<u64>,
    seed_to_soil: Map,
    soil_to_fertilizer: Map,
    fertilizer_to_water: Map,
    water_to_light: Map,
    light_to_temperature: Map,
    temperature_to_humidity: Map,
    humidity_to_location: Map,
}

impl Input {
    fn follow_through_maps(&self, key: u64) -> u64 {
        let Self {
            seed_to_soil,
            soil_to_fertilizer,
            fertilizer_to_water,
            water_to_light,
            light_to_temperature,
            temperature_to_humidity,
            humidity_to_location,
            ..
        } = self;

        let maps = [
            seed_to_soil,
            soil_to_fertilizer,
            fertilizer_to_water,
            water_to_light,
            light_to_temperature,
            temperature_to_humidity,
            humidity_to_location,
        ];

        maps.into_iter().fold(key, |key, map| map.get(key))
    }
}

fn parse_input(s: &str) -> Result<Input, ParseInputError> {
    use parse_input_error::*;

    let mut lines = s.lines().fuse();

    let seeds = lines.next().context(MissingSeedsSnafu)?;
    let mut seeds_parts = seeds.rsplitn(2, ':');
    let seeds = seeds_parts
        .next()
        .context(MissingSeedValuesSnafu { seeds })?;
    let seeds = seeds
        .split_ascii_whitespace()
        .map(|seed| seed.parse::<u64>().context(InvalidSeedSnafu { seed }))
        .collect::<Result<_, _>>()?;

    lines = lines.dropping(1); // Skip blank line

    let _header = lines.next().context(SeedToSoilMissingSnafu)?;
    let seed_to_soil = parse_map(&mut lines).context(SeedToSoilInvalidSnafu)?;

    let _header = lines.next().context(SoilToFertilizerMissingSnafu)?;
    let soil_to_fertilizer = parse_map(&mut lines).context(SoilToFertilizerInvalidSnafu)?;

    let _header = lines.next().context(FertilizerToWaterMissingSnafu)?;
    let fertilizer_to_water = parse_map(&mut lines).context(FertilizerToWaterInvalidSnafu)?;

    let _header = lines.next().context(WaterToLightMissingSnafu)?;
    let water_to_light = parse_map(&mut lines).context(WaterToLightInvalidSnafu)?;

    let _header = lines.next().context(LightToTemperatureMissingSnafu)?;
    let light_to_temperature = parse_map(&mut lines).context(LightToTemperatureInvalidSnafu)?;

    let _header = lines.next().context(TemperatureToHumidityMissingSnafu)?;
    let temperature_to_humidity =
        parse_map(&mut lines).context(TemperatureToHumidityInvalidSnafu)?;

    let _header = lines.next().context(HumidityToLocationMissingSnafu)?;
    let humidity_to_location = parse_map(&mut lines).context(HumidityToLocationInvalidSnafu)?;

    Ok(Input {
        seeds,
        seed_to_soil,
        soil_to_fertilizer,
        fertilizer_to_water,
        water_to_light,
        light_to_temperature,
        temperature_to_humidity,
        humidity_to_location,
    })
}

#[derive(Debug, Snafu)]
#[snafu(module)]
enum ParseInputError {
    MissingSeeds,

    MissingSeedValues {
        seeds: String,
    },

    InvalidSeed {
        source: std::num::ParseIntError,
        seed: String,
    },

    SeedToSoilMissing,

    SeedToSoilInvalid {
        source: ParseMapError,
    },

    SoilToFertilizerMissing,

    SoilToFertilizerInvalid {
        source: ParseMapError,
    },

    FertilizerToWaterMissing,

    FertilizerToWaterInvalid {
        source: ParseMapError,
    },

    WaterToLightMissing,

    WaterToLightInvalid {
        source: ParseMapError,
    },

    LightToTemperatureMissing,

    LightToTemperatureInvalid {
        source: ParseMapError,
    },

    TemperatureToHumidityMissing,

    TemperatureToHumidityInvalid {
        source: ParseMapError,
    },

    HumidityToLocationMissing,

    HumidityToLocationInvalid {
        source: ParseMapError,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Range {
    start: u64,
    length: u64,
}

struct Map(BTreeMap<u64, Range>);

impl Map {
    fn get(&self, key: u64) -> u64 {
        let candidate_ascending = self.0.range(key..).next();
        let candidate_descending = self.0.range(..key).next_back();

        let candidates = [candidate_ascending, candidate_descending]
            .into_iter()
            .flatten();

        for (&source_start, &destination_range) in candidates {
            let source_end = source_start + destination_range.length;
            if (source_start..source_end).contains(&key) {
                let delta = key - source_start;
                return destination_range.start + delta;
            }
        }

        key
    }
}

fn parse_map<'a>(lines: impl IntoIterator<Item = &'a str>) -> Result<Map, ParseMapError> {
    use parse_map_error::*;

    let mut map = BTreeMap::new();

    let lines = lines.into_iter().take_while(|l| !l.trim().is_empty());

    for line in lines {
        let mut parts = line.splitn(3, ' ');
        let destination_start = parts.next().context(DestinationMissingSnafu { line })?;
        let source_start = parts.next().context(SourceMissingSnafu { line })?;
        let length = parts.next().context(LengthMissingSnafu { line })?;

        let destination_start = destination_start
            .parse()
            .context(DestinationInvalidSnafu { line })?;
        let source_start = source_start.parse().context(SourceInvalidSnafu { line })?;
        let length = length.parse().context(LengthInvalidSnafu { line })?;

        let destination_range = Range {
            start: destination_start,
            length,
        };

        map.insert(source_start, destination_range);
    }

    Ok(Map(map))
}

#[derive(Debug, Snafu)]
#[snafu(module)]
enum ParseMapError {
    DestinationMissing {
        line: String,
    },

    SourceMissing {
        line: String,
    },

    LengthMissing {
        line: String,
    },

    DestinationInvalid {
        source: std::num::ParseIntError,
        line: String,
    },

    SourceInvalid {
        source: std::num::ParseIntError,
        line: String,
    },

    LengthInvalid {
        source: std::num::ParseIntError,
        line: String,
    },
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_INPUT_1: &str = include_str!("../example-input-1");

    #[test]
    #[snafu::report]
    fn example_1() -> Result<(), Error> {
        assert_eq!(35, lowest_seed_location(EXAMPLE_INPUT_1)?);
        Ok(())
    }

    #[test]
    #[snafu::report]
    fn example_2() -> Result<(), Error> {
        assert_eq!(46, lowest_seed_range_location(EXAMPLE_INPUT_1)?);
        Ok(())
    }
}
