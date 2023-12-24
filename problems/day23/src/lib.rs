use std::collections::{HashMap, HashSet, VecDeque};
use tokio::sync::mpsc::Receiver;

#[derive(Debug, Eq, PartialEq)]
enum PathSlope {
    Path,
    SlopeN,
    SlopeE,
    SlopeW,
    SlopeS,
}

async fn parse_line(line_no: usize, line: &str) -> Vec<((usize, usize), PathSlope)> {
    line.trim()
        .chars()
        .enumerate()
        .filter_map(|(col, chr)| match chr {
            '.' => Some(((line_no, col), PathSlope::Path)),
            '>' => Some(((line_no, col), PathSlope::SlopeE)),
            'v' => Some(((line_no, col), PathSlope::SlopeS)),
            '^' => Some(((line_no, col), PathSlope::SlopeN)),
            '<' => Some(((line_no, col), PathSlope::SlopeW)),
            _ => None,
        })
        .collect()
}

#[derive(Debug, Clone)]
struct Progress {
    loc: (usize, usize),
    last_loc: (usize, usize),
    steps: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct ProgressCanClimb {
    loc: (usize, usize),
    history: HashSet<(usize, usize)>,
}

fn next_steps(
    loc: &(usize, usize),
    map: &HashMap<(usize, usize), PathSlope>,
    can_climb: bool,
) -> Vec<(usize, usize)> {
    // short circuit if on a slope

    if !can_climb {
        if let Some(pathslope) = map.get(loc) {
            match pathslope {
                PathSlope::Path => (),
                PathSlope::SlopeN => return vec![(loc.0 - 1, loc.1)],
                PathSlope::SlopeE => return vec![(loc.0, loc.1 + 1)],
                PathSlope::SlopeW => return vec![(loc.0, loc.1 - 1)],
                PathSlope::SlopeS => return vec![(loc.0 + 1, loc.1)],
            }
        }
    }

    let mut moves = Vec::new();
    if loc.0 > 0 {
        if let Some(pathslope) = map.get(&(loc.0 - 1, loc.1)) {
            if can_climb | (*pathslope != PathSlope::SlopeS) {
                moves.push((loc.0 - 1, loc.1));
            }
        }
    }

    if loc.1 > 0 {
        if let Some(pathslope) = map.get(&(loc.0, loc.1 - 1)) {
            if can_climb | (*pathslope != PathSlope::SlopeE) {
                moves.push((loc.0, loc.1 - 1));
            }
        }
    }

    if let Some(pathslope) = map.get(&(loc.0 + 1, loc.1)) {
        if can_climb | (*pathslope != PathSlope::SlopeN) {
            moves.push((loc.0 + 1, loc.1));
        }
    }

    if let Some(pathslope) = map.get(&(loc.0, loc.1 + 1)) {
        if can_climb | (*pathslope != PathSlope::SlopeW) {
            moves.push((loc.0, loc.1 + 1));
        }
    }

    moves
}
fn longest_route_can_climb(
    map: &HashMap<(usize, usize), PathSlope>,
    start: (usize, usize),
    end_row: usize,
) -> usize {
    let mut queue = vec![ProgressCanClimb {
        loc: start,
        history: HashSet::new(),
    }];

    let mut longest = 0usize;

    loop {
        dbg!(&queue.len());
        let mut next_queue = Vec::new();

        for step in queue.iter().take(10000) {
            if step.loc.0 == end_row {
                longest = std::cmp::max(longest, step.history.len());
                dbg!(&longest);
                continue;
            }

            for next_step in next_steps(&step.loc, map, true) {
                if step.history.contains(&next_step) {
                    continue;
                }
                let mut history = step.history.clone();
                history.insert(step.loc);

                let progress = ProgressCanClimb {
                    loc: next_step,
                    history,
                };

                if !next_queue.contains(&progress) {
                    next_queue.push(progress)
                }
            }
        }

        if next_queue.is_empty() {
            break;
        } else {
            next_queue.sort_by_key(|p| p.loc.0 + p.loc.1);
            queue = next_queue;
        }
    }
    longest
}

fn longest_route(
    map: &HashMap<(usize, usize), PathSlope>,
    start: (usize, usize),
    end_row: usize,
) -> usize {
    let mut queue = VecDeque::from(vec![Progress {
        loc: start,
        last_loc: start,
        steps: 0,
    }]);
    let mut finished = Vec::new();

    loop {
        let mut next_queue = VecDeque::new();

        for step in queue.iter() {
            if step.loc.0 == end_row {
                finished.push(step.steps);
                continue;
            }

            for next_step in next_steps(&step.loc, map, false) {
                if next_step == step.last_loc {
                    continue;
                }

                next_queue.push_back(Progress {
                    loc: next_step,
                    last_loc: step.loc,
                    steps: step.steps + 1,
                })
            }
        }

        if next_queue.is_empty() {
            break;
        } else {
            queue = next_queue;
        }
    }

    finished.into_iter().max().unwrap()
}

pub async fn solve(mut rx: Receiver<(usize, String)>) {
    let mut tasks = Vec::new();
    while let Some((line_no, line)) = rx.recv().await {
        if line.is_empty() {
            continue;
        }
        let task = tokio::spawn(async move { parse_line(line_no, &line).await });
        tasks.push(task);
    }

    let mut map = HashMap::new();
    for task in tasks {
        if let Ok(pathslopes) = task.await {
            for (loc, typ) in pathslopes.into_iter() {
                map.insert(loc, typ);
            }
        }
    }

    let end_row = map.keys().map(|k| k.0).max().unwrap();
    let part1 = longest_route(&map, (0, 1), end_row);
    let part2 = longest_route_can_climb(&map, (0, 1), end_row);

    println!("Part 1: {} Part 2: {}", part1, part2,);
}
