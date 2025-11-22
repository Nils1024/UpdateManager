use std::io::{Error, ErrorKind};
use std::net::TcpListener;
use std::path::Path;
use crate::comm::conn::Conn;
use crate::comm::conn_event::{ConnEvent, ConnEventType};
use crate::util;

/// Starts the server and waits for incoming connections.
pub fn start() -> std::io::Result<()> {
    let binding = util::config::get_config_name();
    let path = Path::new(&binding);

    if util::config::does_config_exists() {
        util::config::read_config(path)
    } else {
        match util::config::write_default_config(path) {
            Ok(_) => util::config::read_config(path),
            Err(_) => return Err(Error::from(ErrorKind::InvalidData)),
        }
    }

    let socket = TcpListener::bind("127.0.0.1:1234")?;
    let mut connections = Vec::new();

    for stream in socket.incoming() {
        let new_conn = Conn::new(stream?);

        println!("New client");
        new_conn.send_msg("Hello\n".to_owned());

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
        util::hash::get_dir_hash(Path::new("./"));
    }
}