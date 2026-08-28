use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use tokio_ws::run;

async fn spawn_server() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        run(listener).await;
    });

    format!("ws://{}", addr)
}

#[tokio::test]
async fn echo_text() {
    let url = spawn_server().await;

    let (mut ws, _) = connect_async(url).await.unwrap();

    ws.send(Message::Text("hello".into())).await.unwrap();

    let message = ws.next().await.unwrap().unwrap();

    assert_eq!(message, Message::Text("hello".into()));
}

#[tokio::test]
async fn preserves_message_order() {
    let url = spawn_server().await;

    let (mut ws, _) = connect_async(url).await.unwrap();

    ws.send(Message::Text("one".into())).await.unwrap();
    ws.send(Message::Text("two".into())).await.unwrap();
    ws.send(Message::Text("three".into())).await.unwrap();

    assert_eq!(
        ws.next().await.unwrap().unwrap(),
        Message::Text("one".into())
    );
    assert_eq!(
        ws.next().await.unwrap().unwrap(),
        Message::Text("two".into())
    );
    assert_eq!(
        ws.next().await.unwrap().unwrap(),
        Message::Text("three".into())
    );
}

#[tokio::test]
async fn close_cleanly() {
    let url = spawn_server().await;

    let (mut ws, _) = connect_async(url).await.unwrap();

    ws.send(Message::Close(None)).await.unwrap();

    let result = ws.next().await;

    assert!(result.is_none() || matches!(result, Some(Ok(Message::Close(_)))));
}
