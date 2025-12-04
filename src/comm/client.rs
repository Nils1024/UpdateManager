use std::net::TcpStream;
use crate::comm::conn::Conn;
use crate::comm::conn_event::{ConnEvent, ConnEventType};
use crate::util::config::get_config;

pub fn connect() {
    let conn = Conn::new(
        TcpStream::connect(
            format!("{}:{}",
                    get_config().get("address").unwrap_or(&"127.0.0.1".to_string()),
                    get_config().get("port").unwrap())).unwrap());

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