mod input_simulator;

use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let day = std::env::args().nth(1).expect("Please provide_day number");
    let test = std::env::args().nth(2).is_some_and(|s| s == "test");
    let large = std::env::args().nth(2).is_some_and(|s| s == "large");

    let filename = match (test, large) {
        (true, false) => format!("problems/day{}/input_test.txt", day),
        (false, true) => format!("problems/day{day}/day{day}_big_input.txt"),
        (false, false) => format!("problems/day{}/input.txt", day),
        (true, true) => panic!("incompatible options"),
    };
    dbg!(&filename);
    let input_data = match day.as_str() {
        "15" => std::fs::read_to_string(filename)
            .expect("Can't read input file")
            .split(',')
            .map(|s| s.to_string())
            .collect::<Vec<String>>(),
        "05" | "13" => std::fs::read_to_string(filename)
            .expect("Can't read input file")
            .split("\n\n")
            .map(|s| s.to_string())
            .collect::<Vec<String>>(),
        "06" => {
            let rows = std::fs::read_to_string(filename)
                .expect("Can't read input file")
                .lines()
                .map(|l| {
                    let (_feature, values) = l.split_once(':').unwrap();
                    values
                        .split(' ')
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>()
                })
                .collect::<Vec<Vec<String>>>();

            rows[0]
                .iter()
                .zip(rows[1].iter())
                .map(|(s1, s2)| format!("{s1} {s2}"))
                .collect::<Vec<String>>()
        }
        _ => std::fs::read_to_string(filename)
            .expect("Can't read input file")
            .lines()
            .map(|s| s.to_string())
            .collect::<Vec<String>>(),
    };

    let (tx, rx) = mpsc::channel(100_000);
    let (tx2, rx2) = mpsc::channel(100_000);

    match day.as_str() {
        "03" | "04" | "10" | "11" | "14" | "16" | "17" | "18" | "21" | "22" | "23" => {
            tokio::spawn(async move {
                input_simulator::simulate_user_input_enumerated(tx2, input_data).await;
            });
        }
        _ => {
            tokio::spawn(async move {
                input_simulator::simulate_user_input(tx, input_data).await;
            });
        }
    };

    match day.as_str() {
        "01" => day01::solve(rx).await,
        "02" => day02::solve(rx).await,
        "03" => day03::solve(rx2).await,
        "04" => day04::solve(rx2).await,
        "05" => day05::solve(rx).await,
        "06" => day06::solve(rx).await,
        "07" => day07::solve(rx).await,
        "08" => day08::solve(rx).await,
        "09" => day09::solve(rx).await,
        "10" => day10::solve(rx2).await,
        "11" => day11::solve(rx2).await,
        "12" => day12::solve(rx).await,
        "13" => day13::solve(rx).await,
        "14" => day14::solve(rx2).await,
        "15" => day15::solve(rx).await,
        "16" => day16::solve(rx2).await,
        "17" => day17::solve(rx2).await,
        "18" => day18::solve(rx2).await,
        "19" => day19::solve(rx).await,
        "20" => day20::solve(rx).await,
        "21" => day21::solve(rx2).await,
        "22" => day22::solve(rx2).await,
        "23" => day23::solve(rx2).await,
        "24" => day24::solve(rx).await,
        "25" => day25::solve(rx).await,
        _ => eprintln!("Solution for day {} not implemented", day),
    };
}
