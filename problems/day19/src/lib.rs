use std::collections::{HashMap, VecDeque};
use tokio::sync::mpsc::Receiver;

const RATINGS: usize = 4;
const MIN_RATING: usize = 1;
const MAX_RATING: usize = 4000;

#[derive(Debug, Clone, Copy)]
struct Interval {
    start: usize,
    end: usize,
}

#[derive(Debug, Clone)]
struct Gear {
    ratings: [usize; RATINGS],
}

#[derive(Debug, Clone)]
struct GearInterval {
    ratings: [Interval; RATINGS],
}

impl GearInterval {
    fn size(&self) -> usize {
        let mut output = 1;

        for interval in self.ratings.iter() {
            output *= interval.end + 1 - interval.start;
        }

        output
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Operation {
    LT,
    GT,
    ANY,
}

#[derive(Debug, Clone)]
struct Rule {
    rating: usize,
    operation: Operation,
    argument: usize,
    output: String,
}

impl Rule {
    fn apply(&self, gear: &Gear) -> Option<String> {
        let val = gear.ratings[self.rating];

        let condition = match self.operation {
            Operation::LT => val < self.argument,
            Operation::GT => val > self.argument,
            Operation::ANY => true,
        };

        if condition {
            Some(self.output.clone())
        } else {
            None
        }
    }
    fn interval_apply(&self, gearinterval: &mut GearInterval) -> Option<(String, GearInterval)> {
        // assumes the ANY operation is always the last rule
        match self.operation {
            Operation::LT => {
                let interval = gearinterval.ratings[self.rating];
                if self.argument >= interval.start {
                    let mut new_gearinterval = gearinterval.clone();
                    if self.argument <= interval.end {
                        new_gearinterval.ratings[self.rating].end = self.argument - 1;
                        gearinterval.ratings[self.rating].start = self.argument;
                        Some((self.output.clone(), new_gearinterval))
                    } else {
                        gearinterval.ratings[self.rating].start =
                            gearinterval.ratings[self.rating].end + 1;
                        Some((self.output.clone(), new_gearinterval))
                    }
                } else {
                    None
                }
            }
            Operation::GT => {
                let interval = gearinterval.ratings[self.rating];
                if self.argument <= interval.end {
                    let mut new_gearinterval = gearinterval.clone();
                    if self.argument >= interval.start {
                        new_gearinterval.ratings[self.rating].start = self.argument + 1;
                        gearinterval.ratings[self.rating].end = self.argument;
                        Some((self.output.clone(), new_gearinterval))
                    } else {
                        gearinterval.ratings[self.rating].start =
                            gearinterval.ratings[self.rating].end + 1;
                        Some((self.output.clone(), new_gearinterval))
                    }
                } else {
                    None
                }
            }
            Operation::ANY => Some((self.output.clone(), gearinterval.clone())),
        }
    }
}

#[derive(Debug)]
struct Pattern {
    rules: Vec<Rule>,
}

impl Pattern {
    fn apply(&self, gear: &Gear) -> String {
        for rule in self.rules.iter() {
            if let Some(next_step) = rule.apply(gear) {
                return next_step;
            }
        }
        panic!("full pattern without resolution");
    }

    fn interval_apply(&self, gearinterval: &GearInterval) -> Vec<(String, GearInterval)> {
        let mut output = Vec::new();
        let mut gearinterval = gearinterval.clone();
        for rule in self.rules.iter() {
            if gearinterval.size() == 0 {
                break;
            }
            if let Some(next) = rule.interval_apply(&mut gearinterval) {
                output.push(next);
            }
        }
        output
    }
}

#[derive(Debug)]
enum GearPattern {
    Gear(Gear),
    Pattern(String, Pattern),
}

async fn parse_line(line: &str) -> GearPattern {
    if line.starts_with('{') {
        let ratings: Vec<_> = line
            .trim()
            .split(['=', ',', '{', '}'])
            .filter_map(|s| s.parse::<usize>().ok())
            .collect();

        GearPattern::Gear(Gear {
            ratings: ratings.try_into().unwrap(),
        })
    } else {
        let mut split = line.trim().split(['{', ',', '}']);
        let name = split.next().unwrap();
        let mut rules = Vec::new();

        for rule_str in split {
            if rule_str.trim().is_empty() {
                continue;
            }
            if let Some((condition, output)) = rule_str.split_once(':') {
                let operation = if condition.contains('>') {
                    Operation::GT
                } else {
                    Operation::LT
                };

                let (rating_name, argument_str) = condition.split_once(['<', '>']).unwrap();
                let rating = match rating_name {
                    "x" => 0,
                    "m" => 1,
                    "a" => 2,
                    "s" => 3,
                    _ => panic!("incorrect input"),
                };

                let argument = argument_str.parse::<usize>().unwrap();

                rules.push(Rule {
                    rating,
                    operation,
                    argument,
                    output: output.into(),
                });
            } else {
                rules.push(Rule {
                    rating: 0,
                    operation: Operation::ANY,
                    argument: 0,
                    output: rule_str.into(),
                });
            }
        }

        GearPattern::Pattern(name.into(), Pattern { rules })
    }
}

pub async fn solve(mut rx: Receiver<String>) {
    let mut tasks = Vec::new();

    while let Some(line) = rx.recv().await {
        if line.is_empty() {
            continue;
        }

        let task = tokio::spawn(async move { parse_line(&line).await });
        tasks.push(task);
    }

    let mut gears: Vec<Gear> = Vec::new();
    let mut patterns: HashMap<String, Pattern> = HashMap::new();

    for task in tasks {
        if let Ok(gearpattern) = task.await {
            match gearpattern {
                GearPattern::Gear(gear) => {
                    gears.push(gear);
                }
                GearPattern::Pattern(name, pattern) => {
                    patterns.insert(name, pattern);
                }
            }
        }
    }

    let mut part1 = 0;

    for gear in gears.into_iter() {
        let mut pattern_name = "in".to_string();
        while !["A", "R"].contains(&pattern_name.as_str()) {
            pattern_name = patterns.get(&pattern_name).unwrap().apply(&gear);
            if pattern_name.as_str() == "A" {
                part1 += gear.ratings.iter().sum::<usize>();
            }
        }
    }

    let mut part2 = 0;

    let mut queue = VecDeque::from(vec![(
        "in".to_string(),
        GearInterval {
            ratings: [Interval {
                start: MIN_RATING,
                end: MAX_RATING,
            }; RATINGS],
        },
    )]);

    loop {
        let mut next_queue = VecDeque::new();

        for (pattern_name, gearinterval) in queue {
            for (next_pattern_name, child_gearinterval) in patterns
                .get(&pattern_name)
                .unwrap()
                .interval_apply(&gearinterval)
            {
                if next_pattern_name == "A" {
                    part2 += child_gearinterval.size();
                } else if next_pattern_name != "R" {
                    next_queue.push_back((next_pattern_name, child_gearinterval));
                }
            }
        }

        if next_queue.is_empty() {
            break;
        }
        queue = next_queue;
    }

    println!("Part 1: {} Part 2: {}", part1, part2);
}
