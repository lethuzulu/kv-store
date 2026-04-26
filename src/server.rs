use crate::protocol::response::Response;
use crate::protocol::{
    Command, DeletePayload, GetPayload, Payload, Request, deserialize_request, serialize_response,
};
use crate::store::KvStore;
use anyhow::Result;
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug)]
pub struct TcpServer {
    inner: TcpListener,
    store: Arc<Mutex<KvStore>>,
}

impl TcpServer {
    pub fn new(address: impl ToSocketAddrs) -> Result<Self> {
        let inner = TcpListener::bind(address)?;
        let store = KvStore::new("store.log")?;
        let store = Arc::new(Mutex::new(store));
        println!("Tcp Server listening in...");
        Ok(Self { inner, store })
    }

    pub fn listen(&self) {
        for connection in self.inner.incoming() {
            match connection {
                Ok(stream) => {
                    let store = Arc::clone(&self.store);
                    thread::spawn(move || {
                        handle_connection(stream, store);
                    });
                }
                Err(_) => {
                    println!("Error occurred. Listening for next connection");
                }
            }
        }
    }
}

fn handle_connection(stream: TcpStream, store: Arc<Mutex<KvStore>>) {
    let mut line = String::new();
    let mut writer = stream.try_clone().unwrap();
    let mut reader = BufReader::new(&stream);
    loop {
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(e) => {
                eprintln!("read error: {:?}", e);
                break;
            }
        }
        let request = match deserialize_request(&line) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("deserialization error: {}", e);
                let serialized = match serialize_response(Err(e)) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("serialize error: {}", e);
                        break;
                    }
                };
                if let Err(e) = writer.write_all(&serialized) {
                    eprintln!("write error: {}", e);
                    break;
                }
                line.clear();
                continue;
            }
        };

        let result = handler(request, &store);

        let serialized_response = match serialize_response(result) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("serialize error {}", e);
                break;
            }
        };

        if let Err(e) = writer.write_all(&serialized_response) {
            eprintln!("write error: {}", e);
            break;
        }
        line.clear();
    }
}

fn handler(request: Request, store: &Arc<Mutex<KvStore>>) -> Result<Response> {
    let mut store = store.lock().unwrap();
    match request.cmd {
        Command::Set { key, value } => {
            store.set(key, value)?;
            Ok(Payload::Set.into())
        }

        Command::Delete { key } => {
            let b = store.delete(key)?;
            if b {
                Ok(DeletePayload::Removed.into())
            } else {
                Ok(DeletePayload::NotFound.into())
            }
        }

        Command::Get { key } => match store.get(&key) {
            Some(v) => Ok(GetPayload::Found(v).into()),
            None => Ok(GetPayload::NotFound.into()),
        },
    }
}
