use std::net::TcpListener;
use std::thread;
use std::time::Duration;
use update_manager::util::observer::Event;
use crate::comm::conn::Conn;

#[cfg(target_os = "macos")]
fn create_pid_file() {

}

#[cfg(target_os = "linux")]
fn create_pid_file() {
    
}

#[cfg(target_os = "windows")]
fn create_pid_file() {
    
}

/// Starts the server and waits for incoming connections.
pub fn start() -> std::io::Result<()> {
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