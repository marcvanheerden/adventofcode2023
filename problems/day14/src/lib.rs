#![feature(btree_extract_if)]
use std::collections::BTreeMap;
use tokio::sync::mpsc::Receiver;

async fn get_rocks(row: usize, line: &str) -> BTreeMap<(usize, usize), bool> {
    let mut rocks = BTreeMap::new();
    for (col, chr) in line.chars().enumerate() {
        match chr {
            'O' => rocks.insert((row, col), true),
            '#' => rocks.insert((row, col), false),
            _ => None,
        };
    }
    rocks
}

pub async fn solve(mut rx: Receiver<(usize, String)>) {
    let mut tasks = Vec::new();

    let mut line_count = 0usize;
    while let Some((line_no, line)) = rx.recv().await {
        line_count += 1;
        let task = tokio::spawn(async move { get_rocks(line_no, &line).await });
        tasks.push(task);
    }

    let mut rocks = BTreeMap::new();

    for task in tasks {
        if let Ok(mut new_rocks) = task.await {
            rocks.append(&mut new_rocks);
        }
    }

    let part1 = tilt_north(rocks, line_count);

    println!("Part 1: {part1} Part 2: ");
}

fn tilt_north(rocks: BTreeMap<(usize, usize), bool>, lines: usize) -> usize {
    let mut stack_top: BTreeMap<_, _> = rocks
        .range((0, 0)..(0, usize::MAX))
        .map(|(loc, _movable)| (loc.1, 0))
        .collect();

    dbg!(&stack_top);
    dbg!(&lines);

    let mut rocks = rocks.clone();

    for line in 1..lines {
        let mut to_move: Vec<((usize, usize), bool)> = rocks
            .extract_if(|loc, movable| {
                if line != loc.0 {
                    return false;
                }

                if !*movable {
                    let stack = stack_top.entry(loc.1).or_insert(loc.0);
                    *stack = loc.0;
                    return false;
                }

                let mut space_to_move = true;
                if let Some(stop) = stack_top.get(&loc.1) {
                    if stop == &(loc.0 - 1) {
                        space_to_move = false;
                    }
                    stack_top.insert(loc.1, stop + 1);
                } else {
                    stack_top.insert(loc.1, 0);
                }

                space_to_move
            })
            .collect();

        let mut to_move = to_move
            .into_iter()
            .map(|(loc, movable)| ((*stack_top.get(&loc.1).unwrap(), loc.1), movable))
            .collect();

        rocks.append(&mut to_move);
    }

    dbg!(&rocks);
    rocks
        .into_iter()
        .filter(|(_loc, movable)| *movable)
        .map(|(loc, _movable)| loc.0.abs_diff(lines))
        .sum()
}
