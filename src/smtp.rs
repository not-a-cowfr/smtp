use std::str::FromStr;
use std::{env::var, error::Error};
use tokio::sync::OnceCell;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

static BIND_ADDRESS: OnceCell<String> = OnceCell::const_new();
static PORT: OnceCell<String> = OnceCell::const_new();

pub async fn start_smtp() -> Result<(), Box<dyn Error>> {
    let bind_address = var("BIND_ADDRESS").unwrap_or("0.0.0.0".to_string());
    BIND_ADDRESS.set(bind_address.clone()).unwrap();
    let smtp_port = var("PORT").unwrap_or("2525".to_string());
    PORT.set(smtp_port.clone()).unwrap();
    // let smtp_domain = var("SMTP_DOMAIN").unwrap_or("smtp.notacow.fr".to_string());

    let listener = TcpListener::bind(format!("{}:{}", bind_address, smtp_port)).await?;
    tracing::info!("Starting smtp server at {}:{}", bind_address, smtp_port);

    loop {
        let (stream, addr) = listener.accept().await?;
        tracing::info!("New connection from: {}", addr);

        let add = bind_address.clone();
        let port = smtp_port.clone();
        tokio::spawn(async move {
            handle_smtp(stream, format!("{}:{}", add, port))
                .await
                .log_error();
        });
    }
}

#[derive(Debug, PartialEq)]
enum SmtpState {
    GREET,
    MAIL,
    RCPT,
    DATA,
    POST_DATA,
    QUIT,
}

impl FromStr for SmtpState {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "HELO" | "EHLO" => Ok(SmtpState::GREET),
            "MAIL" => Ok(SmtpState::MAIL),
            "RCPT" => Ok(SmtpState::RCPT),
            "DATA" => Ok(SmtpState::DATA),
            "QUIT" => Ok(SmtpState::QUIT),
            _ => Err(()),
        }
    }
}

struct EmailContent {
    pub buffer: [u8; 1024],
    pub reset_state: bool,
    pub sender: String,
    pub recievers: Vec<String>,
    pub data: String,
}

impl Default for EmailContent {
    fn default() -> Self {
        EmailContent {
            buffer: [0; 1024],
            reset_state: false,
            sender: String::new(),
            recievers: Vec::new(),
            data: String::new(),
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

    let mut current_state: SmtpState = SmtpState::QUIT;
    let mut data = String::new();
    'conn: loop {
        let n = match stream.read(&mut content.buffer).await {
            Ok(n) if n > 0 => n,
            _ => break 'conn,
        };

        let request = String::from_utf8_lossy(&content.buffer[..n]).to_string();
        let mut parts = request.trim().splitn(2, ' ');
        let command_str = parts.next().unwrap_or("").to_uppercase();

        if current_state != SmtpState::DATA {
            tracing::debug!("Recieved command: {:?}", command_str);

            match command_str.parse::<SmtpState>() {
                Ok(cmd) => {
                    current_state = cmd;
                }
                Err(_) => {
                    stream
                        .write_all(b"500 Unknown command\r\n")
                        .await
                        .log_error();
                    continue 'conn;
                }
            };

            let args = parts.next().unwrap_or("");

            match current_state {
                SmtpState::MAIL => content.sender = args.replace("FROM:", ""),
                SmtpState::RCPT => content.recievers.push(args.replace("TO:", "")),
                _ => {}
            }

            stream.respond(&current_state).await;

            if current_state == SmtpState::QUIT {
                break 'conn;
            }
        } else {
            if request.ends_with(".\r\n") {
                current_state = SmtpState::POST_DATA;
                stream.respond(&current_state).await;
            }
            data.push_str(&request);
        }
    }

    data.truncate(data.len() - 3);
    content.data = data;

    Ok(())
}

trait Log<T> {
    fn log_error(&self);
}

impl<T, E: std::fmt::Debug> Log<T> for Result<T, E> {
    fn log_error(&self) {
        if let Err(e) = self {
            tracing::error!("Error handling TcpStream: {:?}", e);
        }
    }
}

trait Respond {
    async fn respond<'a>(&mut self, state: &SmtpState);
}

const OK_RESPONSE: &str = "250 Ok";

impl Respond for TcpStream {
    async fn respond<'a>(&mut self, state: &SmtpState) {
        let response = match state {
            SmtpState::GREET => &format!(
                "250 {}:{} Hello",
                BIND_ADDRESS.get().unwrap(),
                PORT.get().unwrap()
            ),
            SmtpState::DATA => "354 End data with <CR><LF>.<CR><LF>",
            SmtpState::QUIT => "221 Bye",
            _ => OK_RESPONSE,
        };

        self.write_all(format!("{}\r\n", response).as_bytes())
            .await
            .log_error();
    }
}
