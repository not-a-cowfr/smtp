#![allow(non_camel_case_types)]

use dotenv::dotenv;
use tracing::Level;

mod smtp;

#[tokio::main]
async fn main() {
    dotenv().ok();

    #[cfg(debug_assertions)]
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    #[cfg(not(debug_assertions))]
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    tokio::spawn(async {
        if let Err(e) = smtp::start_smtp().await {
            eprintln!("Error starting smtp server: {}", e)
        }
    })
    .await
    .unwrap();
}
