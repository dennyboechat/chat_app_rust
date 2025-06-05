# Simple Chat Application in Rust

A lightweight command-line chat application built using **Rust**, **WebSockets**, and **SQLite**. This app supports private messaging, chat history, keyword search, and real-time communication — all from the terminal!

## Features

-  Users can send and receive messages in real time
-  Private messaging with `/msg username your message`
-  Persists chat history in a local SQLite database
-  View recent messages with `/history`
-  Search messages by keyword using `/search keyword`
-  Built with asynchronous concurrency via `tokio`
-  Uses enums and structs for message structure
-  Thread-safe memory access via `Arc<Mutex<>>`

## How to Run

### Prerequisites

- Ensure [Rust](https://www.rust-lang.org/tools/install) is installed on your system.

### Clone the Repository

```bash
git clone https://github.com/dennyboechat/chat_app_rust.git
cd chat_app_rust
```

### Build and Run

In one terminal (for the server):

```bash
cargo run -- server
```

In another terminal (for a client):

```bash
cargo run -- client
```

## Usage

Once inside the client terminal:

- Send a public message: just type and hit Enter
- Send a private message: `/msg username Hello there!`
- View message history: `/history`
- Search messages: `/search hello`

## Structure

- `ChatMessage` enum: Clean and unified message format
- SQLite-backed persistence: All messages are logged to `chat_history.db`
- Timestamps and user IDs are stored for traceability
- `Arc<Mutex<>>`: Ensures thread-safe shared state
- `tokio`: Powers async runtime for multiple clients

## Manageable Components

- Server and client separated via `clap` subcommands
- Modular message handling via `ChatMessage` enum
- Message logging is abstracted via `.log_to_db()` method
- Concurrent user sessions handled with `Arc<Mutex<HashMap>>`
- Async I/O separated into tasks via `tokio::spawn`
- Simple and extensible command parser (`/msg`, `/history`, `/search`)

## Challenges

- Managing WebSocket async streams (read/write split)
- Borrow checker issues (ownership of WebSocket writer)
- Thread-safe access to database and user state
- Integration of multiple crates (`tokio`, `rusqlite`, `tokio-tungstenite`)
- Manual multi-client testing in separate terminals

---
