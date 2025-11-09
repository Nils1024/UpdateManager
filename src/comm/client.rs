use std::net::TcpStream;

use crate::comm::conn::Conn;

pub fn connect() {
    let conn = Conn::new(TcpStream::connect("127.0.0.1:1234").unwrap());

    conn.send_msg("HALLO\n".to_owned());

    conn.close();
}