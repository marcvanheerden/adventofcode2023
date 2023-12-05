use std::collections::HashMap;
use tokio::sync::mpsc::Receiver;

async fn calc_line(line: String, line_no: usize) -> (usize, usize) {
    let (_card, numbers) = line.split_once(':').expect("incorrect input format");
    let (winning, played) = numbers.split_once('|').expect("incorrect input format");

    let winning: Vec<&str> = winning.trim().split(' ').collect();
    let matches = played
        .trim()
        .split(' ')
        .filter(|s| winning.contains(s))
        .filter(|s| !s.is_empty())
        .count();

    (line_no, matches)
}

pub async fn solve(mut rx: Receiver<(usize, String)>) {
    let mut tasks = Vec::new();

    while let Some((line_no, line)) = rx.recv().await {
        let task = tokio::spawn(async move { calc_line(line, line_no).await });
        tasks.push(task);
    }

    let mut total_score = 0;
    let mut match_map = Vec::new();

    for task in tasks {
        if let Ok((card_no, matches)) = task.await {
            if match_map.len() < (card_no + 1) {
                match_map.resize(card_no + 1, 0);
            }
            match_map[card_no] = matches;

            if matches == 0 {
                continue;
            }

            total_score += 2usize.pow((matches - 1).try_into().unwrap());
        }
    }

    let cards = match_map.len();
    let mut memo = HashMap::new();
    let mut card_count = 0usize;

    for card in 0..=cards {
        card_count += card_recurse(card, &match_map, &mut memo);
    }

    println!("Part 1: {total_score} Part 2: {card_count}");
}

fn card_recurse(idx: usize, matches: &[usize], memo: &mut HashMap<usize, usize>) -> usize {
    if let Some(mem) = memo.get(&idx) {
        return *mem;
    }
    let mut total = 1;
    for card in (idx + 1)..=(idx + matches[idx]) {
        total += card_recurse(card, matches, memo);
    }
    memo.insert(idx, total);
    total
}
