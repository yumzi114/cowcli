//use futures_util::{future, pin_mut,StreamExt};
// use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{future, pin_mut};
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::{Message, error::Error as WsError};
use futures::{Sink, SinkExt, Stream, StreamExt};
use stream_reconnect::{UnderlyingStream, ReconnectStream};
use std::task::{Context, Poll};
use futures::prelude::*; 
use tokio::task;
use std::io;
use std::pin::Pin;


use futures_channel::mpsc;
use tokio_util::codec::{Decoder, Encoder};
use tokio::net::{TcpListener, TcpStream};
use tokio_serial::{SerialPort, SerialPortBuilderExt, StopBits};
use tracing::{debug, info};
use tracing_subscriber::fmt::time;
use std::fmt::Debug;
use std::io::{stdin, Read};
use std::{env, thread};
use std::time::Duration;
use clap::{Command, Parser};
use mini_redis::{Connection, Frame};
use tracing_subscriber::{fmt,fmt::format, prelude::*};

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use cowapi::{LineCodec};



#[cfg(unix)]
const SOCKETURL: &'static str = env!("SOCKETURL");
const SERIAL_DEVICE: &'static str = env!("SERIAL_DEVICE");
const SIZE: &'static str = env!("SIZE");






//프로그램 실행시 소켓 URL 지정시 기본 - env구성
#[derive(Parser)]
#[command(name = "CowCLI")]
#[command(version = "0.1")]
#[command(about = "Data Sender", long_about = None)]
#[command(next_line_help = true)]
struct Cli {
    #[arg(default_value_t = SOCKETURL.to_string())]
    socket: String,
    #[arg(default_value_t = SERIAL_DEVICE.to_string())]
    serial: String,
    #[arg(default_value_t = SIZE.to_string())]
    size: String,
}

struct MyWs(WebSocketStream<MaybeTlsStream<TcpStream>>);
impl UnderlyingStream<String, Result<Message, WsError>, WsError> for MyWs {
    fn establish(addr: String) -> Pin<Box<dyn Future<Output = Result<Self, WsError>> + Send>> {
        Box::pin(async move {
            let (ws_stream, _) = connect_async(addr).await?;
            Ok(MyWs(ws_stream))
        })
    }

    fn is_write_disconnect_error(&self, err: &WsError) -> bool {
        matches!(
            err,
            WsError::ConnectionClosed
                | WsError::AlreadyClosed
                | WsError::Io(_)
                | WsError::Tls(_)
                | WsError::Protocol(_)
        )
    }

    fn is_read_disconnect_error(&self, item: &Result<Message, WsError>) -> bool {
        if let Err(e) = item {
            self.is_write_disconnect_error(e)
        } else {
            false
        }
    }

    fn exhaust_err() -> WsError {
        WsError::Io(io::Error::new(io::ErrorKind::Other, "Exhausted"))
    }
}
impl Stream for MyWs {
    type Item = Result<Message, WsError>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.get_mut().0.poll_next_unpin(cx)
    }
}
impl Sink<Message> for MyWs {
    type Error = WsError;

    fn poll_ready(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.get_mut().0.poll_ready_unpin(cx)
    }

    fn start_send(
        self: Pin<&mut Self>,
        item: Message,
    ) -> Result<(), Self::Error> {
        self.get_mut().0.start_send_unpin(item)
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        // WebSocketStream은 poll_flush 메소드를 구현하고 있어야 합니다.
        let mut this = self.get_mut();
        match task::block_in_place(|| futures::executor::block_on(this.0.flush())) {
            Ok(()) => Poll::Ready(Ok(())),
            Err(e) => Poll::Ready(Err(e)),
        }
    }

    fn poll_close(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        // WebSocketStream은 poll_close 메소드를 구현하고 있어야 합니다.
        let mut this = self.get_mut();
        match task::block_in_place(|| futures::executor::block_on(this.0.close(None))) {
            Ok(()) => Poll::Ready(Ok(())),
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}
type ReconnectWs = ReconnectStream<MyWs, String, Result<Message, WsError>, WsError>;
#[tokio::main]
async fn main()-> mini_redis::Result<()>{
    let cli = Cli::parse();
    // let ttt = console_subscriber::init();
    // let console_layer = console_subscriber::spawn();
    // let fmt = format().with_timer(time::Uptime::default());
    let fmt_layer = fmt::layer()
    .without_time()
    .with_file(false)
    .with_line_number(false)
    .with_target(false);
    let console_layer = console_subscriber::ConsoleLayer::builder()
    .with_default_env()

    // .enable_self_trace(true)
    .spawn();
    tracing_subscriber::registry()
    .with(console_layer)
    // .with(tracing_subscriber::fmt::layer())
    .with(fmt_layer)
    .init();
    
    
    let cli = Cli::parse();
    info!(
        "TRY CONNECT SOCKET URL {}", cli.socket
    );
    info!(
        "TRY CONNECT SERIAL URL {}", cli.serial
    );
    info!(
        "SET FarmSize is {}", cli.size
    );

    let (stdin_tx, stdin_rx) = mpsc::unbounded();
    let ping_tx = stdin_tx.clone();
    // // let ping_handle =tokio::task::Builder::new().name("Ping Sender Task");
    // // Ping Sender Channel
    let handle = tokio::task::Builder::new()
        .name("Ping Sender Task")
        .spawn(async move{
        loop{
            thread::sleep(Duration::from_secs(1));
            // ping_tx.unbounded_send(Message::Text("ping".to_string())).unwrap();
            ping_tx.unbounded_send(Message::Ping(vec![0])).unwrap();
        }// Do some async work
        // "return value"
    });
    let serial_tx = stdin_tx.clone();
    let serial_handle = tokio::task::Builder::new()
        .name("Serial Reciver")
        .spawn(async move{
            let mut port = tokio_serial::new(cli.serial.to_string(), 115_200).open_native_async().unwrap();
            #[cfg(unix)]
            port.set_stop_bits(StopBits::One).unwrap();
            
            port.set_exclusive(false)
                .expect("Unable to set serial port exclusive to false");
            let mut reader =LineCodec.framed(port);
           
            while let Some(line_result)=reader.next().await {
                // serial_tx.unbounded_send(Message::Ping(vec![8])).unwrap();
                // serial_tx.unbounded_send(Message::Text("test".to_string())).unwrap();
                println!("{:?}",line_result);
                if let Ok(data)=line_result{
                    serial_tx.unbounded_send(Message::Binary(data)).unwrap();
                    let ddd: Vec<_> =cli.size.split(":").collect();
                    let mut far_size = Vec::new();
                    for i in ddd{
                        let data = i.parse::<u16>().unwrap();
                        for e in data.to_be_bytes(){
                            far_size.push(
                                e
                            )
                        }
                    }
                    serial_tx.unbounded_send(Message::Binary(far_size)).unwrap();
                }
            };
    });
    // let mut ws_stream = ReconnectWs::connect(String::from(cli.socket.to_string())).await.unwrap();
    let mut ws_stream = ReconnectWs::connect(String::from("ws://192.168.0.10:8080/socket")).await.unwrap();
    // let (ws_stream, _) = connect_async(cli.socket.to_string()).await.expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");
    let (write, read) = ws_stream.split();
    let stdin_to_ws = stdin_rx.map(Ok).forward(write);
    let ws_to_stdout = {
        read.for_each(|message| async {
            // let data = message.unwrap().into_data();
            if let Ok(msg)=message{
                match msg {
                    Message::Ping(data)=>tokio::io::stdout().write_all(&data).await.unwrap(),
                    Message::Text(text)=>{
                        // tokio::io::stdout().write_all(text.as_bytes()).await.unwrap()
                    },
                    
                    _=>{
    
                    }
                }
            }
        })
    };
    pin_mut!(stdin_to_ws, ws_to_stdout);
    future::select(stdin_to_ws, ws_to_stdout).await;
    Ok(())
}

