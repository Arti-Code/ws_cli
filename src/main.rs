use std::{env, time::Duration};
use futures_util::{future, pin_mut, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[tokio::main]
async fn main() {
    let ver = env!("CARGO_PKG_VERSION");
    let author = env!("CARGO_PKG_AUTHORS");
    println!(" ");
    println!("-=WebSocket Client App=-");
    println!("ver: {} | {}", ver, author);
    println!(" ");
    tokio::time::sleep(Duration::from_secs(1)).await;
    let url = match env::args().nth(1) {
        Some(url) => url,
        None => "ws://yamanote.proxy.rlwy.net:26134".to_string()
    };
    println!("connecting to: {}", &url);
    tokio::time::sleep(Duration::from_secs(1)).await;
    let (stdin_tx, stdin_rx) = futures_channel::mpsc::unbounded();
    tokio::spawn(read_stdin(stdin_tx));

    let (ws_stream, _) = connect_async(&url).await.expect("failed to connect");
    println!("connected!");
    println!(" ");

    let (write, read) = ws_stream.split();

    let stdin_to_ws = stdin_rx.map(Ok).forward(write);
    let ws_to_stdout = {
        read.for_each(|message| async {
            let data = message.unwrap().into_data();
            let mut symbol = "[↘︎]".as_bytes().to_vec();
            let mut d = data.to_vec();
            symbol.append(&mut d);
            tokio::io::stdout().write_all(&symbol).await.unwrap();
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
        tx.unbounded_send(Message::binary(buf)).unwrap();
    }
}

// ↗