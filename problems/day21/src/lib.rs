use std::collections::{HashMap, HashSet};

use tokio::sync::mpsc::Receiver;

async fn parse_line(line_no: isize, line: &str) -> (Vec<(isize, isize)>, Option<(isize, isize)>) {
    let start: Vec<_> = line
        .chars()
        .enumerate()
        .filter_map(|(col, chr)| {
            if chr == 'S' {
                Some((line_no, col.try_into().unwrap()))
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
                Some((line_no, col.try_into().unwrap()))
            } else {
                None
            }
        })
        .collect();

    (rocks, start.first().copied())
}

fn untile(
    loc: &(isize, isize),
    lower_bound: &(isize, isize),
    upper_bound: &(isize, isize),
) -> (isize, isize) {
    let mut y = loc.0;
    let mut x = loc.1;
    let y_size = upper_bound.0 - lower_bound.0 + 1;
    let x_size = upper_bound.1 - lower_bound.1 + 1;

    while y < lower_bound.0 {
        y += y_size;
    }
    while x < lower_bound.1 {
        x += x_size;
    }
    while y > upper_bound.0 {
        y -= y_size;
    }
    while x > upper_bound.1 {
        x -= x_size;
    }

    (y, x)
}

fn neighbours(
    loc: &(isize, isize),
    rocks: &HashSet<(isize, isize)>,
    lower_bound: &(isize, isize),
    upper_bound: &(isize, isize),
    tile: bool,
) -> Vec<(isize, isize)> {
    let mut output = Vec::new();

    if tile {
        output.push((loc.0 - 1, loc.1));
        output.push((loc.0, loc.1 - 1));
        output.push((loc.0 + 1, loc.1));
        output.push((loc.0, loc.1 + 1));
    } else {
        if loc.0 > lower_bound.0 {
            output.push((loc.0 - 1, loc.1));
        }
        if loc.1 > lower_bound.1 {
            output.push((loc.0, loc.1 - 1));
        }
        if loc.0 < upper_bound.0 {
            output.push((loc.0 + 1, loc.1));
        }
        if loc.1 < upper_bound.1 {
            output.push((loc.0, loc.1 + 1));
        }
    }

    if tile {
        output
            .into_iter()
            .filter(|loc| {
                let uloc = untile(loc, lower_bound, upper_bound);
                !rocks.contains(&uloc)
            })
            .collect()
    } else {
        output
            .into_iter()
            .filter(|loc| !rocks.contains(loc))
            .collect()
    }
}

fn visit_gardens(
    rocks: &HashSet<(isize, isize)>,
    lower_bound: (isize, isize),
    upper_bound: (isize, isize),
    tile: bool,
    start: &(isize, isize),
    steps: isize,
) -> Vec<usize> {
    let mut positions = HashSet::new();
    positions.insert(*start);
    let mut gardens = vec![1];

    let mut cache: HashMap<(isize, isize), Vec<(isize, isize)>> = HashMap::new();

    for _ in 0..steps {
        let mut next_positions = HashSet::new();

        for step in positions.iter() {
            let nbors = if let Some(nbors) = cache.get(step) {
                nbors.clone()
            } else {
                let nbors = neighbours(step, rocks, &lower_bound, &upper_bound, tile);
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
            gardens.push(next_positions.len());
            positions = next_positions;
        }
    }

    gardens
}

fn fit_quadratic_equation(x_values: Vec<f64>, y_values: Vec<f64>) -> (f64, f64, f64) {
    // this function was written by ChatGPT and took a couple of tries to get it right
    let n = x_values.len() as f64;
    let sum_x = x_values.iter().sum::<f64>();
    let sum_y = y_values.iter().sum::<f64>();
    let sum_xx = x_values.iter().map(|&x| x * x).sum::<f64>();
    let sum_xy = x_values
        .iter()
        .zip(&y_values)
        .map(|(&x, &y)| x * y)
        .sum::<f64>();
    let sum_xxx = x_values.iter().map(|&x| x * x * x).sum::<f64>();
    let sum_xxxx = x_values.iter().map(|&x| x * x * x * x).sum::<f64>();
    let sum_xxy = x_values
        .iter()
        .zip(&y_values)
        .map(|(&x, &y)| x * x * y)
        .sum::<f64>();

    // Matrix elements of the normal equation
    let a11 = n;
    let a12 = sum_x;
    let a13 = sum_xx;
    let a21 = sum_x;
    let a22 = sum_xx;
    let a23 = sum_xxx;
    let a31 = sum_xx;
    let a32 = sum_xxx;
    let a33 = sum_xxxx;

    let b1 = sum_y;
    let b2 = sum_xy;
    let b3 = sum_xxy;

    // Solving the normal equations using Cramer's Rule
    let det_a = a11 * (a22 * a33 - a23 * a32) - a12 * (a21 * a33 - a23 * a31)
        + a13 * (a21 * a32 - a22 * a31);
    let det_a1 =
        b1 * (a22 * a33 - a23 * a32) - b2 * (a21 * a33 - a23 * a31) + b3 * (a21 * a32 - a22 * a31);
    let det_a2 =
        a11 * (b2 * a33 - b3 * a32) - a12 * (b1 * a33 - b3 * a31) + a13 * (b1 * a32 - b2 * a31);
    let det_a3 =
        a11 * (a22 * b3 - a23 * b2) - a12 * (a21 * b3 - a23 * b1) + a13 * (a21 * b2 - a22 * b1);

    let a = det_a1 / det_a;
    let b = det_a2 / det_a;
    let c = det_a3 / det_a;

    (a, b, c)
}

pub async fn solve(mut rx: Receiver<(usize, String)>) {
    let mut tasks = Vec::new();
    let mut rows = 0;
    let mut cols = 0;
    while let Some((line_no, line)) = rx.recv().await {
        if line.is_empty() {
            continue;
        }
        let line_no: isize = line_no.try_into().unwrap();
        rows = std::cmp::max(rows, line_no);
        cols = std::cmp::max(cols, line.trim().len().try_into().unwrap());
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

    dbg!(&rows);
    dbg!(&cols);
    let part1 = visit_gardens(&rocks, (0, 0), (rows, cols - 1), false, &start, 64);
    let part2 = visit_gardens(&rocks, (0, 0), (rows, cols - 1), true, &start, 65 + 131 * 2);

    let y: Vec<_> = part2
        .iter()
        .skip(65)
        .step_by(131)
        .map(|val| *val as f64)
        .collect();

    let x: Vec<_> = (0..y.len()).map(|val| val as f64).collect();
    let coeffs = fit_quadratic_equation(x, y);
    let target = 26501365.0;
    let x = (target - 65.0) / 131.0;
    let part2 = coeffs.2 * x * x + coeffs.1 * x + coeffs.0;

    println!(
        "Part 1: {} Part 2: {}",
        part1.last().unwrap(),
        part2.round()
    );
}
