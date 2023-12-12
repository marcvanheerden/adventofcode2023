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

#[derive(Debug, Clone, PartialEq)]
struct RunCalc {
    runs: Vec<u8>, // current runs calculated
    open: bool,    // whether the last run is still open (could increase)
    springs: Vec<Spring>,
}

impl RunCalc {
    fn new(springs: Vec<Spring>) -> Self {
        Self {
            runs: Vec::new(),
            open: false,
            // reverse the springs so we can pop off springs as we go
            springs: springs.into_iter().rev().collect(),
        }
    }

    fn valid(&self, target: &[u8]) -> bool {
        if self.springs.is_empty() {
            return self.runs == target;
        }

        if self.open {
            let matches: Vec<_> = self
                .runs
                .iter()
                .zip(target.iter())
                .map(|(s, t)| s == t)
                .collect();

            let closed = matches.iter().rev().skip(1).all(|&b| b);

            if self.runs.len() > target.len() {
                return false;
            }

            let run_len = self.runs.len() - 1;
            let open = target[run_len] >= self.runs[run_len];

            return open & closed;
        }

        self.runs.iter().zip(target.iter()).all(|(s, t)| s == t)
    }

    fn assess(&mut self) {
        let next_spring = self.springs.pop().expect("assessing empty spring list");
        match (self.open, next_spring) {
            (true, Spring::Operational) => {
                let run_length = self.runs.len() - 1;
                self.runs[run_length] += 1;
            }
            (true, Spring::Damaged) => {
                self.open = false;
            }
            (false, Spring::Operational) => {
                self.open = true;
                self.runs.push(1);
            }
            (_, _) => (),
        }
    }

    fn step(&self) -> Vec<Self> {
        // terminal condition?

        if self.springs.last().expect("one step too far") != &Spring::Unknown {
            let mut out = self.clone();
            out.assess();
            return vec![out];
        }

        let mut out1 = self.clone();
        out1.springs.pop();
        out1.springs.push(Spring::Operational);
        out1.assess();

        let mut out2 = self.clone();
        out2.springs.pop();
        out2.springs.push(Spring::Damaged);
        out2.assess();

        return vec![out1, out2];
    }

    fn collapse(&self, _other: &Self) -> Vec<Self> {
        Vec::new()
    }
}

async fn calc_line(line: &str) -> (usize, usize) {
    let (springs_str, counts_str) = line.split_once(' ').unwrap();

    let springs: Vec<Spring> = springs_str.chars().map(Spring::new).collect();
    let counts: Vec<u8> = counts_str
        .split(',')
        .map(|s| s.parse::<u8>().unwrap())
        .collect();

    //let mut springs2: Vec<_> = (0..PART2_SCALE)
    //    .map(|_| {
    //        let mut s = springs.clone();
    //        s.push(Spring::Unknown);
    //        s
    //    })
    //    .flatten()
    //    .collect();

    //springs2 = springs2.into_iter().rev().skip(1).rev().collect();

    //let counts2: Vec<_> = (0..PART2_SCALE).map(|_| counts.clone()).flatten().collect();

    //(options(springs, counts), options(springs2, counts2))

    let length = springs.len();
    let mut run_calcs = vec![RunCalc::new(springs)];

    for _ in 0..length {
        run_calcs = run_calcs
            .into_iter()
            .map(|rc| rc.step())
            .flatten()
            .filter(|rc| rc.valid(&counts))
            .collect();
    }

    (run_calcs.len(), 0)
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
