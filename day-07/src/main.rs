use arrayvec::ArrayVec;
use itertools::Itertools;
use snafu::prelude::*;
use std::str::FromStr;

const INPUT: &str = include_str!("../input");

#[snafu::report]
fn main() -> Result<(), Error> {
    let total = total_winnings(INPUT)?;
    // Part 1: 253603890
    println!("{total}");

    Ok(())
}

fn total_winnings(s: &str) -> Result<usize, Error> {
    let mut input = s
        .lines()
        .map(|line| parse_line(line).context(InvalidLineSnafu { line }))
        .collect::<Result<Vec<_>, _>>()?;

    input.sort_by(|a, b| a.0.cmp(&b.0));

    Ok(input
        .iter()
        .enumerate()
        .map(|(idx, (_, bid))| (idx + 1) * bid)
        .sum())
}

#[derive(Debug, Snafu)]
enum Error {
    InvalidLine {
        source: ParseLineError,
        line: String,
    },
}

fn parse_line(l: &str) -> Result<(Hand, usize), ParseLineError> {
    use parse_line_error::*;

    let (hand, bid) = l.split_once(' ').context(MalformedSnafu)?;
    let hand = hand.parse().context(InvalidHandSnafu { hand })?;
    let bid = bid.parse().context(InvalidBidSnafu { bid })?;

    Ok((hand, bid))
}

#[derive(Debug, Snafu)]
#[snafu(module)]
enum ParseLineError {
    Malformed,

    InvalidHand {
        source: ParseHandError,
        hand: String,
    },

    InvalidBid {
        source: std::num::ParseIntError,
        bid: String,
    },
}

const HAND_SIZE: usize = 5;
type Cards = [Card; HAND_SIZE];

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Hand {
    HighCard(Cards),
    OnePair(Cards),
    TwoPair(Cards),
    ThreeOfAKind(Cards),
    FullHouse(Cards),
    FourOfAKind(Cards),
    FiveOfAKind(Cards),
}

impl FromStr for Hand {
    type Err = ParseHandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use parse_hand_error::*;
        use Hand::*;

        let cards = s
            .as_bytes()
            .iter()
            .map(|&b| b.try_into().context(InvalidCardSnafu { b }))
            .collect::<Result<ArrayVec<Card, HAND_SIZE>, _>>()?
            .into_inner()
            .ok()
            .context(NotFiveCardsSnafu)?;

        let mut categorized = cards;
        categorized.sort();

        let groups = categorized.iter().group_by(|&&c| c);
        let mut counts = groups
            .into_iter()
            .map(|(_, g)| g.count())
            .collect::<ArrayVec<_, HAND_SIZE>>();

        counts.sort();

        Ok(match &*counts {
            [5] => FiveOfAKind(cards),
            [1, 4] => FourOfAKind(cards),
            [2, 3] => FullHouse(cards),
            [1, 1, 3] => ThreeOfAKind(cards),
            [1, 2, 2] => TwoPair(cards),
            [1, 1, 1, 2] => OnePair(cards),
            _ => HighCard(cards),
        })
    }
}

#[derive(Debug, Snafu)]
#[snafu(module)]
enum ParseHandError {
    InvalidCard { source: ParseCardError, b: u8 },

    NotFiveCards,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Card {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl TryFrom<u8> for Card {
    type Error = ParseCardError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use Card::*;

        Ok(match value {
            b'2' => Two,
            b'3' => Three,
            b'4' => Four,
            b'5' => Five,
            b'6' => Six,
            b'7' => Seven,
            b'8' => Eight,
            b'9' => Nine,
            b'T' => Ten,
            b'J' => Jack,
            b'Q' => Queen,
            b'K' => King,
            b'A' => Ace,
            _ => return ParseCardSnafu.fail(),
        })
    }
}

#[derive(Debug, Snafu)]
struct ParseCardError;

#[cfg(test)]
mod test {
    use assert_matches::assert_matches;

    use super::*;

    const EXAMPLE_INPUT_1: &str = include_str!("../example-input-1");

    #[test]
    #[snafu::report]
    fn example_1() -> Result<(), Error> {
        assert_eq!(6440, total_winnings(EXAMPLE_INPUT_1)?);

        Ok(())
    }

    #[test]
    #[snafu::report]
    fn hand_categories() -> Result<(), ParseHandError> {
        use Hand::*;

        assert_matches!("AAAAA".parse()?, FiveOfAKind(..));
        assert_matches!("AA8AA".parse()?, FourOfAKind(..));
        assert_matches!("23332".parse()?, FullHouse(..));
        assert_matches!("TTT98".parse()?, ThreeOfAKind(..));
        assert_matches!("23432".parse()?, TwoPair(..));
        assert_matches!("A23A4".parse()?, OnePair(..));
        assert_matches!("23456".parse()?, HighCard(..));

        Ok(())
    }
}
