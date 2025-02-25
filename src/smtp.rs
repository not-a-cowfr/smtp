use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream}};
use std::{env::var, error::Error};

pub async fn start_smtp() -> Result<(), Box<dyn Error>> {
    let bind_address = var("HOST").unwrap_or("0.0.0.0".to_string());
    let smtp_port = var("SMTP_PORT").unwrap_or("2525".to_string());
    
    let listener = TcpListener::bind(format!("{}:{}", bind_address, smtp_port)).await?;
    println!("Starting smtp server at {}:{}", bind_address, smtp_port);

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("New connection from: {}", addr);

        tokio::spawn(async move {
            if let Err(e) = handle_smtp(stream).await {
                eprintln!("Error handling tcp stream: {}", e);
            }
        });
    }
}

// Rest of your smtp.rs code remains the same

struct EmailContent {
    pub buffer: [u8; 1024],
    pub reset_state: bool,
    pub sender: String,
    pub reciever: String,
    pub data: Vec<u8>,
}

impl Default for EmailContent {
    fn default() -> Self {
        EmailContent {
            buffer: [0; 1024],
            reset_state: false,
            sender: String::new(),
            reciever: String::new(),
            data: Vec::new(),
        }
    }
}

async fn handle_smtp(
    mut stream: TcpStream,
) -> Result<(), Box<dyn Error>> {
    let mut content = EmailContent::default();

    if let Err(e) = stream.write_all(b"220 smtp server").await {
        eprintln!("Error sending message: {}", e);
        return Err(Box::new(e));
    }

    loop {
        let n = match stream.read(&mut content.buffer).await {
            Ok(n) if n > 0 => n,
            _ => break,
        };

        let request = String::from_utf8_lossy(&content.buffer[..n]).to_string();
        let requests_caps = request.to_uppercase();
        println!("Recieved command: {}", requests_caps.trim());

        match requests_caps.trim() {
            command if command.starts_with("HELO") || command.starts_with("EHLO") => {
                if let Err(e) = stream.write_all(b"250 Hello").await {
                    eprintln!("Error handling TcpStream: {}", e);
                    break;
                }
            }
            _ => {
                if let Err(e) = stream.write_all(b"250 Unkown Command").await {
                    eprintln!("Error handling TcpStream: {}", e);
                    break;
                }
            }
        }
    }
    Ok(())
}
