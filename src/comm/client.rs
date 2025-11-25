use std::net::TcpStream;
use crate::comm::conn::Conn;
use crate::comm::conn_event::{ConnEvent, ConnEventType};

pub fn connect() {
    let conn = Conn::new(TcpStream::connect("127.0.0.1:4455").unwrap());

    if let Ok(mut publisher) = conn.events() {
        publisher.subscribe(ConnEventType::MsgReceived, received_message);
    }

    conn.send_msg("ClientHello\n".to_owned());

    conn.wait_for_shutdown();

    conn.close();
}

pub fn received_message(event: ConnEvent) {
    println!("Message received: {}", event.payload);
}