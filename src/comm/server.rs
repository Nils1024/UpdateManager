use std::net::{TcpListener};
use std::path::Path;
use crate::comm::conn::Conn;
use crate::comm::conn_event::{ConnEvent, ConnEventType};
use crate::util;

pub struct Server {
    dir_hash: String,
    socket: TcpListener,
}

impl Server {
    pub fn new(addr: String) -> Self {
        Server {
            dir_hash: util::hash::get_dir_hash(Path::new("./")),
            socket: TcpListener::bind(addr).unwrap(),
        }
    }

    /// Starts the server and waits for incoming connections.
    pub fn start(&self) -> std::io::Result<()> {
        let mut connections = Vec::new();

        for stream in self.socket.incoming() {
            let new_conn = Conn::new(stream?);

            println!("New client");

            let hash = self.get_hash();

            if let Ok(mut publisher) = new_conn.events() {
                publisher.subscribe(ConnEventType::MsgReceived, move |event| {
                    received_message(event, &hash);
                });
            }
            connections.push(new_conn);
        }

        Ok(())
    }

    pub fn get_hash(&self) -> String {
        self.dir_hash.clone()
    }
}

fn received_message(event: ConnEvent, hash: &str) {
    println!("Message received: {}", event.payload);

    if event.payload == "ClientHello\n" {
        event.source.send_msg(hash.to_string());
    }
}
