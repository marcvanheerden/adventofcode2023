use std::collections::{HashMap, HashSet};

use tokio::sync::mpsc::Receiver;

async fn parse_line(line_no: usize, line: &str) -> (Vec<(usize, usize)>, Option<(usize, usize)>) {
    let start: Vec<_> = line
        .chars()
        .enumerate()
        .filter_map(|(col, chr)| {
            if chr == 'S' {
                Some((line_no, col))
            } else {
                None
            }
        })
        .collect();

    let rocks = line
        .chars()
        .enumerate()
        .filter_map(|(col, chr)| {
            if ['#'].contains(&chr) {
                Some((line_no, col))
            } else {
                None
            }
        })
        .collect();

    (rocks, start.first().copied())
}

fn neighbours(
    loc: &(usize, usize),
    rocks: &HashSet<(usize, usize)>,
    lower_bound: Option<(usize, usize)>,
    upper_bound: Option<(usize, usize)>,
) -> Vec<(usize, usize)> {
    let mut output = Vec::new();

    if let Some(lower) = lower_bound {
        if loc.0 > lower.0 {
            output.push((loc.0 - 1, loc.1));
        }
        if loc.1 > lower.1 {
            output.push((loc.0, loc.1 - 1));
        }
    } else {
        output.push((loc.0 - 1, loc.1));
        output.push((loc.0, loc.1 - 1));
    }

    if let Some(upper) = upper_bound {
        if loc.0 < upper.0 {
            output.push((loc.0 + 1, loc.1));
        }
        if loc.1 < upper.1 {
            output.push((loc.0, loc.1 + 1));
        }
    } else {
        output.push((loc.0 + 1, loc.1));
        output.push((loc.0, loc.1 + 1));
    }

    output
        .into_iter()
        .filter(|loc| !rocks.contains(loc))
        .collect()
}

fn visit_gardens(
    rocks: &HashSet<(usize, usize)>,
    lower_bound: Option<(usize, usize)>,
    upper_bound: Option<(usize, usize)>,
    start: &(usize, usize),
    steps: usize,
) -> usize {
    let mut positions = HashSet::new();
    positions.insert(*start);

    let mut cache: HashMap<(usize, usize), Vec<(usize, usize)>> = HashMap::new();

    for x in 0..steps {
        let mut next_positions = HashSet::new();

        for step in positions.iter() {
            let nbors = if let Some(nbors) = cache.get(step) {
                nbors.clone()
            } else {
                let nbors = neighbours(step, rocks, lower_bound, upper_bound);
                cache.insert(*step, nbors.clone());
                nbors
            };

            for neighbour in nbors.iter() {
                next_positions.insert(*neighbour);
            }
        }

        if next_positions.is_empty() {
            break;
        } else {
            positions = next_positions;
        }
    }

    positions.len()
}

pub async fn solve(mut rx: Receiver<(usize, String)>) {
    let mut tasks = Vec::new();
    let mut rows = 0;
    let mut cols = 0;
    while let Some((line_no, line)) = rx.recv().await {
        if line.is_empty() {
            continue;
        }
        rows = std::cmp::max(rows, line_no);
        cols = std::cmp::max(cols, line.trim().len());
        let task = tokio::spawn(async move { parse_line(line_no, &line).await });
        tasks.push(task);
    }

    let mut rocks = HashSet::new();
    let mut start = (0, 0);
    for task in tasks {
        if let Ok((rocks_, start_)) = task.await {
            for rock in rocks_.into_iter() {
                rocks.insert(rock);
            }
            if let Some(starting_point) = start_ {
                start = starting_point;
            }
        }
    }

    let part1 = visit_gardens(&rocks, Some((0, 0)), Some((rows, cols - 1)), &start, 64);
    let part2 = 1;

    println!("Part 1: {} Part 2: {}", part1, part2);
}
