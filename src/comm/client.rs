use std::net::TcpStream;
use std::thread;
use std::time::Duration;
use crate::comm::conn::Conn;
use crate::comm::conn_event::{ConnEvent, ConnEventType};

pub fn connect() {
    let conn = Conn::new(TcpStream::connect("127.0.0.1:1234").unwrap());

    if let Ok(mut publisher) = conn.events() {
        publisher.subscribe(ConnEventType::MsgReceived, received_message);
    }

    conn.send_msg("ClientHello\n".to_owned());

    thread::sleep(Duration::from_secs(2));

    conn.close();
}

pub fn received_message(event: ConnEvent) {
    println!("Message received: {}", event.payload);
}