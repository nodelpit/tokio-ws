use ::tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;

pub async fn run(listener: TcpListener) {
    loop {
        let (stream, _addr) = match listener.accept().await {
            Ok(stream) => stream,
            Err(err) => {
                println!("failed to accept connection: {err}");
                continue;
            }
        };

        let ws_stream = match accept_async(stream).await {
            Ok(ws) => {
                println!("Connected to WebSocket");
                ws
            }

            Err(err) => {
                println!("failed to join websocket: {err}");
                continue;
            }
        };

        let (mut write, mut read) = ws_stream.split();

        loop {
            match read.next().await {
                Some(Ok(message)) => match message {
                    Message::Text(ref text) => {
                        println!("message received: {}", text);
                        let _echo_send = write.send(message).await;
                    }

                    Message::Close(_) => {
                        println!("connection closed");
                        let _ = write.close().await;
                        break;
                    }

                    _ => {
                        println!("message not conform");
                    }
                },

                Some(Err(err)) => {
                    println!("failed to read message: {err}");
                    break;
                }

                None => {
                    println!("connection closed");
                    break;
                }
            }
        }
    }
}
