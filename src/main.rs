use tokio::net::TcpListener;
use tokio_ws::run;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:3000").await?;

    run(listener).await;

    Ok(())
}
