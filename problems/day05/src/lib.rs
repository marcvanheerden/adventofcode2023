use std::str::FromStr;
use tokio::sync::mpsc::Receiver;

#[derive(Debug, Clone, PartialEq)]
struct Interval {
    start: usize,
    len: usize,
}

impl Interval {
    fn compare(&self, other: &Self) -> (Option<Self>, Vec<Self>) {
        // calculate overlap if any and left over intervals
        let x1 = self.start;
        let x2 = self.start + self.len - 1;
        let y1 = other.start;
        let y2 = other.start + other.len - 1;

        if (y2 >= x1) & (y1 <= x2) {
            // has overlap
            let overlap1 = std::cmp::max(x1, y1);
            let overlap2 = std::cmp::min(x2, y2);
            let overlap = Some(Interval {
                start: overlap1,
                len: overlap2 + 1 - overlap1,
            });
            let mut left_over = Vec::new();

            if x1 < overlap1 {
                left_over.push(Interval {
                    start: x1,
                    len: overlap1 - x1,
                });
            }
            if x2 > overlap2 {
                left_over.push(Interval {
                    start: overlap2 + 1,
                    len: x2 - overlap2,
                });
            }

            return (overlap, left_over);
        }

        (None, Vec::new())
    }
}

#[derive(Debug, PartialEq, Clone)]
enum Commodity {
    Seed,
    Soil,
    Fertilizer,
    Water,
    Light,
    Temperature,
    Humidity,
    Location,
}

impl FromStr for Commodity {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let variant = match s {
            "seed" => Self::Seed,
            "soil" => Self::Soil,
            "fertilizer" => Self::Fertilizer,
            "water" => Self::Water,
            "light" => Self::Light,
            "temperature" => Self::Temperature,
            "humidity" => Self::Humidity,
            "location" => Self::Location,
            _ => panic!("invalid commodity input"),
        };

        Ok(variant)
    }
}

#[derive(Debug)]
struct Stage {
    comm: Commodity,
    values: Vec<usize>,
    intervals: Vec<Interval>,
}

impl FromStr for Stage {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (comm_str, values_str) = s.split_once("s: ").unwrap();
        let values: Vec<_> = values_str
            .trim()
            .split(' ')
            .map(|s| s.parse::<usize>().unwrap())
            .collect();

        let intervals = values
            .chunks(2)
            .map(|chunk| Interval {
                start: chunk[0],
                len: chunk[1],
            })
            .collect();

        Ok(Stage {
            comm: Commodity::from_str(comm_str).unwrap(),
            values,
            intervals,
        })
    }
}

#[derive(Debug)]
struct MapTable {
    in_: Commodity,
    out: Commodity,
    interchange: Vec<(usize, usize, usize)>,
}

impl MapTable {
    fn next_stage(&self, stage: &Stage) -> Option<Stage> {
        if self.in_ != stage.comm {
            return None;
        }

        let values = stage
            .values
            .iter()
            .map(|v| {
                for (out_start, int_start, int_len) in self.interchange.iter() {
                    if v >= int_start {
                        let diff = v.abs_diff(*int_start);
                        if diff < *int_len {
                            return out_start + diff;
                        }
                    }
                }
                *v
            })
            .collect::<Vec<usize>>();

        let mut queue = stage.intervals.clone();
        let mut to_add = Vec::new();
        let mut intervals = Vec::new();

        while !queue.is_empty() {
            'a: for interval in &queue {
                for (out_start, start, len) in self.interchange.iter() {
                    let (overlap, mut offcuts) = interval.compare(&Interval {
                        start: *start,
                        len: *len,
                    });
                    if let Some(overlap_) = overlap {
                        let mut new_interval = overlap_.clone();
                        if out_start > start {
                            new_interval.start += out_start - start;
                        } else {
                            new_interval.start -= start - out_start;
                        }
                        intervals.push(new_interval);
                        to_add.append(&mut offcuts);
                        continue 'a;
                    }
                }
                // default, no overlap found
                intervals.push(interval.clone());
            }
            queue.clear();
            queue.append(&mut to_add);
        }

        Some(Stage {
            comm: self.out.clone(),
            values,
            intervals,
        })
    }
}

impl FromStr for MapTable {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        let header = lines.next().unwrap();

        let (comm2comm, _) = header.split_once(' ').unwrap();
        let (comm1, comm2) = comm2comm.split_once("-to-").unwrap();

        let interchange = lines
            .map(|l| {
                let vals: Vec<usize> = l.split(' ').map(|v| v.parse::<usize>().unwrap()).collect();
                (vals[0], vals[1], vals[2])
            })
            .collect();

        Ok(MapTable {
            in_: Commodity::from_str(comm1).unwrap(),
            out: Commodity::from_str(comm2).unwrap(),
            interchange,
        })
    }
}

#[derive(Debug)]
enum Input {
    Stage(Stage),
    MapTable(MapTable),
}

impl FromStr for Input {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        let header = lines.next().unwrap();

        if header.contains("map") {
            Ok(Input::MapTable(MapTable::from_str(s).unwrap()))
        } else {
            Ok(Input::Stage(Stage::from_str(header).unwrap()))
        }
    }
}

pub async fn solve(mut rx: Receiver<String>) {
    let mut tasks = Vec::new();

    while let Some(line) = rx.recv().await {
        let task = tokio::spawn(async move { Input::from_str(&line).unwrap() });
        tasks.push(task);
    }

    let mut stage = Stage::from_str("seeds: 1 2").unwrap();
    let mut maptables = Vec::new();

    for task in tasks {
        if let Ok(input) = task.await {
            match input {
                Input::Stage(s) => stage = s,
                Input::MapTable(mt) => maptables.push(mt),
            }
        }
    }

    while stage.comm != Commodity::Location {
        for mt in maptables.iter() {
            if let Some(next_stage) = mt.next_stage(&stage) {
                stage = next_stage;
            }
        }
    }

    let part1 = stage.values.iter().min().unwrap();
    let part2 = stage.intervals.iter().map(|i| i.start).min().unwrap();
    println!("Part 1: {part1} Part 2: {part2}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interval_non_overlap() {
        let a = Interval { start: 1, len: 1 };
        let b = Interval { start: 2, len: 2 };

        let (overlap, offcuts) = a.compare(&b);
        assert!(overlap.is_none());
        assert!(offcuts.is_empty());
    }

    #[test]
    fn interval_contained() {
        let a = Interval { start: 1, len: 5 };
        let b = Interval { start: 2, len: 2 };

        let (overlap, offcuts) = a.compare(&b);
        assert_eq!(overlap, Some(Interval { start: 2, len: 2 }));
        assert_eq!(offcuts[0], Interval { start: 1, len: 1 });
        assert_eq!(offcuts[1], Interval { start: 4, len: 2 });
        assert_eq!(offcuts.len(), 2);
    }
}
