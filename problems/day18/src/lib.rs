#![feature(extract_if)]

use colored::Colorize;
use tokio::sync::mpsc::Receiver;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum Direction {
    U,
    R,
    D,
    L,
}

fn count_enclosed(
    row_limits: (isize, isize),
    col_limits: (isize, isize),
    path: &[Step],
    print: bool,
) -> (isize, isize, isize) {
    let mut nonpath_cells = Vec::new();
    let path_positions: Vec<_> = path.iter().map(|a| a.location).collect();
    for row in row_limits.0..=row_limits.1 {
        for col in col_limits.0..=col_limits.1 {
            if !path_positions.contains(&(row, col)) {
                nonpath_cells.push((row, col));
            }
        }
    }

    let mut areas = Vec::new();
    let mut current_area = vec![vec![nonpath_cells.pop().unwrap()]];
    while !nonpath_cells.is_empty() {
        let flood: Vec<_> = nonpath_cells
            .extract_if(|c| {
                current_area
                    .last()
                    .unwrap()
                    .iter()
                    .any(|c2| (c.0.abs_diff(c2.0) <= 1) & (c.1.abs_diff(c2.1) <= 1))
            })
            .collect();

        if flood.is_empty() {
            let current_area_flat: Vec<_> = current_area.iter().cloned().flatten().collect();
            areas.push(current_area_flat);
            current_area.clear();
            if let Some(cell) = nonpath_cells.pop() {
                current_area = vec![vec![cell]];
            }
        } else {
            current_area.push(flood);
        }
    }
    let current_area_flat: Vec<_> = current_area.iter().cloned().flatten().collect();
    areas.push(current_area_flat);

    let right: Vec<_> = areas
        .clone()
        .into_iter()
        .filter(|a| a.iter().any(|c| path.iter().any(|a| a.right_side(c))))
        .flatten()
        .collect();

    let left: Vec<_> = areas
        .into_iter()
        .filter(|a| a.iter().any(|c| path.iter().any(|a| a.left_side(c))))
        .flatten()
        .collect();

    assert_eq!(
        (row_limits.1 - row_limits.0 + 1) * (col_limits.1 - col_limits.0 + 1),
        (right.len() + left.len() + path.len()).try_into().unwrap()
    );

    if print {
        println!();
        for row in row_limits.0..=row_limits.1 {
            for col in col_limits.0..=col_limits.1 {
                if path_positions.contains(&(row, col)) {
                    print!("{}", '#'.to_string().red());
                } else if right.contains(&(row, col)) & left.contains(&(row, col)) {
                    print!("{}", '#'.to_string().purple());
                } else if right.contains(&(row, col)) {
                    print!("{}", '#'.to_string().yellow());
                } else if left.contains(&(row, col)) {
                    print!("{}", '#'.to_string().green());
                } else {
                    print!("#");
                }
            }
            println!()
        }
        println!();
    }

    (
        right.len().try_into().unwrap(),
        left.len().try_into().unwrap(),
        path.len().try_into().unwrap(),
    )
}

#[derive(Debug, PartialEq, Clone)]
struct Step {
    start_heading: Direction,
    end_heading: Direction,
    location: (isize, isize),
}

impl Step {
    fn right_side(&self, loc: &(isize, isize)) -> bool {
        let start = match self.start_heading {
            Direction::U => *loc == (self.location.0, self.location.1 + 1),
            Direction::R => *loc == (self.location.0 + 1, self.location.1),
            Direction::D => self.location == (loc.0, loc.1 + 1),
            Direction::L => self.location == (loc.0 + 1, loc.1),
        };

        let end = match self.end_heading {
            Direction::U => *loc == (self.location.0, self.location.1 + 1),
            Direction::R => *loc == (self.location.0 + 1, self.location.1),
            Direction::D => self.location == (loc.0, loc.1 + 1),
            Direction::L => self.location == (loc.0 + 1, loc.1),
        };

        start | end
    }
    fn left_side(&self, loc: &(isize, isize)) -> bool {
        let start = match self.start_heading {
            Direction::D => *loc == (self.location.0, self.location.1 + 1),
            Direction::L => *loc == (self.location.0 + 1, self.location.1),
            Direction::U => self.location == (loc.0, loc.1 + 1),
            Direction::R => self.location == (loc.0 + 1, loc.1),
        };
        let end = match self.end_heading {
            Direction::D => *loc == (self.location.0, self.location.1 + 1),
            Direction::L => *loc == (self.location.0 + 1, self.location.1),
            Direction::U => self.location == (loc.0, loc.1 + 1),
            Direction::R => self.location == (loc.0 + 1, loc.1),
        };

        start | end
    }
}

#[derive(Debug, PartialEq, Clone)]
struct Digger {
    heading: Direction,
    location: (isize, isize),
}

#[derive(Debug, Eq, PartialEq)]
struct Move {
    direction: Direction,
    magnitude: isize,
}

fn moves_to_steps(moves: &[Move]) -> Vec<Step> {
    let mut location = (0isize, 0isize);
    let mut output: Vec<Step> = Vec::new();

    for move_ in moves.iter() {
        let diff = match move_.direction {
            Direction::U => (-1isize, 0isize),
            Direction::R => (0, 1),
            Direction::D => (1, 0),
            Direction::L => (0, -1),
        };

        for step in 0..move_.magnitude {
            if step == 0 {
                if let Some(last_step) = output.last_mut() {
                    last_step.end_heading = move_.direction;
                }
            }
            location = (location.0 + diff.0, location.1 + diff.1);

            output.push(Step {
                start_heading: move_.direction,
                end_heading: move_.direction,
                location,
            });
        }
    }

    output
}

fn discrete_to_continuous(current_dir: Direction, next_dir: Direction) -> (f64, f64) {
    match (current_dir, next_dir) {
        (Direction::U, Direction::L) => (0.5, -0.5),
        (Direction::U, Direction::R) => (-0.5, -0.5),
        (Direction::L, Direction::U) => (0.5, -0.5),
        (Direction::L, Direction::D) => (0.5, 0.5),
        (Direction::D, Direction::L) => (0.5, 0.5),
        (Direction::D, Direction::R) => (-0.5, 0.5),
        (Direction::R, Direction::D) => (-0.5, 0.5),
        (Direction::R, Direction::U) => (-0.5, -0.5),
        _ => panic!("180s are radical but not allowed"),
    }
}

fn gauss_area(moves: &[Move]) -> f64 {
    // assumes clockwise traversal
    let adj = discrete_to_continuous(
        moves.last().unwrap().direction,
        moves.first().unwrap().direction,
    );
    let mut points: Vec<(isize, isize, f64, f64)> = vec![(0, 0, adj.0, adj.1)];

    for moves in moves.windows(2) {
        let adj = discrete_to_continuous(moves[0].direction, moves[1].direction);

        let diff = match moves[0].direction {
            Direction::U => (-1isize, 0isize),
            Direction::R => (0, 1),
            Direction::D => (1, 0),
            Direction::L => (0, -1),
        };

        let next_point = if let Some(point) = points.last() {
            (
                point.0 + moves[0].magnitude * diff.0,
                point.1 + moves[0].magnitude * diff.1,
                (point.0 + moves[0].magnitude * diff.0) as f64 + adj.0,
                (point.1 + moves[0].magnitude * diff.1) as f64 + adj.1,
            )
        } else {
            (0isize, 0isize, 0f64, 0f64)
        };
        points.push(next_point);
    }
    points.push(points[0].clone());

    let sums: (f64, f64) = points
        .windows(2)
        .map(|w| (w[0].2 * w[1].3, w[1].2 * w[0].3))
        .fold((0.0, 0.0), |acc, (x, y)| (acc.0 + x, acc.1 + y));

    (sums.0 - sums.1).abs() / 2.0
}

async fn parse_line(line: &str) -> (Move, Move) {
    let line = line.replace(['(', ')', '#'], "");
    let parts: Vec<_> = line.split(' ').collect();
    let magnitude = parts[1].parse::<isize>().unwrap();

    let direction = match parts[0] {
        "R" => Direction::R,
        "D" => Direction::D,
        "L" => Direction::L,
        "U" => Direction::U,
        _ => panic!("incorrect input format"),
    };

    let move1 = Move {
        direction,
        magnitude,
    };

    let direction = match parts[2].chars().next_back().unwrap() {
        '0' => Direction::R,
        '1' => Direction::D,
        '2' => Direction::L,
        '3' => Direction::U,
        _ => panic!("incorrect input format"),
    };

    let magnitude =
        isize::from_str_radix(&parts[2].chars().take(5).collect::<String>(), 16).unwrap();

    (
        move1,
        Move {
            direction,
            magnitude,
        },
    )
}

pub async fn solve(mut rx: Receiver<(usize, String)>) {
    let mut holder = Vec::new();
    while let Some((line_no, line)) = rx.recv().await {
        holder.push((line_no, parse_line(&line).await));
    }

    holder.sort_by_key(|f| f.0);
    let mut moves1 = Vec::new();
    let mut moves2 = Vec::new();

    for (_, moves) in holder.into_iter() {
        moves1.push(moves.0);
        moves2.push(moves.1);
    }

    let steps = moves_to_steps(&moves1);

    let min_rows = steps.iter().map(|s| s.location.0).min().unwrap();
    let max_rows = steps.iter().map(|s| s.location.0).max().unwrap();
    let min_cols = steps.iter().map(|s| s.location.1).min().unwrap();
    let max_cols = steps.iter().map(|s| s.location.1).max().unwrap();

    let part1 = gauss_area(&moves1);
    let part2 = gauss_area(&moves2);

    println!("Part 1: {:?} Part 2: {}", part1, part2);
}
