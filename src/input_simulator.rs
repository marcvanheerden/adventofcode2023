use tokio::sync::mpsc::Sender;

pub async fn simulate_user_input(tx: Sender<String>, input_data: Vec<String>) {
    println!("Sending data");

    for line in input_data {
        tx.send(line).await.expect("Failed to send input");
    }

    println!("Finished sending data");
}
