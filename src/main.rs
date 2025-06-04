// src/main.rs
use std::{collections::HashMap, sync::{Arc, Mutex}};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::accept_async;
use clap::{Parser, Subcommand};
use colored::*;
use chrono::Local;
use rusqlite::{params, Connection};

#[derive(Parser)]
#[command(name = "ChatApp")]
#[command(about = "Simple CLI Chat Application", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Server,
    Client,
}

#[derive(Debug, Clone)]
enum ChatMessage {
    Public { from: String, content: String, timestamp: String },
    Private { from: String, to: String, content: String, timestamp: String },
    System(String),
}

impl ChatMessage {
    fn to_string(&self) -> String {
        match self {
            ChatMessage::Public { from, content, timestamp } => {
                format!("[{}][{}]: {}", timestamp, from, content)
            }
            ChatMessage::Private { from, to, content, timestamp } => {
                format!("[{}][Private from {} to {}]: {}", timestamp, from, to, content)
            }
            ChatMessage::System(msg) => msg.clone(),
        }
    }

    fn log_to_db(&self, conn: &Connection) {
        match self {
            ChatMessage::Public { from, content, timestamp } => {
                let _ = conn.execute(
                    "INSERT INTO messages (from_user, to_user, content, timestamp, is_private) VALUES (?1, NULL, ?2, ?3, 0)",
                    params![from, content, timestamp],
                );
            }
            ChatMessage::Private { from, to, content, timestamp } => {
                let _ = conn.execute(
                    "INSERT INTO messages (from_user, to_user, content, timestamp, is_private) VALUES (?1, ?2, ?3, ?4, 1)",
                    params![from, to, content, timestamp],
                );
            }
            _ => {}
        }
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Server => run_server().await,
        Commands::Client => run_client().await,
    }
}

async fn run_server() -> std::io::Result<()> {
    use tokio::sync::mpsc;

    let conn = Connection::open("chat_history.db").expect("Failed to open DB");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            from_user TEXT NOT NULL,
            to_user TEXT,
            content TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            is_private INTEGER NOT NULL
        )",
        [],
    ).expect("Failed to create table");

    let conn = Arc::new(Mutex::new(conn));
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    let user_senders: Arc<Mutex<HashMap<String, tokio::sync::mpsc::UnboundedSender<Message>>>> = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (stream, _) = listener.accept().await?;
        let ws_stream = accept_async(stream).await.unwrap();
        let user_senders_clone = Arc::clone(&user_senders);
        let db_conn = Arc::clone(&conn);

        tokio::spawn(async move {
            let (mut ws_sender, mut ws_receiver) = ws_stream.split();

            let username = match ws_receiver.next().await {
                Some(Ok(Message::Text(name))) => name,
                _ => {
                    eprintln!("Failed to receive username.");
                    return;
                }
            };

            let (tx_user, mut rx_user) = mpsc::unbounded_channel();
            user_senders_clone.lock().unwrap().insert(username.clone(), tx_user);

            let private_sender_task = tokio::spawn(async move {
                while let Some(msg) = rx_user.recv().await {
                    if let Err(e) = ws_sender.send(msg).await {
                        eprintln!("Failed to send private msg: {}", e);
                        break;
                    }
                }
            });

            while let Some(Ok(msg)) = ws_receiver.next().await {
                if let Message::Text(text) = msg {
                    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                    if text.starts_with("/msg ") {
                        let parts: Vec<&str> = text[5..].splitn(2, ' ').collect();
                        if parts.len() == 2 {
                            let target = parts[0].trim();
                            let message = parts[1].trim();
                            let chat = ChatMessage::Private {
                                from: username.clone(),
                                to: target.to_string(),
                                content: message.to_string(),
                                timestamp: now,
                            };
                            chat.log_to_db(&db_conn.lock().unwrap());
                            if let Some(tx) = user_senders_clone.lock().unwrap().get(target) {
                                let _ = tx.send(Message::Text(chat.to_string()));
                            } else {
                                let err = ChatMessage::System(format!("[Error] User '{}' not found.", target));
                                let _ = user_senders_clone.lock().unwrap().get(&username).map(|tx| tx.send(Message::Text(err.to_string())));
                            }
                        }
                    } else {
                        let chat = ChatMessage::Public {
                            from: username.clone(),
                            content: text.clone(),
                            timestamp: now,
                        };
                        chat.log_to_db(&db_conn.lock().unwrap());
                        println!("{}", chat.to_string());
                    }
                }
            }

            let _ = private_sender_task.await;
            user_senders_clone.lock().unwrap().remove(&username);
        });
    }
}
async fn run_client() -> std::io::Result<()> {
    use tokio_tungstenite::connect_async;
    use tokio::io::{AsyncBufReadExt, BufReader};
    use std::io::Write;
    use rusqlite::Connection;

    print!("Enter your username: ");
    std::io::stdout().flush().unwrap();
    let mut username = String::new();
    std::io::stdin().read_line(&mut username).unwrap();
    let username = username.trim().to_string();

    let (ws_stream, _) = connect_async("ws://127.0.0.1:8080").await.expect("Failed to connect");
    let (mut write, mut read) = ws_stream.split();

    write.send(Message::Text(username.clone())).await.unwrap();

    let stdin = BufReader::new(tokio::io::stdin());
    let mut lines = stdin.lines();

    tokio::spawn(async move {
        while let Some(Ok(msg)) = read.next().await {
            if let Message::Text(txt) = msg {
                if txt.contains(&username) {
                    println!("{}", txt.green());
                } else {
                    println!("{}", txt.cyan());
                }
            }
        }
    });

    while let Ok(Some(line)) = lines.next_line().await {
        if line.starts_with("/history") {
            let conn = Connection::open("chat_history.db").expect("Failed to open DB");
            println!("--- Message History ---");
            let mut stmt = conn.prepare("SELECT timestamp, from_user, to_user, content, is_private FROM messages ORDER BY id DESC LIMIT 10").unwrap();
            let rows = stmt.query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i32>(4)?
                ))
            }).unwrap();
            for row in rows {
                if let Ok((ts, from, to, content, private)) = row {
                    if private == 1 {
                        println!("[{}][Private from {} to {}]: {}", ts, from, to.unwrap_or("?".to_string()), content);
                    } else {
                        println!("[{}][{}]: {}", ts, from, content);
                    }
                }
            }
        } else if line.starts_with("/search ") {
            let keyword = line[8..].trim();
            let conn = Connection::open("chat_history.db").expect("Failed to open DB");
            println!("--- Search Results for '{}': ---", keyword);
            let query = format!("%{}%", keyword);
            let mut stmt = conn.prepare("SELECT timestamp, from_user, to_user, content, is_private FROM messages WHERE content LIKE ? ORDER BY id DESC LIMIT 10").unwrap();
            let rows = stmt.query_map([query], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i32>(4)?
                ))
            }).unwrap();
            for row in rows {
                if let Ok((ts, from, to, content, private)) = row {
                    if private == 1 {
                        println!("[{}][Private from {} to {}]: {}", ts, from, to.unwrap_or("?".to_string()), content);
                    } else {
                        println!("[{}][{}]: {}", ts, from, content);
                    }
                }
            }
        } else {
            write.send(Message::Text(line)).await.unwrap();
        }
    }

    Ok(())
}

