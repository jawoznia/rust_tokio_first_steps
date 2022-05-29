use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::broadcast;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    let (tx, _) = broadcast::channel(10);

    loop {
        let (mut socket, addr) = listener.accept().await?;

        let tx = tx.clone();
        let mut rx = tx.subscribe();

        tokio::spawn(async move {
            let (reader, mut writer) = socket.split();

            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            // In a loop, read data from the socket and write the data back.
            loop {
                tokio::select! {
                    result = reader.read_line(&mut line) => {
                        if result.unwrap() == 0 {
                            break;
                        }
                            tx.send((line.clone(), addr)).unwrap();
                            line.clear();
                    }
                    result = rx.recv() => {
                        let (msg, sender_addr) = result.unwrap();
                        if addr != sender_addr {
                            if let Err(e) = writer.write_all(msg.as_bytes()).await {
                                eprintln!("failed to write to socket; err = {:?}", e);
                                return;
                            }
                        }
                }
                }
            }
        });
    }
}
