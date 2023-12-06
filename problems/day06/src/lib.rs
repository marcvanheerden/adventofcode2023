use tokio::sync::mpsc::Receiver;

async fn calc_line(line: &str) -> usize {
    dbg!(line);

    let (time, distance) = line.split_once(' ').unwrap();
    let time = time.parse::<usize>().unwrap();
    let distance = distance.parse::<usize>().unwrap();

    (1..time)
        .filter(|wait| (time - wait) * wait > distance)
        .count()
}

pub async fn solve(mut rx: Receiver<String>) {
    let mut tasks = Vec::new();
    while let Some(line) = rx.recv().await {
        let task = tokio::spawn(async move { calc_line(&line).await });
        tasks.push(task);
    }

    let mut ways_to_win = Vec::new();
    for task in tasks {
        if let Ok(ways) = task.await {
            ways_to_win.push(ways);
        }
    }

    let part1 = ways_to_win.into_iter().product::<usize>();
    println!("Part 1: {part1} Part 2: ");
}
