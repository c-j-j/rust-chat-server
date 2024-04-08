use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::select;
use tokio::sync::mpsc::{Receiver, Sender};

struct Message {
    payload: String,
}

#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:5050").await.unwrap();

    let (tx, mut rx): (Sender<Message>, Receiver<Message>) = tokio::sync::mpsc::channel(1024);
    println!("Waiting for connections...");
    let mut users: Vec<OwnedWriteHalf> = vec![];
    loop {
        select! {
            socket = listener.accept() => {
                if let Ok(socket) = socket {
                    let (reader, writer) = socket.0.into_split();
                    tokio::spawn(handle_socket(reader, tx.clone()));
                    users.push(writer);
                }
            }
            message = rx.recv() => {
                if let Some(message) = message {
                for user in users.iter_mut() {
                    user.write(message.payload.as_bytes()).await.map_err(|err| {
                            eprintln!("ERROR: Unable to write to stream {err}")
                        }).unwrap();
                }
                }
            }
        }
    }
}

async fn handle_socket(mut stream: OwnedReadHalf, sender: Sender<Message>) {
    let mut buffer = [0; 1024];
    loop {
        if let Ok(n) = stream.read(&mut buffer).await {
            let message = std::str::from_utf8(&buffer[..n]).unwrap();
            sender
                .send(Message {
                    payload: message.to_string(),
                })
                .await
                .map_err(|err| eprintln!("ERROR: Unable to write to channel {err}"))
                .unwrap();
        }
    }
}
