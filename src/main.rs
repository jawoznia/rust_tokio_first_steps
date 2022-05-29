use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::broadcast;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    let (tx, _) = broadcast::channel::<String>(10);

    loop {
        let (mut socket, _) = listener.accept().await?;

        let tx = tx.clone();
        let mut rx = tx.subscribe();
        tokio::spawn(async move {
            let mut buf = [0; 1024];

            // In a loop, read data from the socket and write the data back.
            loop {
                let _n = match socket.read(&mut buf).await {
                    // socket closed
                    Ok(n) if n == 0 => return,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };

                if let Ok(line) = std::str::from_utf8(&buf) {
                    tx.send(line.to_string()).unwrap();
                }

                let msg = rx.recv().await.unwrap();

                // Write the data back
                if let Err(e) = socket.write_all(msg.as_bytes()).await {
                    eprintln!("failed to write to socket; err = {:?}", e);
                    return;
                }
            }
        });
    }
}
