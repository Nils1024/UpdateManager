use std::{collections::VecDeque, io::{ErrorKind, Read, Write}, net::{Shutdown, TcpStream}, sync::{Arc, Condvar, Mutex, atomic::{AtomicBool, Ordering}}, thread::{self, JoinHandle}, time::Duration};
use std::io::BufReader;
use std::sync::{LockResult, MutexGuard};
use crate::comm::conn_event;
use crate::comm::conn_event::ConnEvent;
use crate::comm::conn_event::ConnEventType::MsgReceived;
use crate::util::observer::observer::Publisher;

#[derive(Clone)]
pub struct Conn {
    reader: Arc<Mutex<TcpStream>>,
    writer: Arc<Mutex<TcpStream>>,
    send_messages: Arc<(Mutex<VecDeque<String>>, Condvar)>,
    received_messages: Arc<Mutex<Vec<String>>>,
    running: Arc<AtomicBool>,
    reader_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    writer_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    publisher: Arc<Mutex<Publisher<ConnEvent>>>
}

impl Conn {
    pub fn new(stream: TcpStream) -> Arc<Self> {
        let reader = stream.try_clone().unwrap();
        let writer = stream;
        
        let conn = Arc::new(Self {
            reader: Arc::new(Mutex::new(reader)),
            writer: Arc::new(Mutex::new(writer)),
            send_messages: Arc::new((Mutex::new(VecDeque::new()), Condvar::new())),
            received_messages: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(AtomicBool::new(true)),
            reader_handle: Arc::new(Mutex::new(None)),
            writer_handle: Arc::new(Mutex::new(None)),
            publisher: Arc::new(Mutex::new(Publisher::default()))
        });
        
        let reader_conn = Arc::clone(&conn);
        let reader_handle = thread::spawn(move || reader_conn.reader());
        *conn.reader_handle.lock().unwrap() = Some(reader_handle);

        let writer_conn = Arc::clone(&conn);
        let writer_handle = thread::spawn(move || writer_conn.writer());
        *conn.writer_handle.lock().unwrap() = Some(writer_handle);

        conn
    }

    pub fn send_msg(&self, msg: String) {
        let (lock, cvar) = &*self.send_messages;
        let mut queue = lock.lock().unwrap();
        queue.push_back(msg);
        cvar.notify_one();
    }

    pub fn close(&self) {
        if !self.running.load(Ordering::SeqCst) {
            return;
        }

        self.running.store(false, Ordering::SeqCst);

        let (_, cvar) = &*self.send_messages;
        cvar.notify_all();

        if let Ok(reader) = self.reader.lock() {
            let _ = reader.shutdown(Shutdown::Both);
        }
        if let Ok(writer) = self.writer.lock() {
            let _ = writer.shutdown(Shutdown::Both);
        }
    }

    pub fn events(&self) -> LockResult<MutexGuard<'_, Publisher<conn_event::ConnEvent>>> {
        self.publisher.lock()
    }

    pub fn wait_for_shutdown(&self) {
        // Wait for reader thread to end
        let reader_handle = self.reader_handle.lock().unwrap().take();
        if let Some(handle) = reader_handle {
            if let Err(e) = handle.join() {
                eprintln!("Reader Thread Panicked: {:?}", e);
            }
        }

        // Wait for writer thread to end
        let writer_handle = self.writer_handle.lock().unwrap().take();
        if let Some(handle) = writer_handle {
            if let Err(e) = handle.join() {
                eprintln!("Writer Thread Panicked: {:?}", e);
            }
        }
    }

    pub fn get_address(&self) -> String {
        self.reader.lock().unwrap().peer_addr().unwrap().to_string()
    }

    fn reader(&self) {
        let stream = match self.reader.lock() {
            Ok(guard) => {
                guard.try_clone().unwrap()
            }
            Err(_) => return,
        };

        let mut reader = BufReader::new(stream);
        let mut buf = vec![0u8; 1024];

        while self.running.load(Ordering::SeqCst) {
            match reader.read(&mut buf) {
                Ok(0) => {
                    self.close();
                    break;
                }
                Ok(size) => {
                    let msg = String::from_utf8_lossy(&buf[..size]).to_string();
                    self.received_messages.lock().unwrap().push(msg.clone());

                    let event = ConnEvent {
                        event_type: MsgReceived,
                        source: self.clone(),
                        timestamp: 123,
                        payload: msg
                    };

                    if let Ok(pub_lock) = self.publisher.lock() {
                        pub_lock.notify(event);
                    }
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock
                    || e.kind() == ErrorKind::TimedOut => {

                    thread::sleep(Duration::from_millis(2));
                    eprintln!("Timeout for 2 seconds");
                    continue;
                }
                Err(e) => {
                    eprintln!("Error reading from a stream: {e}");
                    self.close();
                    break;
                }
            }
        }
    }

    fn writer(&self) {
        let (lock, cvar) = &*self.send_messages;

        while self.running.load(Ordering::SeqCst) {
            let mut queue = lock.lock().unwrap();

            while queue.is_empty() && self.running.load(Ordering::SeqCst) {
                queue = cvar.wait(queue).unwrap();
            }

            while let Some(msg) = queue.pop_front() {
                if let Ok(mut stream) = self.writer.lock() {
                    println!("Sending: {:?}", msg);
                    
                    if let Err(e) = stream.write_all(msg.as_bytes()) {
                        eprintln!("Error writing to a stream: {e}");
                        return;
                    }
                    if let Err(e) = stream.flush() {
                        eprintln!("Error flushing stream. {e}");
                        return;
                    }
                }
            }
        }
    }
}
