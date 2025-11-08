use std::{io::Write, net::{Shutdown, TcpListener, TcpStream}, thread::{self}};

#[cfg(target_os = "macos")]
fn create_pid_file() {

}

#[cfg(target_os = "linux")]
fn create_pid_file() {
    
}

#[cfg(target_os = "windows")]
fn create_pid_file() {
    
}

/// Handles new clients
fn new_client(mut conn: TcpStream) {
    println!("New client");
    conn.write(b"Hello").unwrap();
    conn.shutdown(Shutdown::Both).unwrap();
}

/// Starts the server and waits for incoming connections.
pub fn start() -> std::io::Result<()> {
    let socket = TcpListener::bind("127.0.0.1:1234")?;

    for stream in socket.incoming() {
        let stream = stream.unwrap();

        thread::spawn(|| {
            new_client(stream);
        });
    }

    Ok(())
}