use tokio::sync::mpsc::Receiver;

//0 3 6 9 12 15
//1 3 6 10 15 21
//10 13 16 21 30 45
async fn calc_line(line: &str) -> (isize, isize) {
    let sequence: Vec<isize> = line
        .split(' ')
        .map(|s| s.parse::<isize>().unwrap())
        .collect();

    let reverse_sequence: Vec<isize> = sequence.clone().into_iter().rev().collect();

    (
        sequence[sequence.len() - 1] + next_diff(&sequence),
        reverse_sequence[reverse_sequence.len() - 1] + next_diff(&reverse_sequence),
    )
}

fn next_diff(values: &[isize]) -> isize {
    if values.iter().all(|&v| v == 0) {
        return 0;
    }

    let diffs: Vec<_> = values
        .windows(2)
        .map(|window| window[1] - window[0])
        .collect();

    diffs[diffs.len() - 1] + next_diff(&diffs)
}

pub async fn solve(mut rx: Receiver<String>) {
    let mut tasks = Vec::new();

    while let Some(line) = rx.recv().await {
        if !line.is_empty() {
            let task = tokio::spawn(async move { calc_line(&line).await });
            tasks.push(task);
        }
    }

    let mut next_values = Vec::new();
    let mut prev_values = Vec::new();

    for task in tasks {
        if let Ok((next_val, prev_val)) = task.await {
            next_values.push(next_val);
            prev_values.push(prev_val);
        }
    }

    let part1 = next_values.into_iter().sum::<isize>();
    let part2 = prev_values.into_iter().sum::<isize>();
    println!("Part 1: {part1}, Part 2: {part2}");
}
