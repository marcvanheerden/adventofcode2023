use futures::future::{BoxFuture, FutureExt};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::{thread, time};
use tokio::sync::mpsc::Receiver;
use tokio::time::sleep;

#[derive(Debug, Clone, Eq, PartialEq)]
enum Dir {
    N,
    E,
    S,
    W,
}

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
enum Mirror {
    Hori,
    Vert,
    Forw,
    Back,
}

impl Mirror {
    fn new(chr: char) -> Option<Self> {
        match chr {
            '-' => Some(Self::Hori),
            '|' => Some(Self::Vert),
            '/' => Some(Self::Forw),
            '\\' => Some(Self::Back),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct Beam {
    location: (usize, usize),
    direction: Dir,
}

impl Beam {
    fn hit_mirror(&mut self, mirror: Arc<Mirror>) -> Option<Self> {
        match (*mirror, &self.direction) {
            (Mirror::Hori, Dir::N | Dir::S) => {
                self.direction = Dir::W;
                let mut extra_beam = self.clone();
                extra_beam.direction = Dir::E;
                return Some(extra_beam);
            }
            (Mirror::Vert, Dir::E | Dir::W) => {
                self.direction = Dir::N;
                let mut extra_beam = self.clone();
                extra_beam.direction = Dir::S;
                return Some(extra_beam);
            }
            (Mirror::Forw, Dir::N) => self.direction = Dir::E,
            (Mirror::Forw, Dir::S) => self.direction = Dir::W,
            (Mirror::Forw, Dir::E) => self.direction = Dir::N,
            (Mirror::Forw, Dir::W) => self.direction = Dir::S,
            (Mirror::Back, Dir::N) => self.direction = Dir::W,
            (Mirror::Back, Dir::S) => self.direction = Dir::E,
            (Mirror::Back, Dir::E) => self.direction = Dir::S,
            (Mirror::Back, Dir::W) => self.direction = Dir::N,
            (_, _) => (),
        }

        None
    }

    fn out_of_bounds(&self, bounds: Arc<(usize, usize)>) -> bool {
        ((self.direction == Dir::N) & (self.location.0 == 0))
            | ((self.direction == Dir::S) & (self.location.0 == bounds.0))
            | ((self.direction == Dir::E) & (self.location.1 == bounds.1))
            | ((self.direction == Dir::W) & (self.location.1 == 0))
    }

    fn step(
        &mut self,
        mirrors: Arc<HashMap<(usize, usize), Mirror>>,
        bounds: Arc<(usize, usize)>,
    ) -> (bool, Option<Self>) {
        // check if next step is out of bounds
        if self.out_of_bounds(bounds) {
            return (false, None);
        }

        // move one step
        match self.direction {
            Dir::N => self.location.0 -= 1,
            Dir::S => self.location.0 += 1,
            Dir::E => self.location.1 += 1,
            Dir::W => self.location.1 -= 1,
        }

        // if a mirror is at the new location, potentially change direction
        // and possibly split into a new beam
        if let Some(&mirror) = mirrors.get(&self.location) {
            if let Some(new_beam) = self.hit_mirror(Arc::new(mirror)) {
                return (true, Some(new_beam));
            }
        }

        (true, None)
    }

    fn trace<'a>(
        &'a mut self,
        history: Vec<Self>,
        mirrors: Arc<HashMap<(usize, usize), Mirror>>,
        bounds: Arc<(usize, usize)>,
    ) -> BoxFuture<'a, Vec<(usize, usize)>> {
        async move {
            let mut path = vec![self.location];
            let mut history = history.clone();

            let mut subtraces = Vec::new();

            loop {
                history.push(self.clone());
                let (in_bounds, new_beam) = self.step(mirrors.clone(), bounds.clone());
                path.push(self.location);

                if let Some(mut beam) = new_beam {
                    let mirrors_clone = Arc::clone(&mirrors);
                    let bounds_clone = Arc::clone(&bounds);
                    let hist_clone = history.clone();

                    let task = tokio::spawn(async move {
                        beam.trace(hist_clone, mirrors_clone, bounds_clone).await
                    });
                    subtraces.push(task);
                }

                if !in_bounds | history.contains(self) {
                    break;
                }
            }

            for task in subtraces {
                if let Ok(mut new_path) = task.await {
                    path.append(&mut new_path);
                }
            }

            path
        }
        .boxed()
    }
}

pub async fn solve(mut rx: Receiver<(usize, String)>) {
    let mut tasks = Vec::new();

    let mut row_bound = 0usize;
    let mut col_bound = 0usize;
    while let Some((line_no, line)) = rx.recv().await {
        row_bound = std::cmp::max(row_bound, line_no);
        col_bound = std::cmp::max(col_bound, line.len() - 1);
        let task = tokio::spawn(async move {
            line.chars()
                .enumerate()
                .filter_map(|(col, chr)| {
                    if let Some(mirror) = Mirror::new(chr) {
                        return Some(((line_no, col), mirror));
                    }
                    None
                })
                .collect::<Vec<((usize, usize), Mirror)>>()
        });
        tasks.push(task);
    }

    let mut mirrors = HashMap::new();

    for task in tasks {
        if let Ok(new_mirrors) = task.await {
            for (loc, mirror) in new_mirrors.into_iter() {
                mirrors.insert(loc, mirror);
            }
        }
    }

    let mut beam = Beam {
        location: (0, 0),
        direction: Dir::E,
    };

    if let Some(mirror) = mirrors.get(&beam.location) {
        beam.hit_mirror(Arc::new(*mirror));
    }

    let pos_visited: HashSet<(usize, usize)> = beam
        .trace(
            Vec::new(),
            Arc::new(mirrors),
            Arc::new((row_bound, col_bound)),
        )
        .await
        .into_iter()
        .collect();

    for y in 0..row_bound {
        println!();
        for x in 0..col_bound {
            if pos_visited.contains(&(y, x)) {
                print!("{}", '#');
            } else {
                print!("{}", '.');
            }
        }
    }

    let part1 = pos_visited.len();

    println!("Part 1: {part1} Part 2: ");
}

//######....
//.#...#....
//.#...#####
//.#...##...
//.#...##...
//.#...##...
//.#..####..
//########..
//.#######..
//.#...#.#..
