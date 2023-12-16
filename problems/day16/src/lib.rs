use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Copy)]
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

#[derive(Debug, Clone, Eq, PartialEq, Hash, Copy)]
struct Beam {
    location: (usize, usize),
    direction: Dir,
}

impl Beam {
    fn hit_mirror(&mut self, mirror: &Mirror) -> Option<Self> {
        match (*mirror, &self.direction) {
            (Mirror::Hori, Dir::N | Dir::S) => {
                self.direction = Dir::W;
                let mut extra_beam = *self;
                extra_beam.direction = Dir::E;
                return Some(extra_beam);
            }
            (Mirror::Vert, Dir::E | Dir::W) => {
                self.direction = Dir::N;
                let mut extra_beam = *self;
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
            if let Some(new_beam) = self.hit_mirror(&mirror) {
                return (true, Some(new_beam));
            }
        }

        (true, None)
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

    let beam = Beam {
        location: (0, 0),
        direction: Dir::E,
    };

    let part1 = trace_beam(
        beam,
        Arc::new(mirrors.clone()),
        Arc::new((row_bound, col_bound)),
    )
    .await;
    let part2 = best_trace_beam(Arc::new(mirrors), Arc::new((row_bound, col_bound))).await;

    println!("Part 1: {part1} Part 2: {part2} ");
}

async fn best_trace_beam(
    mirrors: Arc<HashMap<(usize, usize), Mirror>>,
    bounds: Arc<(usize, usize)>,
) -> usize {
    let mut beams = Vec::new();
    for row in 0..=bounds.0 {
        let beam1 = Beam {
            location: (row, 0),
            direction: Dir::E,
        };
        let beam2 = Beam {
            location: (row, bounds.1),
            direction: Dir::W,
        };
        beams.push(beam1);
        beams.push(beam2);
    }

    for col in 0..=bounds.1 {
        let beam1 = Beam {
            location: (0, col),
            direction: Dir::S,
        };
        let beam2 = Beam {
            location: (bounds.0, col),
            direction: Dir::N,
        };
        beams.push(beam1);
        beams.push(beam2);
    }
    let mut tasks = Vec::new();

    for beam in beams.into_iter() {
        let mirrors_clone = Arc::clone(&mirrors);
        let bounds_clone = Arc::clone(&bounds);
        let task = tokio::spawn(async move { trace_beam(beam, mirrors_clone, bounds_clone).await });
        tasks.push(task);
    }

    let mut top_score = 0usize;
    for task in tasks {
        if let Ok(score) = task.await {
            top_score = std::cmp::max(top_score, score);
        }
    }

    top_score
}

async fn trace_beam(
    start: Beam,
    mirrors: Arc<HashMap<(usize, usize), Mirror>>,
    bounds: Arc<(usize, usize)>,
) -> usize {
    let mut start = start;
    if let Some(mirror) = mirrors.get(&start.location) {
        start.hit_mirror(mirror);
    }

    let mut queue = vec![start];
    let mut history = HashSet::new();
    history.insert(start);

    loop {
        let mut next_steps = Vec::new();

        for beam in queue.iter_mut() {
            let (valid, extra_beam) = beam.step(mirrors.clone(), bounds.clone());
            if !valid | history.contains(beam) {
                continue;
            }
            history.insert(*beam);
            next_steps.push(*beam);

            if let Some(new_beam) = extra_beam {
                next_steps.push(new_beam);
                history.insert(new_beam);
            }
        }

        if next_steps.is_empty() {
            break;
        } else {
            queue = next_steps;
        }
    }

    let locations: HashSet<_> = history.into_iter().map(|b| b.location).collect();

    locations.len()
}
