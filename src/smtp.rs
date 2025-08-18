use std::{env::var, error::Error};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

pub async fn start_smtp() -> Result<(), Box<dyn Error>> {
    let bind_address = var("BIND_ADDRESS").unwrap_or("0.0.0.0".to_string());
    let smtp_port = var("PORT").unwrap_or("2525".to_string());
    // let smtp_domain = var("SMTP_DOMAIN").unwrap_or("smtp.notacow.fr".to_string());

    let listener = TcpListener::bind(format!("{}:{}", bind_address, smtp_port)).await?;
    println!("Starting smtp server at {}:{}", bind_address, smtp_port);

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("New connection from: {}", addr);

        let add = bind_address.clone();
        let port = smtp_port.clone();
        tokio::spawn(async move {
            handle_smtp(stream, format!("{}:{}", add, port))
                .await
                .log_error();
        });
    }
}

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

async fn handle_smtp(mut stream: TcpStream, domain: String) -> Result<(), Box<dyn Error>> {
    let mut content = EmailContent::default();

    if let Err(e) = stream
        .write_all(format!("220 {} SMTP Ready\r\n", domain).as_bytes())
        .await
    {
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
                stream
                    .write_all(format!("250 {} Hello\r\n", domain).as_bytes())
                    .await
                    .log_error();
            }
            command if command.starts_with("MAIL") || command.starts_with("RCPT") => {
                stream.write_all(b"250 Ok\r\n").await.log_error();
            }
            command if command.starts_with("DATA") => {
                stream
                    .write_all(b"354 End data with <CR><LF>.<CR><LF>")
                    .await
                    .log_error();
            }
            "." => {
                stream
                    .write_all(b"250 Ok: queued as 12345")
                    .await
                    .log_error();
            }
            command if command.starts_with("QUIT") => {
                stream.write_all(b"221 Bye").await.log_error();
            }
            _ => {
                stream
                    .write_all(b"250 Unknown Command\r\n")
                    .await
                    .log_error();
            }
        }
    }
    Ok(())
}

trait Log<T> {
    fn log_error(&self);
}

impl<T, E: std::fmt::Debug> Log<T> for Result<T, E> {
    fn log_error(&self) {
        if let Err(e) = self {
            eprintln!("Error handling TcpStream: {:?}", e);
        }
    }
}
