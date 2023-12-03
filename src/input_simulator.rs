use tokio::sync::mpsc::Sender;

pub async fn simulate_user_input(tx: Sender<String>, input_data: Vec<String>) {
    println!("Sending data");

    for line in input_data {
        tx.send(line).await.expect("Failed to send input");
    }

    println!("Finished sending data");
}

pub async fn simulate_user_input_enumerated(tx: Sender<(usize, String)>, input_data: Vec<String>) {
    println!("Sending data");

    for (idx, line) in input_data.into_iter().enumerate() {
        tx.send((idx, line)).await.expect("Failed to send input");
    }

    println!("Finished sending data");
}
