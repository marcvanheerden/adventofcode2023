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
    max_rows: isize,
    max_cols: isize,
    path: &[Step],
    print: bool,
) -> (isize, isize, isize) {
    let mut nonpath_cells = Vec::new();
    let path_positions: Vec<_> = path.iter().map(|a| a.location).collect();
    for row in 0..=max_rows {
        for col in 0..=max_cols {
            if !path_positions.contains(&(row, col)) {
                nonpath_cells.push((row, col));
            }
        }
    }

    dbg!("A");
    let mut areas = Vec::new();
    let mut current_area = vec![nonpath_cells.pop().unwrap()];
    while !nonpath_cells.is_empty() {
        let mut flood: Vec<_> = nonpath_cells
            .extract_if(|c| {
                current_area
                    .iter()
                    .any(|c2| (c.0.abs_diff(c2.0) <= 1) & (c.1.abs_diff(c2.1) <= 1))
            })
            .collect();

        if flood.is_empty() {
            areas.push(current_area.clone());
            current_area.clear();
            if let Some(cell) = nonpath_cells.pop() {
                current_area = vec![cell];
            }
        } else {
            current_area.append(&mut flood);
        }
    }

    dbg!("B");
    areas.push(current_area);

    let right: Vec<_> = areas
        .clone()
        .into_iter()
        .filter(|a| a.iter().any(|c| path.iter().any(|a| a.right_side(c))))
        .flatten()
        .collect();

    dbg!("C");
    let left: Vec<_> = areas
        .into_iter()
        .filter(|a| a.iter().any(|c| path.iter().any(|a| a.left_side(c))))
        .flatten()
        .collect();

    assert_eq!(
        (max_rows + 1) * (max_cols + 1),
        (right.len() + left.len() + path.len()).try_into().unwrap()
    );

    if print {
        println!();
        for row in 0..=max_rows {
            for col in 0..=max_cols {
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
    colour: u32,
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

async fn parse_line(line: &str) -> Move {
    let line = line.replace(['(', ')', '#'], "");
    let parts: Vec<_> = line.split(' ').collect();
    let colour = u32::from_str_radix(parts[2], 16).unwrap();
    let magnitude = parts[1].parse::<isize>().unwrap();

    let direction = match parts[0] {
        "R" => Direction::R,
        "D" => Direction::D,
        "L" => Direction::L,
        "U" => Direction::U,
        _ => panic!("incorrect input format"),
    };

    Move {
        direction,
        magnitude,
        colour,
    }
}

pub async fn solve(mut rx: Receiver<(usize, String)>) {
    let mut holder = Vec::new();
    while let Some((line_no, line)) = rx.recv().await {
        holder.push((line_no, parse_line(&line).await));
    }

    holder.sort_by_key(|f| f.0);
    let mut moves = Vec::new();

    for (_, move_) in holder.into_iter() {
        moves.push(move_);
    }

    let steps = moves_to_steps(&moves);

    let max_rows = steps.iter().map(|s| s.location.0).max().unwrap();
    let max_cols = steps.iter().map(|s| s.location.1).max().unwrap();

    let part1 = count_enclosed(max_rows, max_cols, &steps, true);

    //let task1 = tokio::spawn(async move { map1.find_best_path(start1, 1, 3).await });
    //let task2 = tokio::spawn(async move { map2.find_best_path(start2, 4, 10).await });
    //let part1 = task1.await.unwrap();
    //let part2 = task2.await.unwrap();

    let part2 = 2;
    println!("Part 1: {:?} Part 2: {}", part1, part2);
}

#[cfg(test)]
mod tests {
    use super::*;

    // thanks https://blog.x5ff.xyz/blog/async-tests-tokio-rust/
    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }

    #[test]
    fn test_parse() {
        assert_eq!(
            aw!(parse_line("R 6 (#70c710)")),
            Move {
                direction: Direction::R,
                magnitude: 6,
                colour: 7390992
            }
        );
    }
}
