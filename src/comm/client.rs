use std::net::TcpStream;
use std::path::Path;
use crate::comm::conn::Conn;
use crate::comm::conn_event::{ConnEvent, ConnEventType};
use crate::util;
use crate::util::config::get_config;

pub fn connect() {
    if !util::config::does_config_exists() {
        get_config();

        if !util::config::does_config_exists() {
            eprintln!("Failed to create config file");
            return;
        }
    }

    let conn = Conn::new(
        TcpStream::connect(
            format!("{}:{}",
                    get_config().get(util::constants::CONFIG_ADDRESS_KEY).unwrap_or(&util::constants::STD_ADDRESS.to_string()),
                    get_config().get(util::constants::CONFIG_PORT_KEY).unwrap_or(&util::constants::STD_PORT.to_string()))).unwrap());

    if let Ok(mut publisher) = conn.events() {
        publisher.subscribe(ConnEventType::MsgReceived, received_message);
    }

    conn.send_msg_string(util::constants::GREETING_MSG.to_owned());

    conn.wait_for_shutdown();

    conn.close();
}

pub fn received_message(event: ConnEvent) {
    println!("Message received: {:?}", event.payload);

    let hash = util::hash::get_dir_hash(Path::new("./"));

    if String::from_utf8_lossy(&*event.payload) != hash {
        event.source.send_msg_string(hash);
    }
}