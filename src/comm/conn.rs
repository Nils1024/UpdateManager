use std::{collections::VecDeque, io::{Read, Write}, net::{Shutdown, TcpStream}, sync::{Arc, Condvar, Mutex}, thread::{self}};

pub struct Conn {
    stream: Arc<Mutex<TcpStream>>,
    send_messages: Arc<(Mutex<VecDeque<String>>, Condvar)>,
    received_messages: Arc<Mutex<Vec<String>>>
}

impl Conn {
    pub fn new(stream: TcpStream) -> Arc<Self> {
        let conn = Arc::new(Self {
            stream: Arc::new(Mutex::new(stream)),
            send_messages: Arc::new((Mutex::new(VecDeque::new()), Condvar::new())),
            received_messages: Arc::new(Mutex::new(Vec::new()))
        });
        
        let reader_conn = Arc::clone(&conn);
        thread::spawn(move || {
            reader_conn.reader();
        });

        let writer_conn = Arc::clone(&conn);
        thread::spawn(move || {
            writer_conn.writer();
        });

        conn.send_msg("tes".to_owned());

        conn
    }

    pub fn send_msg(&self, msg: String) {
        let (lock, cvar) = &*self.send_messages;
        let mut queue = lock.lock().unwrap();
        queue.push_back(msg);
        cvar.notify_one();
    }

    pub fn close(&self) {
        let _ = self.stream.lock().unwrap().shutdown(Shutdown::Both);
    }

    fn reader(&self) {
        let mut buf= [0u8; 128];

        loop {
            let mut stream = self.stream.lock().unwrap();
            match stream.read(&mut buf) {
                Ok(0) => break,
                Ok(size) => {
                    let msg = String::from_utf8_lossy(&buf[..size]).to_string();
                    self.received_messages.lock().unwrap().push(msg);
                }
                Err(e) => {
                    eprintln!("Error reading from a stream: {e}");
                    break;
                }
            }
        }
    }

    fn writer(&self) {
        let (lock, cvar) = &*self.send_messages;

        loop {
            let mut queue = lock.lock().unwrap();

            while queue.is_empty() {
                queue = cvar.wait(queue).unwrap();
            }

            while let Some(msg) = queue.pop_front() {
                if let Ok(mut stream) = self.stream.lock() {
                    if let Err(e) = stream.write_all(msg.as_bytes()) {
                        eprintln!("Error writing to a stream: {e}");
                        return;
                    }
                }
            }
        }
    }
}
