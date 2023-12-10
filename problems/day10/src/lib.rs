#![feature(extract_if)]

use colored::Colorize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

#[derive(PartialEq, Clone, Copy, Debug)]
enum Direction {
    N,
    E,
    S,
    W,
}

impl Direction {
    fn opp(&self) -> Self {
        match self {
            Self::N => Self::S,
            Self::E => Self::W,
            Self::S => Self::N,
            Self::W => Self::E,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
struct Step {
    start_heading: Direction,
    end_heading: Direction,
    location: (usize, usize),
}

impl Step {
    fn right_side(&self, loc: &(usize, usize)) -> bool {
        let start = match self.start_heading {
            Direction::N => *loc == (self.location.0 + 1, self.location.1),
            Direction::E => *loc == (self.location.0, self.location.1 + 1),
            Direction::S => self.location == (loc.0 + 1, loc.1),
            Direction::W => self.location == (loc.0, loc.1 + 1),
        };

        let end = match self.end_heading {
            Direction::N => *loc == (self.location.0 + 1, self.location.1),
            Direction::E => *loc == (self.location.0, self.location.1 + 1),
            Direction::S => self.location == (loc.0 + 1, loc.1),
            Direction::W => self.location == (loc.0, loc.1 + 1),
        };

        start | end
    }
    fn left_side(&self, loc: &(usize, usize)) -> bool {
        let start = match self.start_heading {
            Direction::S => *loc == (self.location.0 + 1, self.location.1),
            Direction::W => *loc == (self.location.0, self.location.1 + 1),
            Direction::N => self.location == (loc.0 + 1, loc.1),
            Direction::E => self.location == (loc.0, loc.1 + 1),
        };
        let end = match self.end_heading {
            Direction::S => *loc == (self.location.0 + 1, self.location.1),
            Direction::W => *loc == (self.location.0, self.location.1 + 1),
            Direction::N => self.location == (loc.0 + 1, loc.1),
            Direction::E => self.location == (loc.0, loc.1 + 1),
        };

        start | end
    }
}

#[derive(Debug, PartialEq, Clone)]
struct Animal {
    heading: Direction,
    location: (usize, usize),
}

#[derive(Debug)]
struct Pipe {
    in_: Direction,
    out: Direction,
}

impl Pipe {
    fn new(chr: char) -> Option<Self> {
        match chr {
            '|' => Some(Pipe {
                in_: Direction::N,
                out: Direction::S,
            }),
            '-' => Some(Pipe {
                in_: Direction::E,
                out: Direction::W,
            }),
            'F' => Some(Pipe {
                in_: Direction::S,
                out: Direction::E,
            }),
            '7' => Some(Pipe {
                in_: Direction::S,
                out: Direction::W,
            }),
            'J' => Some(Pipe {
                in_: Direction::N,
                out: Direction::W,
            }),
            'L' => Some(Pipe {
                in_: Direction::N,
                out: Direction::E,
            }),
            _ => None,
        }
    }

    fn pass(&self, animal: &mut Animal) -> bool {
        let from_dir = animal.heading.opp();

        let to_dir = if from_dir == self.in_ {
            self.out
        } else if from_dir == self.out {
            self.in_
        } else {
            return false;
        };

        animal.heading = to_dir;
        match to_dir {
            Direction::N => {
                if animal.location.1 > 0 {
                    animal.location.1 -= 1;
                    return true;
                }
            }
            Direction::E => {
                animal.location.0 += 1;
                return true;
            }
            Direction::S => {
                animal.location.1 += 1;
                return true;
            }
            Direction::W => {
                if animal.location.0 > 0 {
                    animal.location.0 -= 1;
                    return true;
                }
            }
        };

        false
    }
}

type PipeMap = Arc<Mutex<HashMap<(usize, usize), Pipe>>>;

async fn follow_animal(start: (usize, usize), pipemap: PipeMap) -> Vec<Step> {
    let steps = [(1, 0), (0, 1)];
    let dirs = [Direction::E, Direction::S];

    let mut animal = Animal {
        heading: Direction::W,
        location: start,
    };

    let mut path = Vec::new();

    let pmap = pipemap.lock().await;
    for (step, dir) in steps.into_iter().zip(dirs.into_iter()) {
        animal.heading = dir;
        animal.location = (start.0 + step.0, start.1 + step.1);
        // check that there is a pipe in that spot
        if let Some(pipe) = pmap.get(&(start.0 + step.0, start.1 + step.1)) {
            if pipe.pass(&mut animal) {
                path.push(Step {
                    start_heading: dir,
                    end_heading: dir,
                    location: start,
                });
                path.push(Step {
                    start_heading: dir,
                    end_heading: dir,
                    location: (start.0 + step.0, start.1 + step.1),
                });
                break;
            }
        }
    }
    drop(pmap);

    if animal.location == start {
        panic!("no first move");
    }
    let pmap = pipemap.lock().await;
    while animal.location != start {
        if let Some(pipe) = pmap.get(&animal.location) {
            let last_heading = animal.heading;
            let last_location = animal.location;
            pipe.pass(&mut animal);
            path.push(Step {
                start_heading: last_heading,
                end_heading: animal.heading,
                location: last_location,
            });
        } else {
            // data still being populated, wait a few ms
            sleep(Duration::from_millis(10)).await;
            dbg!("waiting for more data");
        }
    }

    path
}

async fn process_line(line: &str, line_no: usize, pipemap: PipeMap) -> Option<Vec<Step>> {
    let add_pipes = line
        .chars()
        .map(Pipe::new)
        .enumerate()
        .filter(|(_col, opt_pipe)| opt_pipe.is_some())
        .map(|(col, opt_pipe)| ((col, line_no), opt_pipe.unwrap()));

    // fill in the pipemap
    let mut pmap = pipemap.lock().await;
    pmap.extend(add_pipes);
    drop(pmap);

    // if this line contains the start
    // use this call to calculate the solution
    if let Some(col) = line.find('S') {
        return Some(follow_animal((col, line_no), pipemap).await);
    }

    None
}

fn count_enclosed(max_rows: usize, max_cols: usize, path: &[Step], print: bool) -> (usize, usize) {
    let mut nonpath_cells = Vec::new();
    let path_positions: Vec<_> = path.iter().map(|a| a.location).collect();
    for row in 0..=max_rows {
        for col in 0..=max_cols {
            if !path_positions.contains(&(col, row)) {
                nonpath_cells.push((col, row));
            }
        }
    }

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

    areas.push(current_area);

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
        (max_rows + 1) * (max_cols + 1),
        right.len() + left.len() + path.len()
    );

    println!();
    if print {
        for row in 0..=max_rows {
            for col in 0..=max_cols {
                if path_positions.contains(&(col, row)) {
                    print!("{}", '#'.to_string().red());
                } else if right.contains(&(col, row)) & left.contains(&(col, row)) {
                    print!("{}", '#'.to_string().purple());
                } else if right.contains(&(col, row)) {
                    print!("{}", '#'.to_string().yellow());
                } else if left.contains(&(col, row)) {
                    print!("{}", '#'.to_string().green());
                } else {
                    print!("#");
                }
            }
            println!()
        }
    }
    println!();

    (right.len(), left.len())
}

pub async fn solve(mut rx: Receiver<(usize, String)>) {
    let pipemap = Arc::new(Mutex::new(HashMap::new()));

    let mut tasks = Vec::new();

    while let Some((line_no, line)) = rx.recv().await {
        if !line.is_empty() {
            let pipemap_clone = Arc::clone(&pipemap);
            let task =
                tokio::spawn(async move { process_line(&line, line_no, pipemap_clone).await });
            tasks.push(task);
        }
    }

    let mut animal_path = Vec::new();
    for task in tasks {
        if let Ok(opt_dist) = task.await {
            if let Some(path) = opt_dist {
                animal_path = path;
            }
        }
    }

    let pmap = pipemap.lock().await;
    let max_cols = pmap.keys().map(|c| c.0).max().unwrap();
    let max_rows = pmap.keys().map(|c| c.1).max().unwrap();
    drop(pmap);

    let part1 = animal_path.len() / 2;

    let part2 = count_enclosed(max_rows, max_cols, &animal_path, true);
    println!("Part 1: {}, Part 2: {:?}", part1, part2);
}
