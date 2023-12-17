use std::{
    cmp::Ordering,
    collections::{HashMap, VecDeque},
};

const MAX_STRAIGHT: usize = 3;
const TOP_N: usize = 100000;
const MAX_WAIT: u8 = 240;
const FULL_HIST: bool = false;
const LOC_FACTOR: usize = 2;

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
enum Dir {
    N,
    E,
    S,
    W,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
struct Bearing {
    y: usize,
    x: usize,
    dir: Dir,
    last_moves: VecDeque<Dir>,
}

#[derive(Debug, Clone)]
struct Progress {
    location: (usize, usize),
    heatloss: u32,
    direction: Dir,
    last_moves: VecDeque<Dir>,
}

impl Progress {
    fn rank(&self) -> u32 {
        let location_score: u32 = ((self.location.0 + self.location.1) * LOC_FACTOR)
            .try_into()
            .unwrap();

        if location_score <= self.heatloss {
            self.heatloss - location_score
        } else {
            0
        }
    }
}

impl Ord for Progress {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.rank()).cmp(&(other.rank()))
    }
}

impl PartialOrd for Progress {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Progress {
    fn eq(&self, other: &Self) -> bool {
        (self.location, &self.heatloss) == (other.location, &other.heatloss)
    }
}

impl Eq for Progress {}

#[derive(Debug)]
struct HeatMap {
    heatloss: Vec<u32>,
    row_bound: usize,
    col_bound: usize,
}

impl HeatMap {
    fn next_steps(&self, progress: &Progress) -> Vec<Progress> {
        self.move_options(&progress.location, progress.direction)
            .into_iter()
            // removes option that would lead to too many straight steps
            .filter(|(_, _, dir)| {
                if FULL_HIST {
                    let recent: Vec<_> = progress
                        .last_moves
                        .iter()
                        .rev()
                        .take(MAX_STRAIGHT)
                        .collect();

                    recent != vec![dir; MAX_STRAIGHT]
                } else {
                    progress.last_moves != vec![*dir; MAX_STRAIGHT]
                }
            })
            .map(|(loc0, loc1, dir)| {
                // update last moves tracker for avoiding excessive straight steps
                let mut last_moves = progress.last_moves.clone();
                if (last_moves.len() >= (MAX_STRAIGHT)) & !FULL_HIST {
                    last_moves.pop_front();
                }
                last_moves.push_back(dir);

                Progress {
                    location: (loc0, loc1),
                    heatloss: progress.heatloss + self.get_loc((loc0, loc1)), // add new heatloss
                    direction: dir,
                    last_moves,
                }
            })
            .collect()
    }

    fn move_options(&self, location: &(usize, usize), dir: Dir) -> Vec<(usize, usize, Dir)> {
        // find possible moves for a given location and direction
        let mut moves = Vec::new();
        if (location.0 > 0) & (dir != Dir::S) {
            moves.push((location.0 - 1, location.1, Dir::N));
        }
        if (location.1 > 0) & (dir != Dir::E) {
            moves.push((location.0, location.1 - 1, Dir::W));
        }
        if (location.0 < self.row_bound) & (dir != Dir::N) {
            moves.push((location.0 + 1, location.1, Dir::S));
        }
        if (location.1 < self.col_bound) & (dir != Dir::W) {
            moves.push((location.0, location.1 + 1, Dir::E));
        }

        moves
    }

    fn get_loc(&self, location: (usize, usize)) -> u32 {
        // get the heatloss at a specific location
        // SAFETY: The caller must guarantee that location is within the bounds of
        unsafe {
            *self
                .heatloss
                .get_unchecked(location.0 * (self.col_bound + 1) + location.1)
        }
    }

    fn find_best_path(&self, start: Progress, end: (usize, usize)) -> u32 {
        let mut queue = vec![start];
        let mut wait = 0u8;
        let mut arrived = Vec::new();
        let mut has_arrived = false;
        let mut best: HashMap<Bearing, u32> = HashMap::new();

        loop {
            queue = queue
                .into_iter()
                .filter_map(|prog| {
                    let prog_ = prog.clone();
                    let bearing = Bearing {
                        x: prog_.location.1,
                        y: prog_.location.0,
                        dir: prog_.direction,
                        last_moves: prog_.last_moves,
                    };
                    let entry = best.entry(bearing).or_insert(u32::MAX);
                    if *entry > prog.heatloss {
                        *entry = prog.heatloss;
                        Some(prog)
                    } else {
                        None // a better path has come here before
                    }
                })
                .collect();

            // drop locations that lost way too much heat already
            queue.retain(|p| (p.heatloss as usize) <= (self.row_bound + self.col_bound) * 9);

            let mut next_steps = Vec::new();
            queue.sort_unstable();

            for progress in queue.iter().take(TOP_N) {
                if progress.location == end {
                    arrived.push(progress.clone());
                    has_arrived = true;
                    continue;
                }

                for step in self.next_steps(&progress) {
                    next_steps.push(step);
                }
            }

            // count full queue iterations since first arrival
            if has_arrived {
                wait += 1;
            }

            if wait >= MAX_WAIT {
                dbg!(&arrived
                    .iter()
                    .filter(|p| p.heatloss < 108)
                    .collect::<Vec<_>>());
                return arrived.iter().map(|p| p.heatloss).min().unwrap();
            }

            if next_steps.is_empty() {
                break;
            }

            queue = next_steps;
        }

        if let Some(output) = arrived.iter().map(|p| p.heatloss).min() {
            dbg!(&arrived
                .iter()
                .filter(|p| p.heatloss < 108)
                .collect::<Vec<_>>());
            return output;
        }

        0
    }
}

fn main() {
    let inputs = std::fs::read_to_string("input.txt").unwrap();
    let lines: Vec<_> = inputs.lines().filter(|l| !l.is_empty()).collect();
    let row_bound = lines.len() - 1;
    let col_bound = lines[0].len() - 1;
    let heatloss = lines
        .into_iter()
        .flat_map(|l| {
            l.trim()
                .chars()
                .map(|c| c.to_string().parse::<u32>().unwrap())
        })
        .collect();

    let map = HeatMap {
        heatloss,
        row_bound,
        col_bound,
    };

    let start = Progress {
        location: (0, 0),
        heatloss: 0,
        direction: Dir::S,
        last_moves: VecDeque::new(),
    };
    let part1 = map.find_best_path(start, (row_bound, col_bound));

    println!("Part 1: {} Part 2: ", part1);
}
