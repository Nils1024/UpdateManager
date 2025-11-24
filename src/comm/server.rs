use std::net::TcpListener;
use std::path::Path;
use crate::comm::conn::Conn;
use crate::comm::conn_event::{ConnEvent, ConnEventType};
use crate::util;
use crate::util::config::get_config;

/// Starts the server and waits for incoming connections.
pub fn start() -> std::io::Result<()> {
    let socket = TcpListener::bind(format!("127.0.0.1:{}", get_config().get("port").unwrap()))?;
    let mut connections = Vec::new();

    for stream in socket.incoming() {
        let new_conn = Conn::new(stream?);

        println!("New client");

        if let Ok(mut publisher) = new_conn.events() {
            publisher.subscribe(ConnEventType::MsgReceived, received_message);
        }
        connections.push(new_conn);
    }

    Ok(())
}

pub fn received_message(event: ConnEvent) {
    println!("Message received: {}", event.payload);

    if event.payload == "ClientHello\n" {
        event.source.send_msg(util::hash::get_dir_hash(Path::new("./")));
    }
}