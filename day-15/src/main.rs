use snafu::prelude::*;

const INPUT: &str = include_str!("../input");

#[snafu::report]
fn main() -> Result<(), Error> {
    let sum = sum_of_hashes(INPUT);
    // Part 1: 511513 (too high)
    // -> Forgot to remove the newline
    //         511343
    println!("{sum}");

    let sum = sum_of_focal_power(INPUT)?;
    // Part 2: 294474
    println!("{sum}");

    Ok(())
}

fn sum_of_hashes(s: &str) -> u64 {
    instructions(s).map(hash).map(u64::from).sum()
}

fn sum_of_focal_power(s: &str) -> Result<usize, Error> {
    let mut boxes = vec![Vec::new(); 256];

    for instruction in instructions(s) {
        if let Some((label, focal_length)) = instruction.split_once('=') {
            let focal_length = focal_length
                .parse::<usize>()
                .context(FocalLengthSnafu { focal_length })?;

            let hash = usize::from(hash(label));
            let the_box = &mut boxes[hash];

            match the_box.iter_mut().find(|(l, _)| *l == label) {
                Some(slot) => slot.1 = focal_length,
                None => the_box.push((label, focal_length)),
            }
        } else if let Some(label) = instruction.strip_suffix('-') {
            let hash = usize::from(hash(label));
            let the_box = &mut boxes[hash];

            the_box.retain(|(l, _)| *l != label);
        } else {
            return UnknownSnafu { instruction }.fail();
        }
    }

    // for (i, b) in boxes.iter().enumerate() {
    //     if b.is_empty() {
    //         continue;
    //     }
    //     eprintln!("{i:3}: {b:?}");
    // }

    let sum = boxes
        .into_iter()
        .enumerate()
        .map(|(box_idx, the_box)| {
            the_box
                .into_iter()
                .enumerate()
                .map(|(slot_idx, (_, focal_length))| (box_idx + 1) * (slot_idx + 1) * focal_length)
                .sum::<usize>()
        })
        .sum();

    Ok(sum)
}

#[derive(Debug, Snafu)]
enum Error {
    FocalLength {
        source: std::num::ParseIntError,
        focal_length: String,
    },

    Unknown {
        instruction: String,
    },
}

fn instructions(s: &str) -> impl Iterator<Item = &str> {
    s.split(',').map(str::trim)
}

fn hash(s: &str) -> u16 {
    s.as_bytes()
        .iter()
        .fold(0, |hash, &byte| ((hash + u16::from(byte)) * 17) % 256)
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_INPUT_1: &str = include_str!("../example-input-1");

    #[test]
    fn example_hash() {
        assert_eq!(52, hash("HASH"));
    }

    #[test]
    fn example_1() {
        assert_eq!(1320, sum_of_hashes(EXAMPLE_INPUT_1));
    }

    #[test]
    #[snafu::report]
    fn example_2() -> Result<(), Error> {
        assert_eq!(145, sum_of_focal_power(EXAMPLE_INPUT_1)?);

        Ok(())
    }
}
