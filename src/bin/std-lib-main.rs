use std::{io, thread};
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::mpsc::{Sender, Receiver};
use std::sync::{Arc, mpsc, Mutex};

struct User {
    stream: TcpStream
}

struct Message {
    payload: String,
}

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:5050")?;
    let (tx, rx): (Sender<Message>, Receiver<Message>) = mpsc::channel();

    let users: Arc<Mutex<Vec<User>>> = Arc::new(Mutex::new(vec!()));

    let receiver_users = Arc::clone(&users);
    thread::spawn(move|| {
        loop {
            let message = rx.recv().unwrap();
            let mut u = receiver_users.lock().unwrap();

            for x in u.iter_mut() {
                x.stream.write_all(message.payload.as_bytes()).unwrap();
            }
        }
    });

    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            let user = User { stream: stream.try_clone().unwrap() };
            users.lock().unwrap().push(user);
            let sender = tx.clone();

            thread::spawn(|| {
                handle_stream(stream, sender).unwrap();
            });
        }
    }


    Ok(())
}

fn handle_stream(mut stream: TcpStream, sender: Sender<Message>) -> io::Result<()> {

    let mut buffer = [0; 1024];
    loop {
        match stream.read(&mut buffer) {
            Ok(size) => {
                if let Ok(message) = std::str::from_utf8(&buffer[..size]) {
                    if message.trim() == "help" {
                        stream.write("stop - close the connection\n".as_bytes())?;
                    }

                    if message.trim().eq("stop") {
                        stream.write("Closing the connection\n".as_bytes())?;
                        stream.shutdown(Shutdown::Both).map_err(|err| {
                            eprintln!("Unable to shutdown connection {err}")
                        }).unwrap();
                        return Ok(())
                    }

                    sender.send(Message { payload: message.to_string() } ).unwrap();
                } else if let Err(err) = std::str::from_utf8(&buffer[..size]) {
                    eprintln!("Error converting bytes to utf-8 {err}")
                }
            }
            Err(err) => {
                eprintln!("Error reading from stream {err}")
            }
        };
    }
}

