#![feature(extract_if)]

use rug::Float;
use tokio::sync::mpsc::Receiver;

const PRECISION: u32 = 70;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum Direction {
    U,
    R,
    D,
    L,
}

impl Direction {
    fn opp(&self) -> Self {
        match self {
            Direction::U => Direction::D,
            Direction::R => Direction::L,
            Direction::D => Direction::U,
            Direction::L => Direction::R,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
struct Step {
    start_heading: Direction,
    end_heading: Direction,
    location: (isize, isize),
}

#[derive(Debug, Eq, PartialEq)]
struct Move {
    direction: Direction,
    magnitude: isize,
}

fn discrete_to_continuous(
    current_dir: Direction,
    next_dir: Direction,
    clockwise: bool,
) -> (Float, Float) {
    // adjust discrete blocks to continuous outer points

    let (dir_a, dir_b) = if clockwise {
        (current_dir, next_dir)
    } else {
        (next_dir.opp(), current_dir.opp())
    };

    let out = match (dir_a, dir_b) {
        (Direction::U, Direction::L) => (0.5, -0.5),
        (Direction::U, Direction::R) => (-0.5, -0.5),
        (Direction::L, Direction::U) => (0.5, -0.5),
        (Direction::L, Direction::D) => (0.5, 0.5),
        (Direction::D, Direction::L) => (0.5, 0.5),
        (Direction::D, Direction::R) => (-0.5, 0.5),
        (Direction::R, Direction::D) => (-0.5, 0.5),
        (Direction::R, Direction::U) => (-0.5, -0.5),
        _ => panic!("180s are radical but not allowed"),
    };

    (
        Float::with_val(PRECISION, out.0),
        Float::with_val(PRECISION, out.1),
    )
}

fn gauss_area(moves: &[Move]) -> Float {
    // assumes clockwise traversal
    let cw_adj = discrete_to_continuous(
        moves.last().unwrap().direction,
        moves.first().unwrap().direction,
        true,
    );
    let ccw_adj = discrete_to_continuous(
        moves.last().unwrap().direction,
        moves.first().unwrap().direction,
        false,
    );
    let mut points: Vec<(isize, isize, Float, Float, Float, Float)> =
        vec![(0, 0, cw_adj.0, cw_adj.1, ccw_adj.0, ccw_adj.1)];

    for moves in moves.windows(2) {
        let cw_adj = discrete_to_continuous(moves[0].direction, moves[1].direction, true);
        let ccw_adj = discrete_to_continuous(moves[0].direction, moves[1].direction, false);

        let diff = match moves[0].direction {
            Direction::U => (-1isize, 0isize),
            Direction::R => (0, 1),
            Direction::D => (1, 0),
            Direction::L => (0, -1),
        };

        let next_point = if let Some(point) = points.last() {
            let y = point.0 + moves[0].magnitude * diff.0;
            let x = point.1 + moves[0].magnitude * diff.1;

            (
                y,
                x,
                Float::with_val(PRECISION, y as f64 + cw_adj.0),
                Float::with_val(PRECISION, x as f64 + cw_adj.1),
                Float::with_val(PRECISION, y as f64 + ccw_adj.0),
                Float::with_val(PRECISION, x as f64 + ccw_adj.1),
            )
        } else {
            unreachable!();
        };
        points.push(next_point);
    }
    points.push(points[0].clone());

    let sums: (Float, Float, Float, Float) = points
        .windows(2)
        .map(|w| {
            (
                w[0].clone().2 * w[1].clone().3,
                w[1].clone().2 * w[0].clone().3,
                w[0].clone().4 * w[1].clone().5,
                w[1].clone().4 * w[0].clone().5,
            )
        })
        .fold(
            (
                Float::with_val(PRECISION, 0.0),
                Float::with_val(PRECISION, 0.0),
                Float::with_val(PRECISION, 0.0),
                Float::with_val(PRECISION, 0.0),
            ),
            |acc, (x, y, x2, y2)| (acc.0 + x, acc.1 + y, acc.2 + x2, acc.3 + y2),
        );

    dbg!(&sums);
    let cw_area = (sums.0 - sums.1).abs() / Float::with_val(PRECISION, 2);
    let ccw_area = (sums.2 - sums.3).abs() / Float::with_val(PRECISION, 2);

    if cw_area > ccw_area {
        cw_area
    } else {
        ccw_area
    }
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

    let part1 = gauss_area(&moves1);
    let part2 = gauss_area(&moves2);

    println!("Part 1: {} Part 2: {}", part1, part2);
}
