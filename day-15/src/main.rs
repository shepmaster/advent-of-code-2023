const INPUT: &str = include_str!("../input");

fn main() {
    let sum = sum_of_hashes(INPUT);
    // Part 1: 511513 (too high)
    // -> Forgot to remove the newline
    //         511343
    println!("{sum}");
}

fn sum_of_hashes(s: &str) -> u64 {
    s.split(',').map(str::trim).map(hash).sum()
}

fn hash(s: &str) -> u64 {
    s.as_bytes()
        .iter()
        .fold(0, |hash, &byte| ((hash + u64::from(byte)) * 17) % 256)
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
}
