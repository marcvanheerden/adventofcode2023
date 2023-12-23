use std::collections::HashSet;
use tokio::sync::mpsc::Receiver;

#[derive(Debug, Clone, Copy)]
struct Interval {
    start: usize,
    end: usize,
}

impl Interval {
    fn overlap(&self, other: &Self) -> bool {
        (self.end >= other.start) & (other.end >= self.start)
    }
}

#[derive(Debug, Clone)]
struct Brick {
    number: usize,
    supported_by: Vec<usize>,
    x: Interval,
    y: Interval,
    z: Interval,
}

async fn parse_line(line_no: usize, line: &str) -> Brick {
    let splits: Vec<_> = line
        .trim()
        .split(['~', ','])
        .map(|s| s.parse::<usize>().unwrap())
        .collect();

    let x = Interval {
        start: std::cmp::min(splits[0], splits[3]),
        end: std::cmp::max(splits[0], splits[3]),
    };
    let y = Interval {
        start: std::cmp::min(splits[1], splits[4]),
        end: std::cmp::max(splits[1], splits[4]),
    };
    let z = Interval {
        start: std::cmp::min(splits[2], splits[5]),
        end: std::cmp::max(splits[2], splits[5]),
    };

    Brick {
        number: line_no,
        supported_by: Vec::new(),
        x,
        y,
        z,
    }
}

fn removable_bricks(bricks: &[Brick]) -> (usize, usize) {
    let mut bricks = bricks.to_vec();
    bricks.sort_by_key(|b| (b.z.start, b.x.start, b.y.start));

    // drop the first brick
    bricks[0].z.start -= bricks[0].z.start;
    bricks[0].z.end -= bricks[0].z.start;

    for num in 1..bricks.len() {
        let x = bricks[num].x;
        let y = bricks[num].y;
        let z = bricks[num].z;

        let iter = bricks
            .iter()
            .take(num)
            .filter(|b| b.x.overlap(&x) & b.y.overlap(&y));

        let highest = if let Some(val) = iter.clone().map(|b| b.z.end).max() {
            val
        } else {
            0
        };

        bricks[num].supported_by = iter
            .filter(|b| b.z.end == highest)
            .map(|b| b.number)
            .collect();

        let drop = z.start - highest - 1;
        bricks[num].z.start -= drop;
        bricks[num].z.end -= drop;
    }

    let sole_supporters: HashSet<_> = bricks
        .iter()
        .filter(|b| b.supported_by.len() == 1)
        .flat_map(|b| b.supported_by.clone())
        .collect();

    (bricks.len() - sole_supporters.len(), collapses(&bricks))
}

fn collapses(bricks: &[Brick]) -> usize {
    let mut total_collapses = 0;
    for brick in bricks.iter() {
        let mut collapsed = vec![brick.number];

        loop {
            let more_collapse: Vec<_> = bricks
                .iter()
                // not already collapsed
                .filter(|b| !collapsed.contains(&b.number))
                // has any supporting bricks
                .filter(|b| !b.supported_by.is_empty())
                // all supports have already collapsed
                .filter(|b| b.supported_by.iter().all(|num| collapsed.contains(num)))
                .map(|b| b.number)
                .collect();

            if more_collapse.is_empty() {
                break;
            } else {
                for brick_no in more_collapse.into_iter() {
                    collapsed.push(brick_no);
                }
            }
        }

        total_collapses += collapsed.len() - 1;
    }
    total_collapses
}

pub async fn solve(mut rx: Receiver<(usize, String)>) {
    let mut tasks = Vec::new();
    while let Some((line_no, line)) = rx.recv().await {
        if line.is_empty() {
            continue;
        }
        let task = tokio::spawn(async move { parse_line(line_no, &line).await });
        tasks.push(task);
    }

    let mut bricks = Vec::new();
    for task in tasks {
        if let Ok(brick) = task.await {
            bricks.push(brick);
        }
    }

    let (part1, part2) = removable_bricks(&bricks);
    println!("Part 1: {} Part 2: {}", part1, part2,);
}
