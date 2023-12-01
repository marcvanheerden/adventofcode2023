use tokio::sync::mpsc::Receiver;
use std::collections::HashMap;

pub async fn solve(mut rx: Receiver<String>) {

    let mut total_part1 = 0;
    let mut total_part2 = 0;

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

    let digits = ['1', '2', '3', '4', '5', '6', '7', '8', '9', '0'];
    
    while let Some(line) = rx.recv().await {
        // Part 1
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
        
        for digit in digits {
            let mut start = 0usize;
            while let Some(pos) = line[start..].find(digit) {
                let actual_pos = start + pos;
                start = actual_pos + 1;
                value_position.insert(actual_pos, digit);
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
    }

    println!("Part 1: {total_part1} Part 2: {total_part2}");
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
    }
}
