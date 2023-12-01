use rustc_hash::FxHashMap;
use tokio::sync::mpsc::Receiver;

struct WordTrace<'a> {
    start_pos: usize,
    word: &'a str,
    completed: usize,
    length: usize,
}

async fn calc_line2(line: String) -> (u32, u32) {
    let mut total_part1 = 0;
    let mut total_part2 = 0;

    let number_pairs = [
        ("one", '1'),
        ("two", '2'),
        ("three", '3'),
        ("four", '4'),
        ("five", '5'),
        ("six", '6'),
        ("seven", '7'),
        ("eight", '8'),
        ("nine", '9'),
        ("1", '1'),
        ("2", '2'),
        ("3", '3'),
        ("4", '4'),
        ("5", '5'),
        ("6", '6'),
        ("7", '7'),
        ("8", '8'),
        ("9", '9'),
        ("0", '0'),
    ];

    let numbers: FxHashMap<&str, char> = number_pairs.into_iter().collect();
    let mut first_part1: Option<char> = None;
    let mut last_part1: char = '~';
    let mut first_part2: Option<char> = None;
    let mut last_part2: char = '~';

    let mut candidate_words = Vec::new();

    for (pos, chr) in line.chars().enumerate() {
        // part1 
        if chr.is_ascii_digit() {
            if first_part1.is_none() {
                first_part1 = Some(chr);
            }
            last_part1 = chr;
        }

        // step to next character candidate words 
        candidate_words = candidate_words
            .into_iter()
            .filter_map(|wt: WordTrace| {
                if wt.word.chars().nth(wt.completed).unwrap() == chr {
                    Some(WordTrace {
                        start_pos: wt.start_pos,
                        word: wt.word,
                        completed: wt.completed + 1,
                        length: wt.length,
                    })
                } else {
                    None
                }
            })
            .collect();

        // check if new word traces are starting
        for word in numbers.keys() {
            if word.chars().next().unwrap() == chr {
                candidate_words.push(WordTrace {
                    start_pos: pos,
                    word,
                    completed: 1,
                    length: word.len(),
                });
            }
        }
        
        // if a candidate word is completed, update first/last words
        for word_trace in candidate_words
            .iter()
            .filter(|wt| wt.completed >= wt.length)
        {
            // can only be length of 0 or 1 due to the words in number_pairs.keys()
            let digit = numbers.get(word_trace.word);
            if first_part2.is_none() {
                first_part2 = digit.copied();
            }
            last_part2 = *digit.unwrap();
        }

        // remove completed candidate words 
        candidate_words.retain(|wt| wt.completed < wt.length);
    }

    if let Some(first_digit) = first_part1 {
        if let Ok(diff) = format!("{first_digit}{last_part1}").parse::<u32>() {
            total_part1 += diff;
        }
    }

    if let Some(first_digit) = first_part2 {
        if let Ok(diff) = format!("{first_digit}{last_part2}").parse::<u32>() {
            total_part2 += diff;
        }
    }

    (total_part1, total_part2)
}

pub async fn solve(mut rx: Receiver<String>) {
    let mut total_part1 = 0;
    let mut total_part2 = 0;

    let mut tasks = Vec::new();

    while let Some(line) = rx.recv().await {
        let task = tokio::spawn(async move { calc_line2(line).await });

        tasks.push(task);
    }

    for task in tasks {
        if let Ok((delta1, delta2)) = task.await {
            total_part1 += delta1;
            total_part2 += delta2;
        }
    }

    println!("Part 1: {total_part1} Part 2: {total_part2}");
}
