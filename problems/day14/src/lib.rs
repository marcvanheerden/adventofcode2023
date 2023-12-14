#![feature(extract_if)]
use std::collections::HashMap;
use tokio::sync::mpsc::Receiver;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct Rock {
    row: usize,
    col: usize,
    moves: bool,
}

async fn get_rocks(row: usize, line: &str) -> Vec<Rock> {
    let mut rocks = Vec::new();
    for (col, chr) in line.chars().enumerate() {
        match chr {
            'O' => {
                rocks.push(Rock {
                    row,
                    col,
                    moves: true,
                });
            }
            '#' => {
                rocks.push(Rock {
                    row,
                    col,
                    moves: false,
                });
            }
            _ => {
                ();
            }
        };
    }
    rocks
}

fn get_load(rocks: &[Rock], rows: usize) -> usize {
    rocks
        .iter()
        .filter(|r| r.moves)
        .map(|r| r.row.abs_diff(rows))
        .sum()
}

pub async fn solve(mut rx: Receiver<(usize, String)>) {
    let mut tasks = Vec::new();

    let mut line_count = 0usize;
    let mut line_len = 0usize;
    while let Some((line_no, line)) = rx.recv().await {
        line_count += 1;
        line_len = std::cmp::max(line_len, line.len());

        let task = tokio::spawn(async move { get_rocks(line_no, &line).await });
        tasks.push(task);
    }

    let mut rocks = Vec::new();

    for task in tasks {
        if let Ok(mut new_rocks) = task.await {
            rocks.append(&mut new_rocks);
        }
    }

    let mut rocks1 = rocks.clone();
    tilt(&mut rocks1, line_count, line_len, Dir::N);
    let part1 = get_load(&rocks1, line_count);

    tilt(&mut rocks1, line_count, line_len, Dir::W);
    tilt(&mut rocks1, line_count, line_len, Dir::S);
    tilt(&mut rocks1, line_count, line_len, Dir::E);
    let load = get_load(&rocks1, line_count);

    let mut cycles = 1usize;
    let target_cycles = 1000000000usize;
    let mut part2 = 0;

    for _ in 0..200 {
        tilt(&mut rocks1, line_count, line_len, Dir::N);
        tilt(&mut rocks1, line_count, line_len, Dir::W);
        tilt(&mut rocks1, line_count, line_len, Dir::S);
        tilt(&mut rocks1, line_count, line_len, Dir::E);
        cycles += 1;

        if (cycles >= 141) & (((target_cycles - cycles) % 14) == 0) {
            part2 = get_load(&rocks1, line_count);
            break;
        }
    }

    println!("Part 1: {part1} Part 2: {part2}");
}

enum Dir {
    N,
    S,
    E,
    W,
}

fn tilt(rocks: &mut Vec<Rock>, rows: usize, cols: usize, dir: Dir) {
    let mut stack_top: HashMap<_, _> = match dir {
        Dir::N => rocks
            .iter()
            .filter(|rock| rock.row == 0)
            .map(|rock| (rock.col, 0))
            .collect(),
        Dir::S => rocks
            .iter()
            .filter(|rock| rock.row == (rows - 1))
            .map(|rock| (rock.col, rows - 1))
            .collect(),
        Dir::E => rocks
            .iter()
            .filter(|rock| rock.col == (cols - 1))
            .map(|rock| (rock.row, cols - 1))
            .collect(),
        Dir::W => rocks
            .iter()
            .filter(|rock| rock.col == 0)
            .map(|rock| (rock.row, 0))
            .collect(),
    };

    let steps: Vec<usize> = match dir {
        Dir::N => (1..rows).collect(),
        Dir::S => (0..(rows - 1)).rev().collect(),
        Dir::E => (0..(cols - 1)).rev().collect(),
        Dir::W => (1..cols).collect(),
    };

    for line in steps.into_iter() {
        let to_move: Vec<Rock> = rocks
            .extract_if(|rock| {
                let loc = match dir {
                    Dir::N | Dir::S => (rock.row, rock.col),
                    _ => (rock.col, rock.row),
                };

                if line != loc.0 {
                    return false;
                }

                if !rock.moves {
                    let stack = stack_top.entry(loc.1).or_insert(loc.0);
                    *stack = loc.0;
                    return false;
                }
                let mut space_to_move = true;
                if let Some(stop) = stack_top.get(&loc.1) {
                    let next_dir_of_gravity = match dir {
                        Dir::E | Dir::S => loc.0 + 1,
                        Dir::W | Dir::N => loc.0 - 1,
                    };
                    if *stop == next_dir_of_gravity {
                        space_to_move = false;
                    }
                    match dir {
                        Dir::E | Dir::S => stack_top.insert(loc.1, stop - 1),
                        Dir::W | Dir::N => stack_top.insert(loc.1, stop + 1),
                    };
                } else {
                    match dir {
                        Dir::E => stack_top.insert(loc.1, cols - 1),
                        Dir::S => stack_top.insert(loc.1, rows - 1),
                        Dir::W | Dir::N => stack_top.insert(loc.1, 0),
                    };
                };

                space_to_move
            })
            .collect();

        let mut to_move = to_move
            .into_iter()
            .map(|rock| match dir {
                Dir::N | Dir::S => Rock {
                    row: *stack_top.get(&rock.col).unwrap(),
                    col: rock.col,
                    moves: rock.moves,
                },
                Dir::E | Dir::W => Rock {
                    row: rock.row,
                    col: *stack_top.get(&rock.row).unwrap(),
                    moves: rock.moves,
                },
            })
            .collect();

        rocks.append(&mut to_move);
    }
}
