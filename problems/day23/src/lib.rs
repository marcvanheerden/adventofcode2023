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
    last_loc: (usize, usize),
    slope_history: HashSet<(usize, usize)>,
    steps: usize,
}

fn next_steps(
    loc: &(usize, usize),
    map: &HashMap<(usize, usize), PathSlope>,
) -> Vec<(usize, usize)> {
    // short circuit if on a slope

    if let Some(pathslope) = map.get(loc) {
        match pathslope {
            PathSlope::Path => (),
            PathSlope::SlopeN => return vec![(loc.0 - 1, loc.1)],
            PathSlope::SlopeE => return vec![(loc.0, loc.1 + 1)],
            PathSlope::SlopeW => return vec![(loc.0, loc.1 - 1)],
            PathSlope::SlopeS => return vec![(loc.0 + 1, loc.1)],
        }
    }

    let mut moves = Vec::new();
    if loc.0 > 0 {
        if let Some(pathslope) = map.get(&(loc.0 - 1, loc.1)) {
            if *pathslope != PathSlope::SlopeS {
                moves.push((loc.0 - 1, loc.1));
            }
        }
    }

    if loc.1 > 0 {
        if let Some(pathslope) = map.get(&(loc.0, loc.1 - 1)) {
            if *pathslope != PathSlope::SlopeE {
                moves.push((loc.0, loc.1 - 1));
            }
        }
    }

    if let Some(pathslope) = map.get(&(loc.0 + 1, loc.1)) {
        if *pathslope != PathSlope::SlopeN {
            moves.push((loc.0 + 1, loc.1));
        }
    }

    if let Some(pathslope) = map.get(&(loc.0, loc.1 + 1)) {
        if *pathslope != PathSlope::SlopeW {
            moves.push((loc.0, loc.1 + 1));
        }
    }

    moves
}

fn nbors(loc: &(usize, usize)) -> Vec<(usize, usize)> {
    let mut output = vec![(loc.0 + 1, loc.1), (loc.0, loc.1 + 1)];

    if loc.0 > 0 {
        output.push((loc.0 - 1, loc.1));
    }
    if loc.1 > 0 {
        output.push((loc.0, loc.1 - 1));
    }

    output
}

fn longest_route_can_climb(
    map: &HashMap<(usize, usize), PathSlope>,
    start: (usize, usize),
    end_row: usize,
) -> usize {
    // ported from https://gist.github.com/ke-hermann/279f352829cd590d61104c27cac59bdc
    let paths: HashSet<(usize, usize)> = map.keys().cloned().collect();
    let mut neighbours = HashMap::new();

    for point in paths.iter() {
        let point_nbors: Vec<(usize, usize)> = nbors(point)
            .into_iter()
            .filter(|p| paths.contains(p))
            .collect();

        neighbours.insert(*point, point_nbors);
    }

    let mut intersections: HashSet<(usize, usize)> = neighbours
        .iter()
        .filter(|(_k, v)| v.len() >= 3)
        .map(|(k, _v)| *k)
        .collect();

    // add start and end points to intersections
    intersections.insert(start);
    intersections.insert(*paths.iter().filter(|p| p.0 == end_row).next().unwrap());

    let mut graph = HashMap::new();

    for inter in intersections.iter() {
        for neighbour in neighbours.get(&inter).unwrap() {
            let mut history = HashSet::new();
            history.insert(*inter);
            let (point, distance) =
                intersect_dist(*neighbour, 1, history, &intersections, &neighbours);
            let entry = graph.entry(*inter).or_insert(Vec::new());
            entry.push((point, distance));
        }
    }

    let mut history = HashSet::new();
    history.insert(start);
    breadth_first_search(start, 0, history, end_row, &graph)
}

fn breadth_first_search(
    node: (usize, usize),
    distance: usize,
    history: HashSet<(usize, usize)>,
    end_row: usize,
    graph: &HashMap<(usize, usize), Vec<((usize, usize), usize)>>,
) -> usize {
    if node.0 == end_row {
        return distance;
    }

    let max = graph
        .get(&node)
        .unwrap()
        .iter()
        .filter(|(point, _distance)| !history.contains(point))
        .map(|(point, extra_distance)| {
            let mut new_history = history.clone();
            new_history.insert(*point);
            breadth_first_search(
                *point,
                distance + extra_distance,
                new_history,
                end_row,
                graph,
            )
        })
        .max();

    if let Some(max_) = max {
        max_
    } else {
        0
    }
}

fn intersect_dist(
    cur: (usize, usize),
    dist: usize,
    history: HashSet<(usize, usize)>,
    intersections: &HashSet<(usize, usize)>,
    neighbours: &HashMap<(usize, usize), Vec<(usize, usize)>>,
) -> ((usize, usize), usize) {
    if intersections.contains(&cur) {
        return (cur, dist);
    }

    if let Some(neighbour) = neighbours
        .get(&cur)
        .unwrap()
        .into_iter()
        .filter(|p| !history.contains(p))
        .next()
    {
        let mut new_history = history.clone();
        new_history.insert(cur);
        return intersect_dist(*neighbour, dist + 1, new_history, intersections, neighbours);
    }
    dbg!(&cur);
    dbg!(neighbours.get(&cur));
    dbg!(&history);
    unreachable!();
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

            for next_step in next_steps(&step.loc, map) {
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
