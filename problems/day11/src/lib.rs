use tokio::sync::mpsc::Receiver;

async fn calc_line(line_no: usize, line: String) -> Vec<(usize, usize)> {
    line.chars()
        .enumerate()
        .filter(|(_col, c)| *c == '#')
        .map(|(col, _c)| (col, line_no))
        .collect()
}

pub async fn solve(mut rx: Receiver<(usize, String)>) {
    let mut total_part1 = 0;
    let mut total_part2 = 0;

    let mut tasks = Vec::new();

    while let Some((line_no, line)) = rx.recv().await {
        let task = tokio::spawn(async move { calc_line(line_no, line).await });
        tasks.push(task);
    }

    let mut galaxies = Vec::new();
    for task in tasks {
        if let Ok(mut gals) = task.await {
            galaxies.append(&mut gals);
        }
    }

    let x_vals: Vec<_> = galaxies.iter().map(|(x, _)| x).collect();
    let y_vals: Vec<_> = galaxies.iter().map(|(_, y)| y).collect();

    let max_cols = x_vals.iter().cloned().max().unwrap();
    let max_rows = y_vals.iter().cloned().max().unwrap();

    let mut empty_cols: Vec<_> = (0..=*max_cols).filter(|x| !x_vals.contains(&x)).collect();
    let mut empty_rows: Vec<_> = (0..=*max_rows).filter(|y| !y_vals.contains(&y)).collect();

    dbg!(&galaxies);
    let expansion = 1_000_000 - 1;
    let mut drift = 0usize;
    for col in empty_cols.iter() {
        for galaxy in galaxies.iter_mut() {
            if galaxy.0 > (col + drift) {
                galaxy.0 += expansion;
            }
        }
        drift += expansion;
    }
    dbg!(&galaxies);

    let mut drift = 0usize;
    for row in empty_rows.iter() {
        for galaxy in galaxies.iter_mut() {
            if galaxy.1 > (row + drift) {
                galaxy.1 += expansion;
            }
        }
        drift += expansion;
    }
    dbg!(&galaxies);

    let mut distances = 0usize;
    for gal1 in galaxies.iter() {
        for gal2 in galaxies.iter() {
            if gal1 == gal2 {
                continue;
            }
            distances += gal1.0.abs_diff(gal2.0) + gal1.1.abs_diff(gal2.1);
        }
    }

    distances = distances / 2;

    println!("Part 1: {distances}  Part 2: ");
}
