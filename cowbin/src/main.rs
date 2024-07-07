use futures_channel::mpsc;
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, info};
use std::io::{stdin, Read};
use std::{env, thread};
use std::time::Duration;
use clap::{Command, Parser};
use mini_redis::{Connection, Frame};
use tracing_subscriber::prelude::*;
use futures_util::{future, pin_mut, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};


#[cfg(unix)]
const SOCKETURL: &'static str = env!("SOCKETURL");
const SERIAL_DEVICE: &'static str = env!("SERIAL_DEVICE");
#[derive(Parser)]
#[command(name = "CowCLI")]
#[command(version = "0.1")]
#[command(about = "Data Sender", long_about = None)]
#[command(next_line_help = true)]

//프로그램 실행시 소켓 URL 지정시 기본 - env구성
struct Cli {
    #[arg(default_value_t = SOCKETURL.to_string())]
    socket: String,
    #[arg(default_value_t = SERIAL_DEVICE.to_string())]
    serial: String,
}
#[tokio::main]
async fn main()-> mini_redis::Result<()>{
    let cli = Cli::parse();
    let console_layer = console_subscriber::ConsoleLayer::builder()
    .with_default_env()
    // .enable_self_trace(true)
    .spawn();
    tracing_subscriber::registry()
    .with(console_layer)
    .with(tracing_subscriber::fmt::layer())
//  .with(...)
    .init();
    
    
    let cli = Cli::parse();
    info!(
        "TRY CONNECT SOCKET URL {}", cli.socket
    );
    info!(
        "TRY CONNECT SERIAL URL {}", cli.serial
    );
    let (stdin_tx, stdin_rx) = mpsc::unbounded();
    let ping_tx = stdin_tx.clone();
    let ping_handle =tokio::task::Builder::new().name("Ping Sender Task");
    //Ping Sender Channel
    let handle = tokio::task::Builder::new()
        .name("Ping Sender Task")
        .spawn(async move{
        loop{
            thread::sleep(Duration::from_secs(1));
            ping_tx.unbounded_send(Message::Ping(vec![0])).unwrap();
        }// Do some async work
        // "return value"
    });
    // tokio::task::Builder::new().name("Socket Sender Task").spawn(read_stdin(stdin_tx));
    // tokio::spawn(read_stdin(stdin_tx));
    // let (ws_stream, _) = connect_async(cli.socket.as_str()).await.expect("Failed to connect");
    while let Ok((ws_stream, _))=connect_async(cli.socket.as_str()).await{
        info!(
            "TRY CONNECT SOCKET URL {}", cli.socket
        );
        // Do some other work
        let (write, read) = ws_stream.split();
        // let stdin_to_ws = stdin_rx.map(Ok).forward(write);
        // let ws_to_stdout = {
        //     read.for_each(|message| async {
        //         let data = message.unwrap().into_data();
        //         tokio::io::stdout().write_all(&data).await.unwrap();
        //     })
        // };
        // pin_mut!(stdin_to_ws, ws_to_stdout);
        // future::select(stdin_to_ws, ws_to_stdout).await;
        // let out = handle.await.unwrap();
        // 웹뷰시
        // let _guard = sentry::init(("https://83d6a0d9a7d596862cafc3e4b88ad360@o4507530225975296.ingest.de.sentry.io/4507530232725584", sentry::ClientOptions {
        //     release: sentry::release_name!(),
        //     ..Default::default()
        //   }));
        
        info!(
            "TRY CONNECT SERIAL DEVICE {}", cli.serial
        );
    }
    
    Ok(())
    
    // println!("two: {:?}", cli.two);
    // println!("one: {:?}", cli.one);
}

// async fn process(socket: TcpStream) {
//     // The `Connection` lets us read/write redis **frames** instead of
//     // byte streams. The `Connection` type is defined by mini-redis.
//     let mut connection = Connection::new(socket);

//     if let Some(frame) = connection.read_frame().await.unwrap() {
//         info!("GOT: {:?}", frame);
//         // Respond with an error
//         let response = Frame::Error("unimplemented".to_string());
//         connection.write_frame(&response).await.unwrap();
//     }
// }


// async fn read_stdin(tx: futures_channel::mpsc::UnboundedSender<Message>) {
//     let mut stdin = tokio::io::stdin();
//     loop {
//         let mut buf = vec![0; 1024];
//         let n = match stdin.read(&mut buf).await {
//             Err(_) | Ok(0) => break,
//             Ok(n) => n,
//         };
//         buf.truncate(n);
//         let ttt = String::from_utf8_lossy(&buf).to_string();
//         info!("{}",ttt);
//         // tx.unbounded_send(Message::binary(buf)).unwrap();
//         // tx.unbounded_send(Message::binary(buf)).unwrap();
//         tx.unbounded_send(Message::Text(ttt)).unwrap();
//     }
// }
