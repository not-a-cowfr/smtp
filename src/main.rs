use std::env::var;
use actix_web::{App, HttpServer, get, HttpResponse};

mod smtp;

#[get("/")]
async fn health_check() -> HttpResponse {
    HttpResponse::Ok().body("Server is running")
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let bind_address = var("HOST").unwrap_or("0.0.0.0".to_string());
    let port = var("PORT").unwrap_or("10000".to_string()).parse::<u16>().unwrap();

    println!("Starting web server at {}:{}", bind_address, port);

    tokio::spawn(async {
        if let Err(e) = smtp::start_smtp().await {
            eprintln!("Error starting smtp server: {}", e)
        }
    });

    HttpServer::new(move || {
        App::new()
            .service(health_check)
    })
    .bind(format!("{}:{}", bind_address, port))?
    .run()
    .await
}