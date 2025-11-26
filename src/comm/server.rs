use std::net::{TcpListener};
use std::path::Path;
use std::sync::Arc;
use crate::comm::conn::Conn;
use crate::comm::conn_event::{ConnEvent, ConnEventType};
use crate::util;

pub struct Server {
    dir_hash: String,
    socket: TcpListener,
    connections: Vec<Arc<Conn>>,
}

impl Server {
    pub fn new(addr: String) -> Self {
        Server {
            dir_hash: util::hash::get_dir_hash(Path::new("./")),
            socket: TcpListener::bind(addr).unwrap(),
            connections: Vec::new(),
        }
    }

    /// Starts the server and waits for incoming connections.
    pub fn start(&mut self) -> std::io::Result<()> {
        for stream in self.socket.incoming() {
            let new_conn = Conn::new(stream?);

            println!("New client");

            let hash = self.get_hash();

            self.connections.push(new_conn);

            if let Some(stored_conn) = self.connections.last_mut() {
                if let Ok(mut publisher) = stored_conn.events() {
                    publisher.subscribe(ConnEventType::MsgReceived, move |event| {
                        handle_message(event, &hash);
                    });
                }
            }
        }

        Ok(())
    }

    pub fn get_hash(&self) -> String {
        self.dir_hash.clone()
    }
}

fn handle_message(event: ConnEvent, hash: &str) {
    println!("Message received: {}", event.payload);

    if event.payload == "ClientHello\n" {
        event.source.send_msg(hash.to_string());
    }
}
