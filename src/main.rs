extern crate colored;
use std::io::Write;
use std::{
    env, time::Duration
};
use chat::cmd::Command;
use futures_util::{
    future,
    pin_mut, 
    StreamExt
};
//use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::io::*;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::{Utf8Bytes, protocol::Message}};
use colored::Colorize;

type ChatSender = futures_channel::mpsc::UnboundedSender<Message>;


#[tokio::main]
async fn main() {
    init_display().await;
    let user_name= register_name().await;
    let user_name: &'static str  = Box::leak(user_name.into_boxed_str());
    let url = get_connection_address().await;
    let (stdin_tx, stdin_rx) = futures_channel::mpsc::unbounded();
    //let tx = Arc::new(tokio::sync::Mutex::new(stdin_tx));
    //let tx1 = tx.clone();
    tokio::spawn(send_message(stdin_tx.clone(), &user_name));
    print!("connecting to {}...", &url);
    let ws_stream = establish_connection(&url).await;
    println!("[ok]");
    send_command(stdin_tx.clone(), &Command::RegisterUserName(user_name.to_string())).await;
    let (write, read) = ws_stream.split();
    let stdin_to_ws = stdin_rx.map(Ok).forward(write);
    let ws_to_stdout = {
        read.for_each(|message| async {
            match message {
                Err(e) => {
                    println!("Error receiving message: {}", e);
                    return;
                }
                Ok(msg) => recv_messages(msg).await,
            }
            
        })
    };

    pin_mut!(stdin_to_ws, ws_to_stdout);
    future::select(stdin_to_ws, ws_to_stdout).await;
}

async fn establish_connection(url: &str) -> WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>> {
    //print!("connecting to: {}...", url);
    //let msg = format!("connecting to: {}...", url);
    //std::io::stdout().write_all(msg.as_bytes()).expect("ups!");
    _ = std::io::stdout().flush().expect("ups!");
    tokio::time::sleep(Duration::from_secs(1)).await;
    let (ws_stream, _) = connect_async(url).await.expect("failed to connect");
    //println!("[OK]");
    //println!("---------------------");
    ws_stream
}

async fn recv_messages(msg: Message) {
    let text = msg.into_text()
        .expect("unreadable data");
    let mut text = text.to_string();
    text = text.trim().to_string();
    //_ = std::io::stdout().write_all(text.as_bytes());
    println!("{}", text.bold());
    //text.insert_str(0, "[↘︎]");
    //tokio::io::stdout().write_all(text.into_bytes().as_slice())
    //    .await.expect("can't write a message");
}

async fn send_message(tx: ChatSender, user_name: &str) {
    //let mut stdin0 = tokio::io::stdin();
    let user = format!("[{}] ", user_name).as_bytes().to_vec();
    loop {
        //print!("[Me]: ");
        //_ = std::io::stdout().flush();
        let mut stdin = tokio::io::stdin();
        let mut buf = vec![0; 1024];
        //stdin.chain(next)
        let n = match stdin.read(&mut buf).await {
            Err(_) | Ok(0) => break,
            Ok(n) => n,
        };
        
        buf.truncate(n);
        match check_command(std::str::from_utf8(&buf).unwrap()).await {
            Some(command) => {
                send_command(tx.clone(), &command).await;
                if let Command::Quit = command { break; }
                continue;
            }
            None => {
                let mut msg = user.clone(); 
                msg.extend_from_slice(&buf);
                //let my_msg = String::from_utf8_lossy(&msg).to_string();
                //println!("{}", my_msg.green());
                unsafe {
                    let text = Utf8Bytes::from_bytes_unchecked(msg.into());
                    if tx.unbounded_send(Message::Text(text)).is_err() {
                        break;
                    }
                }
            }
        }        
    }
}

async fn check_command(input: &str) -> Option<Command> {
    let trimmed = input.trim();
    match trimmed {
        "/quit" => Some(Command::Quit),
        "/list" => Some(Command::ListUsers),
        //cmd if cmd.starts_with("/register ") => {
        //    let name = cmd.trim_start_matches("/register ").to_string();
        //    Some(Command::RegisterUserName(name))
        //}
        _ => None,
    }
}

async fn send_command(mut tx: ChatSender, command: &Command) -> bool {
    let mut closing= false;
    let cmd_msg = match command {
        Command::Quit => {
            tx.disconnect();
            //tx.close_channel();
            let msg = "connection closed".to_string().red();
            println!("{}", msg);
            closing = true;
            "/quit".to_string()
        }
        Command::ListUsers => {
            "/list".to_string()
        }
        Command::RegisterUserName(name) => {
            format!("/register {}", name)
        }
    };
    //let text = Utf8Bytes::from_bytes_unchecked(cmd.into());
    let _ = tx.unbounded_send(Message::Text(cmd_msg.into()));
    return closing;
}

async fn register_name() -> String {
    print!("Enter your user name: ");
    std::io::stdout().flush().expect("ups! something went wrong!");
    let mut buf = String::new();
    _ = std::io::stdin().read_line(&mut buf).expect("failed to register user name");
    let name = buf.trim();
    println!("Welcome, {}!", name.green());
    name.to_string()
}

async fn init_display() {
    let ver = env!("CARGO_PKG_VERSION");
    let author = env!("CARGO_PKG_AUTHORS");
    println!(" ");
    let title = "-=WebSocket Chat Client=-".to_string().cyan();
    println!("{}", title);
    println!("{}: {} |  {}", "ver", ver, author);
    println!(" ");
    tokio::time::sleep(Duration::from_secs(1)).await;
}

async fn get_connection_address() -> String {
    match env::args().nth(1) {
        Some(url) => url,
        None => "ws://127.0.0.1:8080".to_string(),
        //None => "ws://yamanote.proxy.rlwy.net:26134".to_string(),
    }
}