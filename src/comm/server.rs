use std::io::{Error, ErrorKind};
use std::net::TcpListener;
use std::path::Path;
use crate::comm::conn::Conn;
use crate::util;
use crate::util::observer::Event;

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
            publisher.subscribe(Event::MsgReceived, print_received_message)
        }
        connections.push(new_conn);
    }

    Ok(())
}

pub fn print_received_message(msg: String) {
    println!("Message received: {}", msg);
}