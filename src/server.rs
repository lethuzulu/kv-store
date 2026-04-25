use crate::protocol::{
    Command, DeleteResponse, GetResponse, Request, Response, ResponseKind, deserialize_request,
};
use crate::store::KvStore;
use anyhow::Result;
use std::io::{BufRead, BufReader};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};

#[derive(Debug)]
pub struct TcpServer {
    inner: TcpListener,
    store: KvStore,
}

impl TcpServer {
    pub fn new(address: impl ToSocketAddrs) -> Result<Self> {
        let inner = TcpListener::bind(address)?;
        let store = KvStore::new("store.log")?;
        println!("Tcp Server listening in...");
        Ok(Self { inner, store })
    }

    pub fn listen(&mut self) {
        for connection in self.inner.incoming() {
            match connection {
                Ok(stream) => {
                    Self::handle_connection(&stream, &mut self.store);
                }
                Err(_) => {
                    println!("Error occurred. Listening for next connection");
                }
            }
        }
    }

    fn handle_connection(stream: &TcpStream, store: &mut KvStore) {
        let mut buffer = String::new();
        let mut reader = BufReader::new(stream);
        loop {
            reader.read_line(&mut buffer).unwrap();

            let request = deserialize_request(&buffer).unwrap();
            buffer.clear();
            // let response = handler(request, store);
        }
    }
}

fn handler(request: Request, store: &mut KvStore) -> Response {
    match request.cmd {
        Command::Set { key, value } => {
            return match store.set(key, value) {
                Ok(_) => Response {
                    result: Ok(ResponseKind::Set),
                }, // durability success
                Err(_) => Response {
                    result: Err("failure".to_string()),
                }, // durability failure
            };
        }
        Command::Get { key } => {
            match store.get(key.as_str()) {
                Some(t) => {
                    return Response {
                        result: Ok(ResponseKind::Get(GetResponse::Found(t))),
                    };
                } // retrieval success, key exists
                None => {
                    return Response {
                        result: Ok(ResponseKind::Get(GetResponse::NotFound)),
                    };
                } // retrieval failure, key does not exist
            }
        }
        Command::Delete { key } => {
            match store.delete(key) {
                Ok(r) => {
                    // durability success
                    match r {
                        true => {
                            return Response {
                                result: Ok(ResponseKind::Delete(DeleteResponse::Removed)),
                            };
                        }
                        false => {
                            return Response {
                                result: Ok(ResponseKind::Delete(DeleteResponse::Removed)),
                            };
                        }
                    }
                }
                Err(_) => {
                    //durability failure
                    return Response {
                        result: Err("failure".to_string()),
                    };
                }
            }
        }
    }
}
