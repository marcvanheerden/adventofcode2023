use std::collections::HashMap;
use tokio::sync::mpsc::Receiver;

const NUMBER_OF_CARDS: usize = 13;
const HAND_SIZE: usize = 5;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Card {
    C2,
    C3,
    C4,
    C5,
    C6,
    C7,
    C8,
    C9,
    T,
    J,
    Q,
    K,
    A,
}

impl From<char> for Card {
    fn from(c: char) -> Self {
        match c {
            '2' => Self::C2,
            '3' => Self::C3,
            '4' => Self::C4,
            '5' => Self::C5,
            '6' => Self::C6,
            '7' => Self::C7,
            '8' => Self::C8,
            '9' => Self::C9,
            'T' => Self::T,
            'J' => Self::J,
            'Q' => Self::Q,
            'K' => Self::K,
            'A' => Self::A,
            _ => panic!("invalid card"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum HandType {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOAK,
    FullHouse,
    FourOAK,
    FiveOAK,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Hand {
    handtype: HandType,
    values: [u8; HAND_SIZE],
}

impl Hand {
    fn new(s: &str, jokers: bool) -> Self {
        if s.len() != HAND_SIZE {
            panic!("invalid hand length");
        }

        if jokers {
            return Self::_new_jokers(s);
        }
        Self::_new_no_jokers(s)
    }

    fn _new_jokers(s: &str) -> Self {
        let new_s = s.replace('J', &Self::_joker_best_replacement(s).to_string());
        let mut hand = Self::_new_no_jokers(&new_s);

        let adjusted_values: Vec<_> = hand
            .values
            .iter()
            .zip(s.chars())
            .map(|(old_val, chr)| if chr == 'J' { 0 } else { old_val + 1 })
            .collect();

        hand.values = adjusted_values.try_into().unwrap();

        hand
    }

    fn _joker_best_replacement(s: &str) -> char {
        let mut counter = HashMap::new();
        for chr in s.chars() {
            if chr == 'J' {
                continue;
            }
            *counter.entry(chr).or_insert(0) += 1;
        }

        if let Some(&max) = counter.values().max() {
            if let Some((chr, _count)) = counter.into_iter().find(|(_chr, count)| *count == max) {
                return chr;
            }
        }

        'A'
    }

    fn _new_no_jokers(s: &str) -> Self {
        let mut card_value_counts = [0u8; NUMBER_OF_CARDS];
        let mut card_value_meta_counts = [0u8; HAND_SIZE];

        let values: Vec<u8> = s.chars().map(|c| Card::from(c) as u8).collect();

        for value in values.iter() {
            card_value_counts[*value as usize] += 1;
        }

        for count in card_value_counts.iter().filter(|&&count| count > 0) {
            card_value_meta_counts[*count as usize - 1] += 1;
        }

        let handtype = match card_value_meta_counts {
            [0, 0, 0, 0, 1] => HandType::FiveOAK,
            [1, 0, 0, 1, 0] => HandType::FourOAK,
            [0, 1, 1, 0, 0] => HandType::FullHouse,
            [2, 0, 1, 0, 0] => HandType::ThreeOAK,
            [1, 2, 0, 0, 0] => HandType::TwoPair,
            [3, 1, 0, 0, 0] => HandType::OnePair,
            [5, 0, 0, 0, 0] => HandType::HighCard,
            _ => unreachable!(),
        };

        Hand {
            handtype,
            values: values.try_into().unwrap(),
        }
    }
}

async fn calc_line(line: &str) -> (Hand, Hand, usize) {
    let (hand_str, bet_str) = line.split_once(' ').unwrap();

    (
        Hand::new(hand_str, false),
        Hand::new(hand_str, true),
        bet_str.parse().unwrap(),
    )
}

pub async fn solve(mut rx: Receiver<String>) {
    let mut tasks = Vec::new();

    while let Some(line) = rx.recv().await {
        let task = tokio::spawn(async move { calc_line(&line).await });
        tasks.push(task);
    }

    let mut hand_bet_pairs_part1 = Vec::new();
    let mut hand_bet_pairs_part2 = Vec::new();

    for task in tasks {
        if let Ok((hand1, hand2, bet)) = task.await {
            hand_bet_pairs_part1.push((hand1, bet));
            hand_bet_pairs_part2.push((hand2, bet));
        }
    }

    let part1 = tokio::spawn(async move {
        hand_bet_pairs_part1.sort_by(|(hand1, _), (hand2, _)| hand1.cmp(hand2));

        hand_bet_pairs_part1
            .iter()
            .enumerate()
            .map(|(idx, (_hand, bet))| (idx + 1) * bet)
            .sum::<usize>()
    });

    let part2 = tokio::spawn(async move {
        hand_bet_pairs_part2.sort_by(|(hand1, _), (hand2, _)| hand1.cmp(hand2));
        hand_bet_pairs_part2
            .iter()
            .enumerate()
            .map(|(idx, (_hand, bet))| (idx + 1) * bet)
            .sum::<usize>()
    });

    let part1 = part1.await.unwrap();
    let part2 = part2.await.unwrap();

    println!("Part 1: {part1}, Part 2: {part2}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jokers() {
        assert_eq!(
            Hand::new("32T3K", false).handtype,
            Hand::new("32T3K", true).handtype
        );
        let jhand = Hand::new("KTJJT", true);
        assert_eq!(jhand.handtype, HandType::FourOAK);
        assert_eq!(jhand.values, [12, 9, 0, 0, 9]);

        let ambig = Hand::new("KTKJT", true);
        assert_eq!(ambig.handtype, HandType::FullHouse);
        assert_eq!(ambig.values, [12, 9, 12, 0, 9]);

        assert!(Hand::new("JKKK2", true) < Hand::new("QQQQ2", true));
        assert!(Hand::new("A444A", true) < Hand::new("8J887", true));
        assert!(Hand::new("54225", true) > Hand::new("7A454", true));
        assert!(Hand::new("95JJ5", true) > Hand::new("T65T8", true));
        assert!(Hand::new("2J9KK", true) < Hand::new("6J699", true));
        assert!(Hand::new("J69A7", true) < Hand::new("KJJKK", true));

        let alljs = Hand::new("JJJJJ", true);
        assert_eq!(alljs.handtype, HandType::FiveOAK);
        assert_eq!(alljs.values, [0, 0, 0, 0, 0]);
    }
}
