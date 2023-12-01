use tokio::sync::mpsc::Receiver;
use std::collections::HashMap;

async fn calc_line(line: String) -> (u32, u32) {
     
    let mut numbers = HashMap::new();
    numbers.insert("one", '1');
    numbers.insert("two", '2');
    numbers.insert("three", '3');
    numbers.insert("four", '4');
    numbers.insert("five", '5');
    numbers.insert("six", '6');
    numbers.insert("seven", '7');
    numbers.insert("eight", '8');
    numbers.insert("nine", '9');
    numbers.insert("1", '1');
    numbers.insert("2", '2');
    numbers.insert("3", '3');
    numbers.insert("4", '4');
    numbers.insert("5", '5');
    numbers.insert("6", '6');
    numbers.insert("7", '7');
    numbers.insert("8", '8');
    numbers.insert("9", '9');
    numbers.insert("0", '0');
   
    let mut total_part1 = 0;
    let mut total_part2 = 0;
    
    let mut first: Option<char> = None;
    let mut last: char = '~';
    
    for chr in line.chars() {
        if chr.is_digit(10) {
            if first.is_none() {
                first = Some(chr);
            }
            last = chr;
        }
    }
    
    if let Some(first_digit) = first {
        total_part1 += format!("{first_digit}{last}").parse::<u32>().unwrap();
    }
    
    // Part 2
    let mut first: Option<char> = None;
    let mut last: char = '~';

    let mut value_position = HashMap::new();
    
    for (str_rep, num_rep) in &numbers {
        let mut start = 0usize;
        while let Some(pos) = line[start..].find(str_rep) {
            let actual_pos = start + pos;
            //start = actual_pos + str_rep.len();
            start += 1;
            value_position.insert(actual_pos, *num_rep);
        }
    }

    for pos in 0..line.len() {
        if let Some(&value) = value_position.get(&pos) {
            if first.is_none() {
                first = Some(value);
            }
            last = value;
        }
    }

    if let Some(first_digit) = first {
        total_part2 += format!("{first_digit}{last}").parse::<u32>().unwrap();
    }

    (total_part1, total_part2)
}

pub async fn solve(mut rx: Receiver<String>) {

    let mut total_part1 = 0;
    let mut total_part2 = 0;

    let mut tasks = Vec::new();
 
    while let Some(line) = rx.recv().await {
        let task = tokio::spawn(async move {
            calc_line(line).await
        });

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

