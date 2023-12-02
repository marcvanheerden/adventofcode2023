use std::str::FromStr;
use tokio::sync::mpsc::Receiver;

struct CubeTally {
    red: usize,
    green: usize,
    blue: usize,
}

impl CubeTally {
    fn sufficient(&self, other: &CubeTally) -> bool {
        (self.red >= other.red) & (self.green >= other.green) & (self.blue >= other.blue)
    }
}

impl FromStr for CubeTally {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tally = CubeTally {
            red: 0,
            green: 0,
            blue: 0,
        };

        for colour_pair in s.split(", ") {
            let (count, colour) = colour_pair.trim().split_once(' ').unwrap();
            match colour {
                "red" => tally.red = count.parse::<usize>().unwrap(),
                "green" => tally.green = count.parse::<usize>().unwrap(),
                "blue" => tally.blue = count.parse::<usize>().unwrap(),
                _ => panic!("invalid colour name input"),
            };
        }

        Ok(tally)
    }
}

async fn calc_line(line: String) -> (u32, usize) {
    let ref_cubes = CubeTally {
        red: 12,
        green: 13,
        blue: 14,
    };

    let (game, draws) = line.split_once(':').unwrap();

    let cube_tallies: Vec<CubeTally> = draws
        .split(';')
        .map(|s| CubeTally::from_str(s).expect("couldn't parse tally"))
        .collect();

    let all_valid = cube_tallies.iter().all(|c| ref_cubes.sufficient(c));

    let score = if all_valid {
        game.trim()
            .split_once(' ')
            .unwrap()
            .1
            .parse::<u32>()
            .unwrap()
    } else {
        0
    };

    let mut min_red = 0;
    let mut min_blue = 0;
    let mut min_green = 0;

    for tally in cube_tallies.iter() {
        min_red = std::cmp::max(min_red, tally.red);
        min_blue = std::cmp::max(min_blue, tally.blue);
        min_green = std::cmp::max(min_green, tally.green);
    }

    (score, min_red * min_blue * min_green)
}

pub async fn solve(mut rx: Receiver<String>) {
    let mut tasks = Vec::new();

    while let Some(line) = rx.recv().await {
        let task = tokio::spawn(async move { calc_line(line).await });
        tasks.push(task);
    }

    let mut valid_score_part1 = 0u32;
    let mut total_power_part2 = 0usize;
    for task in tasks {
        if let Ok((score, power)) = task.await {
            valid_score_part1 += score;
            total_power_part2 += power;
        }
    }

    println!("Part 1: {valid_score_part1} Part 2: {total_power_part2}");
}
