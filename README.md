# A simple chat application using Rust language

A lightweight command-line chat application built using **Rust**, **WebSockets**, and **SQLite**. This app supports private messaging, message history logging, keyword search, and concurrent user handling — all from the terminal!

---

## Features

- Users can **send and receive messages** in real time
- Supports **private messaging** with `/msg username your message`
- Persists **chat history** to a local SQLite database
- View **message history** with `/history`
- **Search messages** by keyword with `/search keyword`
- Built with **asynchronous concurrency** using `tokio`
- Uses **structs and enums** for clean message representation
- Ensures **memory safety** with `Arc<Mutex<>>` patterns

---

## How to Run

### Prerequisites
Ensure you have [Rust installed](https://www.rust-lang.org/tools/install) on your system.

### Clone the Repository

```bash
git clone https://github.com/dennyboechat/chat_app_rust
cd chat_app_rust
cargo build
cargo run -- server (Run the server)
cargo run -- client (Run the client -- in separate terminal)


Usage
Once the client is running, interact through the command line:

Send a public message: Just type your message and press Enter

Send a private message:
/msg username Hello!

View recent message history:
/history

Search messages by keyword:
/search hello


## Structure  
-	Database to store chat history
-  -	timestamp and user ID
-	Memory safety model
-	Enums
-	Structs for message types
-   Async concurrency for message handling

## Manageable components
These components were clearly separated, modular, and easy to manage:

Server and Client separation
Handled via clap subcommands (server and client), allowing clear execution paths.

ChatMessage enum
Unified message logic under ChatMessage enum, handling both public and private messages with consistent structure.

SQLite logging
Logging messages to the chat_history.db via simple function call log_to_db(), with timestamp and message type.

User sender map
Concurrent HashMap<String, Sender> wrapped in Arc<Mutex<>> allowed safe tracking of active users.

Modular async tasks
Used tokio::spawn to separate sending and receiving tasks cleanly.

Command Parsing
Parsing commands like /msg, /history, and /search made the chat input experience flexible and intuitive.

Learning curve of crates
Integrating crates like tokio-tungstenite, rusqlite, and clap together in one project required reading documentation and trial-error.

## Challenges
These parts of the project involved notable complexity or learning:

Handling async message flow
Managing concurrent reads/writes over WebSockets required careful use of split() and tokio::spawn.

Sharing mutable state safely
Ensuring thread-safe access to the user_senders map required learning Arc<Mutex<>> patterns in Rust.

Borrow checker conflicts
Ran into ownership and move issues especially with ws_sender due to its non-Copy type. Refactoring was needed to avoid double borrows.

SQLite database access
Ensuring synchronous database calls inside async context needed wrapping in Arc<Mutex<>> and careful management of DB locks.

Testing multi-client interaction
Manually running multiple terminals for server/client instances made it harder to automate tests for features like private messages or search.

Learning curve of crates
Integrating crates like tokio-tungstenite, rusqlite, and clap together in one project required reading documentation and trial-error.

