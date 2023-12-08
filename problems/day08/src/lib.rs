use std::collections::{HashMap, VecDeque};
use tokio::sync::mpsc::Receiver;

enum Input {
    Directions(VecDeque<bool>),
    Node((String, String, String)),
}

async fn calc_line(line: &str) -> Input {
    if let Some((start, destinations)) = line.split_once(" = (") {
        let (left_dest, right_dest) = destinations.split_once(", ").unwrap();
        return Input::Node((
            start.to_string(),
            left_dest.to_string(),
            right_dest.replace(')', ""),
        ));
    }

    Input::Directions(line.chars().map(|c| c == 'R').collect())
}

pub async fn solve(mut rx: Receiver<String>) {
    let mut tasks = Vec::new();

    while let Some(line) = rx.recv().await {
        if !line.is_empty() {
            let task = tokio::spawn(async move { calc_line(&line).await });
            tasks.push(task);
        }
    }

    let mut directions = VecDeque::new();
    let mut nodes = HashMap::new();

    for task in tasks {
        if let Ok(input) = task.await {
            match input {
                Input::Directions(dir) => {
                    directions = dir;
                }
                Input::Node(node) => {
                    nodes.insert(node.0, (node.1, node.2));
                }
            };
        }
    }

    let part1 = count_to_zzz("AAA".to_string(), &nodes, &directions);

    let part2 = nodes
        .keys()
        .filter(|k| k.chars().skip(2).next().unwrap() == 'A')
        .map(|key| period_to_xxz(key.to_string(), &nodes, &directions))
        .fold(1, |acc, num| lcm(acc, num));

    println!("Part 1: {part1}, Part 2: {part2}");
}

fn gcd(a: usize, b: usize) -> usize {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

fn lcm(a: usize, b: usize) -> usize {
    a / gcd(a, b) * b
}

fn count_to_zzz(
    start: String,
    nodes: &HashMap<String, (String, String)>,
    directions: &VecDeque<bool>,
) -> usize {
    let mut current_node = start.clone();
    let mut count = 0;
    let mut directions = directions.clone();

    loop {
        count += 1;
        if let Some((left, right)) = nodes.get(&current_node) {
            current_node = if directions[0] {
                right.to_string()
            } else {
                left.to_string()
            }
        }
        directions.rotate_left(1);
        if current_node == "ZZZ" {
            break;
        }
    }

    count
}

fn period_to_xxz(
    start: String,
    nodes: &HashMap<String, (String, String)>,
    directions: &VecDeque<bool>,
) -> usize {
    let mut current_node = start.clone();
    let mut count = 0;
    let mut directions = directions.clone();

    loop {
        count += 1;
        if let Some((left, right)) = nodes.get(&current_node) {
            current_node = if directions[0] {
                right.to_string()
            } else {
                left.to_string()
            }
        }
        directions.rotate_left(1);

        if let Some(last_char) = current_node.chars().skip(2).next() {
            if last_char == 'Z' {
                return count;
            }
        }
    }
}
