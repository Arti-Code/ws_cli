extern crate colored;


use std::{env, io::Read, time::Duration};
use futures_util::{future, pin_mut, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use colored::Colorize;


#[tokio::main]
async fn main() {
    let ver = env!("CARGO_PKG_VERSION");
    let author = env!("CARGO_PKG_AUTHORS");
    println!(" ");
    let title = "-=WebSocket Client App=-".to_string().cyan();
    println!("{}", title);
    println!("{}: {} |  {}", "ver", ver, author);
    println!(" ");
    tokio::time::sleep(Duration::from_secs(1)).await;
    let url = match env::args().nth(1) {
        Some(url) => url,
        None => "ws://yamanote.proxy.rlwy.net:26134".to_string()
    };
    let msg = format!("connected to: {}", &url);
    println!("{}", msg);
    tokio::time::sleep(Duration::from_secs(1)).await;
    let (stdin_tx, stdin_rx) = futures_channel::mpsc::unbounded();
    tokio::spawn(read_stdin(stdin_tx));

    let (ws_stream, _) = connect_async(&url).await.expect("failed to connect");
    println!("{}", "connected!".to_string());
    println!("---------------------");

    let (write, read) = ws_stream.split();

    let stdin_to_ws = stdin_rx.map(Ok).forward(write);
    let ws_to_stdout = {
        read.for_each(|message| async {
            //let data = message.unwrap()..into_data();
            let text = message.unwrap().into_text().expect("unreadable data");
            let mut text = text.as_str().to_string();
            //let mut symbol = "[↘︎]".as_bytes().to_vec();
            text.insert_str(0, "[↘︎]");
            //let mut d = data.to_vec();
            //symbol.append(&mut d);
            //tokio::io::stdout().write_all(text).await.unwrap();
            println!("{}", text.green());
            //tokio::io::stdout().write_all(text.red().as_bytes()).await.unwrap();
        })
    };

    pin_mut!(stdin_to_ws, ws_to_stdout);
    future::select(stdin_to_ws, ws_to_stdout).await;
}

async fn read_stdin(tx: futures_channel::mpsc::UnboundedSender<Message>) {
    let mut stdin = tokio::io::stdin();
    loop {
        let mut buf = vec![0; 1024];
        let n = match stdin.read(&mut buf).await {
            Err(_) | Ok(0) => break,
            Ok(n) => n,
        };
        //let symbol = "[↗] ".as_bytes().to_vec();
        //let msg = [symbol, buf].concat();
        buf.truncate(n);
        //let mut bb = buf.
        tx.unbounded_send(Message::text(buf)).unwrap();
        
    }
}

// ↗