mod input_simulator;

use tokio::sync::mpsc;

#[tokio::main]
async fn main() {


    let day = std::env::args().nth(1).expect("Please provide_day number");
    let test = std::env::args().nth(2).is_some_and(|s| s == "test");
    let large = std::env::args().nth(2).is_some_and(|s| s == "large");

    let filename = match (test, large) {
        (true, false) => format!("problems/day{}/input_test.txt", day),
        (false, true) => format!("problems/day{}/input_large.txt", day),
        (false, false) => format!("problems/day{}/input.txt", day),
        (true, true) => panic!("incompatible options"),
    };

    let input_data = std::fs::read_to_string(filename)
        .expect("Can't read input file")
        .lines()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    let (tx, rx) = mpsc::channel(100_000);
 
    tokio::spawn(async move {
        input_simulator::simulate_user_input(tx, input_data).await;
    });

    match day.as_str() {
        "01" => day01::solve(rx).await,
        _ => eprintln!("Solution for day {} not implemented", day),
    };
}



