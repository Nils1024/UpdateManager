use std::net::TcpStream;
use std::thread;
use std::time::Duration;
use crate::comm::conn::Conn;

pub fn connect() {
    let conn = Conn::new(TcpStream::connect("127.0.0.1:1234").unwrap());

    conn.send_msg("HALLO\n".to_owned());

    thread::sleep(Duration::from_secs(2));

    conn.close();
}