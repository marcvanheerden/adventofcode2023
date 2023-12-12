use tokio::sync::mpsc::Receiver;

const PART2_SCALE: usize = 5;

#[derive(Debug, Clone, PartialEq, Copy)]
enum Spring {
    Operational,
    Damaged,
    Unknown,
}

impl Spring {
    fn new(chr: char) -> Self {
        match chr {
            '?' => Self::Unknown,
            '.' => Self::Damaged,
            '#' => Self::Operational,
            _ => panic!("invalid spring input"),
        }
    }
}

async fn calc_line(line: &str) -> (usize, usize) {
    let (springs_str, counts_str) = line.split_once(' ').unwrap();

    let springs: Vec<Spring> = springs_str.chars().map(Spring::new).collect();
    let counts: Vec<u8> = counts_str
        .split(',')
        .map(|s| s.parse::<u8>().unwrap())
        .collect();

    let mut springs2: Vec<_> = (0..PART2_SCALE)
        .map(|_| {
            let mut s = springs.clone();
            s.push(Spring::Unknown);
            s
        })
        .flatten()
        .collect();

    springs2 = springs2.into_iter().rev().skip(1).rev().collect();

    let counts2: Vec<_> = (0..PART2_SCALE).map(|_| counts.clone()).flatten().collect();

    (options(springs, counts), options(springs2, counts2))
}

fn options(springs: Vec<Spring>, counts: Vec<u8>) -> usize {
    let unknowns = springs.iter().filter(|&s| s == &Spring::Unknown).count();

    let mut candidate_springs = vec![springs];
    for _ in 0..unknowns {
        candidate_springs = candidate_springs
            .iter()
            .map(|s| spring_options(s))
            .flatten()
            .filter(|v| check_runs(v, &counts))
            .collect();
    }

    candidate_springs.iter().count()
}

fn spring_options(springs: &[Spring]) -> Vec<Vec<Spring>> {
    let mut new_springs = Vec::new();

    for (idx, spring) in springs.iter().enumerate() {
        match spring {
            Spring::Unknown => {
                new_springs.push(springs.to_vec());
                new_springs.push(springs.to_vec());
                new_springs[0][idx] = Spring::Operational;
                new_springs[1][idx] = Spring::Damaged;
                break;
            }
            _ => (),
        }
    }

    new_springs
}

fn check_runs(springs: &[Spring], target: &[u8]) -> bool {
    // short-circuit reject spring patterns that don't match
    let mut operational = false;
    let mut target = target.iter();
    let mut count_buffer = 0u8;

    for (idx, spring) in springs.iter().enumerate() {
        let min_remaining_space: u8 = target.clone().skip(1).map(|x| x + 1).sum();

        if (springs.len() - idx) < min_remaining_space as usize {
            return false;
        }

        match (operational, *spring) {
            // stop checking if you hit an unknown
            (_, Spring::Unknown) => {
                if let Some(&next_target) = target.next() {
                    if next_target < count_buffer {
                        return false;
                    } else {
                        return true;
                    }
                } else {
                    return true;
                }
            }
            (false, Spring::Operational) => {
                operational = true;
                count_buffer += 1;
            }
            (true, Spring::Operational) => {
                count_buffer += 1;
            }
            (true, Spring::Damaged) => {
                if let Some(next_target) = target.next() {
                    if count_buffer != *next_target {
                        return false;
                    }
                } else {
                    return false;
                }
                operational = false;
                count_buffer = 0;
            }
            (_, _) => (),
        }
    }

    if count_buffer > 0 {
        if let Some(next_target) = target.next() {
            if count_buffer != *next_target {
                return false;
            }
        } else {
            return false;
        }
    }

    if target.next().is_some() {
        false
    } else {
        true
    }
}

pub async fn solve(mut rx: Receiver<String>) {
    let mut tasks = Vec::new();

    while let Some(line) = rx.recv().await {
        let task = tokio::spawn(async move { calc_line(&line).await });
        tasks.push(task);
    }

    let mut part1 = 0usize;
    let mut part2 = 0usize;
    for task in tasks {
        if let Ok((options1, options2)) = task.await {
            part1 += options1;
            part2 += options2;
        }
    }

    println!("Part 1: {part1} Part 2: {part2}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_checker() {
        let op = Spring::Operational;
        let da = Spring::Damaged;
        assert!(check_runs(&[da, op, op, da, op, da], &[2, 1]));
        assert!(!check_runs(&[da, op, op, da, op, da], &[2, 1, 1]));
        assert!(!check_runs(&[da, op, op, da, op, da], &[1, 2]));
        assert!(!check_runs(&[da, op, op, da, op, da], &[2]));
        assert!(check_runs(&[da, op, op, da, op, da, op], &[2, 1, 1]));
        assert!(!check_runs(&[da, op, op, da, op, da, op], &[2, 1, 2]));
        assert!(!check_runs(&[da, op, op, da, op, da, op], &[2, 1, 1, 2]));
        assert!(!check_runs(&[da, op, op, da, op, da, op], &[2, 1]));
    }
}
