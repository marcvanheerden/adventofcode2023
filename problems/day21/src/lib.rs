use std::collections::{HashMap, HashSet, VecDeque};

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

    let plots = line
        .chars()
        .enumerate()
        .filter_map(|(col, chr)| {
            if ['.', 'S'].contains(&chr) {
                Some((line_no, col))
            } else {
                None
            }
        })
        .collect();

    (plots, start.first().copied())
}

fn neighbours(loc: &(usize, usize), map: &HashSet<(usize, usize)>) -> Vec<(usize, usize)> {
    map.iter()
        .filter_map(|(y, x)| {
            if (loc.0.abs_diff(*y) + loc.1.abs_diff(*x)) == 1 {
                Some((*y, *x))
            } else {
                None
            }
        })
        .collect()
}

fn visit_gardens(plots: &HashSet<(usize, usize)>, start: &(usize, usize), steps: usize) -> usize {
    let mut positions = HashSet::new();
    positions.insert(*start);

    let mut cache: HashMap<(usize, usize), Vec<(usize, usize)>> = HashMap::new();

    for x in 0..steps {
        dbg!(&positions.len());
        let mut next_positions = HashSet::new();

        for step in positions.iter() {
            let nbors = if let Some(nbors) = cache.get(step) {
                nbors.clone()
            } else {
                let nbors = neighbours(step, plots);
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
    while let Some((line_no, line)) = rx.recv().await {
        let task = tokio::spawn(async move { parse_line(line_no, &line).await });
        tasks.push(task);
    }

    let mut plots = HashSet::new();
    let mut start = (0, 0);
    for task in tasks {
        if let Ok((plots_, start_)) = task.await {
            for plot in plots_.into_iter() {
                plots.insert(plot);
            }
            if let Some(starting_point) = start_ {
                start = starting_point;
            }
        }
    }

    let part1 = visit_gardens(&plots, &start, 64);
    let part2 = 1;

    println!("Part 1: {} Part 2: {}", part1, part2);
}
