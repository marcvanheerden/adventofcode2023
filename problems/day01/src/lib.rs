use rustc_hash::FxHashMap;
use tokio::sync::mpsc::Receiver;

struct WordTrace<'a> {
    start_pos: usize,
    word: &'a str,
    completed: usize,
    length: usize,
}

async fn find_number_part1<I>(iter: I) -> Option<char>
where I: Iterator<Item = char>,
{
    for chr in iter {
        if chr.is_ascii_digit() {
            return Some(chr)
        }
    }
    None
}

async fn find_number_part2<I>(iter: I, numbers: &FxHashMap<&str, char>, reverse_words: bool) -> Option<char> 
where I: Iterator<Item = char>,
{
    
    let mut candidate_words = Vec::new();
    
    for (pos, chr) in iter.enumerate() {
        
        // step to next character candidate words 
        candidate_words = candidate_words
            .into_iter()
            .filter_map(|wt: WordTrace| {
                if (!reverse_words & (wt.word.chars().nth(wt.completed).unwrap() == chr)) | 
                    (reverse_words & (wt.word.chars().rev().nth(wt.completed).unwrap() == chr)) {
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
            if (!reverse_words & (word.chars().next().unwrap() == chr)) | 
                 (reverse_words & (word.chars().rev().next().unwrap() == chr)) {
                candidate_words.push(WordTrace {
                    start_pos: pos,
                    word,
                    completed: 1, //TODO: reverse
                    length: word.len(),
                });
            }
        }

        // if a candidate word is completed, update first/last words
        for word_trace in candidate_words
            .iter()
            .filter(|wt| wt.completed >= wt.length)
        {
            return numbers.get(word_trace.word).copied();
        }

    }
    None
}


async fn calc_line(line: String) -> (u32, u32) {
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

    let first_digit_part1 = find_number_part1(line.chars()).await.unwrap();
    let last_digit_part1 = find_number_part1(line.chars().rev()).await.unwrap();
    
    let first_digit_part2 = find_number_part2(line.chars(), &numbers, false).await.unwrap();
    let last_digit_part2 = find_number_part2(line.chars().rev(), &numbers, true).await.unwrap();

    (format!("{first_digit_part1}{last_digit_part1}").parse::<u32>().unwrap(),
        format!("{first_digit_part2}{last_digit_part2}").parse::<u32>().unwrap())
}

pub async fn solve(mut rx: Receiver<String>) {
    let mut total_part1 = 0;
    let mut total_part2 = 0;

    let mut tasks = Vec::new();

    while let Some(line) = rx.recv().await {
        let task = tokio::spawn(async move { calc_line(line).await });

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
