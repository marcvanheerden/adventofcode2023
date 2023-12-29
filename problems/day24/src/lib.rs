use tokio::sync::mpsc::Receiver;

#[derive(Debug, Clone, Eq, PartialEq)]
struct Hail {
    position: [isize; 3],
    velocity: [isize; 3],
}

impl Hail {
    async fn new(input: &str) -> Self {
        let parts: Vec<_> = input
            .split([',', '@'])
            .map(|s| s.trim().parse::<isize>().unwrap())
            .collect();

        Hail {
            position: [parts[0], parts[1], parts[2]],
            velocity: [parts[3], parts[4], parts[5]],
        }
    }

    fn get_slope_constant(&self) -> Option<(f64, f64)> {
        if self.velocity[0] == 0 {
            return None;
        }

        let m = self.velocity[1] as f64 / self.velocity[0] as f64;
        let c = self.position[1] as f64 - (self.position[0] as f64) * m;

        Some((m, c))
    }

    fn will_collide_xy(&self, other: &Self) -> Option<[f64; 2]> {
        let mc1 = self.get_slope_constant();
        let mc2 = other.get_slope_constant();

        match (mc1, mc2) {
            // neither of the lines are vertical
            (Some((m1, c1)), Some((m2, c2))) => {
                if (m1 - m2).abs() < 0.0000000001 {
                    // lines are parallel
                    return None;
                }

                let x = (c2 - c1) / (m1 - m2);
                let y = m1 * x + c1;

                let self_future =
                    (x - self.position[0] as f64).signum() as isize == self.velocity[0].signum();
                let other_future =
                    (x - other.position[0] as f64).signum() as isize == other.velocity[0].signum();

                if self_future & other_future {
                    Some([x, y])
                } else {
                    None
                }
            }
            // self is the only vertical line
            (None, Some((m2, c2))) => {
                let x = self.position[0] as f64;
                let y = x * m2 + c2;

                let self_future =
                    (y - self.position[1] as f64).signum() as isize == self.velocity[1].signum();
                let other_future =
                    (x - other.position[0] as f64).signum() as isize == other.velocity[0].signum();

                if self_future & other_future {
                    Some([x, y])
                } else {
                    None
                }
            }
            // other is the only vertical line
            (Some((m1, c1)), None) => {
                let x = other.position[0] as f64;
                let y = x * m1 + c1;

                let self_future =
                    (x - self.position[0] as f64).signum() as isize == self.velocity[0].signum();
                let other_future =
                    (y - other.position[1] as f64).signum() as isize == other.velocity[1].signum();

                if self_future & other_future {
                    Some([x, y])
                } else {
                    None
                }
            }
            // both vertical lines
            (None, None) => None,
        }
    }
}

fn wedge2(vec1: &[f64], vec2: &[f64]) -> Vec<f64> {
    vec![
        vec1[0] * vec2[1] - vec1[1] * vec2[0],
        vec1[1] * vec2[2] - vec1[2] * vec2[1],
        vec1[2] * vec2[0] - vec1[0] * vec2[2],
    ]
}

fn to_float(arr: &[isize; 3]) -> Vec<f64> {
    arr.iter().map(|v| *v as f64).collect()
}

fn determinant<I: AsRef<[f64]>>(vec1: I, vec2: I, vec3: I) -> f64 {
    let vec1 = vec1.as_ref();
    let vec2 = vec2.as_ref();
    let vec3 = vec3.as_ref();

    vec1[0] * vec2[1] * vec3[2] + vec1[1] * vec2[2] * vec3[0] + vec1[2] * vec2[0] * vec3[1]
        - vec1[0] * vec2[2] * vec3[1]
        - vec1[1] * vec2[0] * vec3[2]
        - vec1[2] * vec2[1] * vec3[0]
}

fn subtract(vec1: &[isize; 3], vec2: &[isize; 3]) -> Vec<f64> {
    vec1.iter()
        .zip(vec2.iter())
        .map(|(v1, v2)| (*v1 - *v2) as f64)
        .collect()
}

// with credit to https://github.com/apprenticewiz/adventofcode/blob/main/2023/rust/day24b/src/main.rs
// and https://openstax.org/books/college-algebra-2e/pages/7-8-solving-systems-with-cramers-rule
fn find_collider(stones: &[Hail]) -> isize {
    let row1 = wedge2(
        &subtract(&stones[0].velocity, &stones[1].velocity),
        &subtract(&stones[0].position, &stones[1].position),
    );

    let row2 = wedge2(
        &subtract(&stones[0].velocity, &stones[2].velocity),
        &subtract(&stones[0].position, &stones[2].position),
    );

    let row3 = wedge2(
        &subtract(&stones[1].velocity, &stones[2].velocity),
        &subtract(&stones[1].position, &stones[2].position),
    );

    let lhs: Vec<Vec<f64>> = vec![
        vec![row1[0], row2[0], row3[0]],
        vec![row1[1], row2[1], row3[1]],
        vec![row1[2], row2[2], row3[2]],
    ];

    let rhs: Vec<f64> = vec![
        -determinant(
            &to_float(&stones[0].position),
            &to_float(&stones[0].velocity),
            &to_float(&stones[1].position),
        ) - determinant(
            &to_float(&stones[1].position),
            &to_float(&stones[1].velocity),
            &to_float(&stones[0].position),
        ),
        -determinant(
            &to_float(&stones[0].position),
            &to_float(&stones[0].velocity),
            &to_float(&stones[2].position),
        ) - determinant(
            &to_float(&stones[2].position),
            &to_float(&stones[2].velocity),
            &to_float(&stones[0].position),
        ),
        -determinant(
            &to_float(&stones[1].position),
            &to_float(&stones[1].velocity),
            &to_float(&stones[2].position),
        ) - determinant(
            &to_float(&stones[2].position),
            &to_float(&stones[2].velocity),
            &to_float(&stones[1].position),
        ),
    ];

    let lhs_det = determinant(&lhs[0], &lhs[1], &lhs[2]);
    let x_det = determinant(&rhs, &lhs[1], &lhs[2]);
    let y_det = determinant(&lhs[0], &rhs, &lhs[2]);
    let z_det = determinant(&lhs[0], &lhs[1], &rhs);

    let x = (x_det / lhs_det).round() as isize;
    let y = (y_det / lhs_det).round() as isize;
    let z = (z_det / lhs_det).round() as isize;

    x + y + z
}

pub async fn solve(mut rx: Receiver<String>) {
    let mut tasks = Vec::new();
    while let Some(line) = rx.recv().await {
        if line.is_empty() {
            continue;
        }
        let task = tokio::spawn(async move { Hail::new(&line).await });
        tasks.push(task);
    }

    let mut stones = Vec::new();
    for task in tasks {
        if let Ok(hailstone) = task.await {
            stones.push(hailstone);
        }
    }

    let test_area = 200000000000000f64..=400000000000000f64;
    let mut part1 = 0usize;
    for (idx1, stone1) in stones.iter().enumerate() {
        for (idx2, stone2) in stones.iter().enumerate() {
            if idx1 >= idx2 {
                continue;
            }
            if let Some(coll) = stone1.will_collide_xy(stone2) {
                if test_area.contains(&coll[0]) & test_area.contains(&coll[1]) {
                    part1 += 1;
                }
            }
        }
    }

    let part2 = find_collider(&stones);

    println!("Part 1: {} Part 2: {}", part1, part2);
}
