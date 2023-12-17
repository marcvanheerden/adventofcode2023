use std::{cmp::Ordering, collections::HashMap};
use tokio::sync::mpsc::Receiver;

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
    run: u8,
}

#[derive(Debug, Clone)]
struct Progress {
    heatloss: u32,
    location: (usize, usize),
    direction: Dir,
    run: u8,
}

impl Ord for Progress {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.heatloss).cmp(&(other.heatloss))
    }
}

impl PartialOrd for Progress {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Progress {
    fn eq(&self, other: &Self) -> bool {
        (self.heatloss) == (other.heatloss)
    }
}

impl Eq for Progress {}

#[derive(Debug, Clone)]
struct HeatMap {
    heatloss: Vec<u32>,
    row_bound: usize,
    col_bound: usize,
}

impl HeatMap {
    fn next_steps(&self, progress: &Progress, min_run: u8, max_run: u8) -> Vec<Progress> {
        // removes option that would lead to too many straight steps
        self.move_options(&progress.location, progress.direction)
            .into_iter()
            .filter(|(_, _, dir)| {
                let start = progress.location == (0, 0);
                let turning_too_soon = (progress.run < min_run) & (*dir != progress.direction);
                let straight_too_long = (progress.run >= max_run) & (*dir == progress.direction);
                !(turning_too_soon | straight_too_long) | start
            })
            .map(|(loc0, loc1, dir)| {
                let run = if dir == progress.direction {
                    progress.run + 1
                } else {
                    1
                };

                Progress {
                    location: (loc0, loc1),
                    heatloss: progress.heatloss + self.get_loc((loc0, loc1)), // add new heatloss
                    direction: dir,
                    run,
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

    async fn find_best_path(&self, start: Progress, min_run: u8, max_run: u8) -> u32 {
        let mut queue = vec![start];
        let mut best: HashMap<Bearing, u32> = HashMap::new();
        let mut arrived = Vec::new();

        loop {
            // keep only the best heatloss for a given state
            // sorting beforehand allows us to update the best heatloss HashMap
            // and use it for filtering in one pass
            // sorting is relatively cheap as it stays in a nearly sorted state
            queue.sort_unstable();
            queue.retain(|prog| {
                let prog_ = prog.clone();
                let bearing = Bearing {
                    x: prog_.location.1,
                    y: prog_.location.0,
                    dir: prog_.direction,
                    run: prog_.run,
                };
                let entry = best.entry(bearing).or_insert(u32::MAX);
                if *entry > prog.heatloss {
                    *entry = prog.heatloss;
                    true
                } else {
                    false
                }
            });

            let mut next_steps = Vec::new();
            for progress in queue.iter() {
                if (progress.location == (self.row_bound, self.col_bound))
                    & (progress.run >= min_run)
                {
                    arrived.push(progress.clone());
                    continue;
                }

                for step in self.next_steps(progress, min_run, max_run) {
                    next_steps.push(step);
                }
            }

            if next_steps.is_empty() {
                break;
            }

            queue = next_steps;
        }

        if let Some(output) = arrived.iter().map(|p| p.heatloss).min() {
            return output;
        }

        0
    }
}

pub async fn solve(mut rx: Receiver<(usize, String)>) {
    let mut holder = Vec::new();
    while let Some((line_no, line)) = rx.recv().await {
        holder.push((
            line_no,
            line.chars()
                .map(|c| c.to_string().parse::<u32>().unwrap())
                .collect::<Vec<u32>>(),
        ));
    }
    holder.sort_by_key(|f| f.0);
    let mut heatloss = Vec::new();
    let row_bound = holder.len() - 1;
    let col_bound = holder[0].1.len() - 1;

    for (_, mut line) in holder.into_iter() {
        heatloss.append(&mut line);
    }

    let map1 = HeatMap {
        heatloss,
        row_bound,
        col_bound,
    };

    let map2 = map1.clone();

    let start1 = Progress {
        location: (0, 0),
        heatloss: 0,
        direction: Dir::S,
        run: 0,
    };

    let start2 = start1.clone();

    let task1 = tokio::spawn(async move { map1.find_best_path(start1, 1, 3).await });
    let task2 = tokio::spawn(async move { map2.find_best_path(start2, 4, 10).await });
    let part1 = task1.await.unwrap();
    let part2 = task2.await.unwrap();

    println!("Part 1: {} Part 2: {}", part1, part2);
}
