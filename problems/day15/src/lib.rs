use tokio::sync::mpsc::Receiver;

const EMPTYBUCKET: Vec<Lens> = Vec::new();

#[derive(Debug)]
struct Lens {
    label: String,
    focal_length: Option<usize>,
}

fn hash_section(section: &str) -> usize {
    let mut current_value = 0;
    for chr in section.trim().chars() {
        current_value += (chr as u8) as usize;
        current_value *= 17;
        current_value %= 256;
    }

    current_value
}

pub async fn solve(mut rx: Receiver<String>) {
    let mut tasks = Vec::new();

    while let Some(section) = rx.recv().await {
        let task = tokio::spawn(async move {
            let part1 = hash_section(&section);

            let lens = if section.trim().ends_with("-") {
                Lens {
                    label: section.trim().replace("-", ""),
                    focal_length: None,
                }
            } else {
                let (label, focal_length) = section.split_once('=').unwrap();
                Lens {
                    label: label.into(),
                    focal_length: focal_length.trim().parse::<usize>().ok(),
                }
            };
            let hash = hash_section(&lens.label);

            (part1, hash, lens)
        });
        tasks.push(task);
    }

    let mut total_score = 0;
    let mut lenses = Vec::new();

    for task in tasks {
        if let Ok((score, hash, lens)) = task.await {
            total_score += score;
            lenses.push((hash, lens));
        }
    }

    let mut buckets = [EMPTYBUCKET; 256];

    for (hash, lens) in lenses {
        match lens.focal_length {
            Some(_) => {
                if let Some(lens_) = buckets[hash].iter_mut().find(|l| l.label == lens.label) {
                    lens_.focal_length = lens.focal_length;
                } else {
                    buckets[hash].push(lens);
                }
            }
            None => {
                buckets[hash].retain(|l| l.label != lens.label);
            }
        }
    }

    let part2 = buckets
        .into_iter()
        .enumerate()
        .map(|(idx, bucket)| {
            bucket
                .into_iter()
                .enumerate()
                .map(|(idx1, lens)| (idx1 + 1) * lens.focal_length.unwrap())
                .sum::<usize>()
                * (idx + 1)
        })
        .sum::<usize>();

    println!("Part 1: {total_score} Part 2: {part2}");
}
