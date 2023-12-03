use tokio::sync::mpsc::Receiver;

#[derive(Debug)]
struct CandidatePart {
    part_id: String,
    line_no: usize,
    positions: Vec<usize>,
    confirmed_part: bool,
}

struct Symbol {
    chr: char,
    loc: (usize, usize),
}

impl CandidatePart {
    fn touching(&self, loc: &(usize, usize)) -> bool {
        (self.line_no.abs_diff(loc.0) <= 1) & self.positions.iter().any(|p| p.abs_diff(loc.1) <= 1)
    }
}

async fn calc_line(line: String, line_no: usize) -> (Vec<CandidatePart>, Vec<Symbol>) {
    let mut building_part = false;

    let mut candidate_parts = Vec::new();
    let mut symbols = Vec::new();

    for (col_no, chr) in line.chars().enumerate() {
        match (chr, building_part) {
            ('.', false) => (),
            ('.', true) => building_part = false,
            ('0'..='9', false) => {
                candidate_parts.push(CandidatePart {
                    part_id: format!("{chr}"),
                    line_no,
                    positions: vec![col_no],
                    confirmed_part: false,
                });
                building_part = true;
            }
            ('0'..='9', true) => {
                let last = candidate_parts.len() - 1;
                candidate_parts[last].part_id.push(chr);
                candidate_parts[last].positions.push(col_no);
            }
            (_, true) => {
                symbols.push(Symbol {
                    chr,
                    loc: (line_no, col_no),
                });
                building_part = false;
            }
            (_, false) => {
                symbols.push(Symbol {
                    chr,
                    loc: (line_no, col_no),
                });
            }
        }
    }

    (candidate_parts, symbols)
}

pub async fn solve(mut rx: Receiver<(usize, String)>) {
    let mut tasks = Vec::new();

    while let Some((line_no, line)) = rx.recv().await {
        let task = tokio::spawn(async move { calc_line(line, line_no).await });
        tasks.push(task);
    }

    let mut all_parts = Vec::new();
    let mut all_symbols = Vec::new();

    for task in tasks {
        if let Ok((mut parts, mut symbols)) = task.await {
            all_parts.append(&mut parts);
            all_symbols.append(&mut symbols);
        }
    }

    for symbol in all_symbols.iter() {
        for part in all_parts.iter_mut().filter(|p| p.touching(&symbol.loc)) {
            part.confirmed_part = true;
        }
    }

    let part1 = all_parts
        .iter()
        .filter(|p| p.confirmed_part)
        .map(|p| p.part_id.parse::<usize>().unwrap())
        .sum::<usize>();

    let mut part2 = 0usize;
    for symbol in all_symbols {
        if symbol.chr != '*' {
            continue;
        }
        let parts = all_parts
            .iter()
            .filter(|p| p.touching(&symbol.loc))
            .map(|p| p.part_id.clone())
            .collect::<Vec<String>>();

        if parts.len() == 2 {
            part2 += parts
                .into_iter()
                .map(|s| s.parse::<usize>().unwrap())
                .product::<usize>();
        }
    }

    println!("Part 1: {part1} Part 2: {part2}");
}
