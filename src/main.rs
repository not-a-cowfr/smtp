mod smtp;

#[tokio::main]
async fn main() {
    tokio::spawn(async {
        if let Err(e) = smtp::start_smtp().await {
            eprintln!("Error starting smtp server: {}", e)
        }
    })
    .await
    .unwrap();
}
