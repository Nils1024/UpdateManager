use std::{collections::VecDeque, io::{ErrorKind, Read, Write}, net::{Shutdown, TcpStream}, sync::{Arc, Condvar, Mutex, atomic::{AtomicBool, Ordering}}, thread::{self, JoinHandle}, time::Duration};
use std::sync::{LockResult, MutexGuard};
use update_manager::util::observer::{Event, Publisher};

pub struct Conn {
    reader: Arc<Mutex<TcpStream>>,
    writer: Arc<Mutex<TcpStream>>,
    send_messages: Arc<(Mutex<VecDeque<String>>, Condvar)>,
    received_messages: Arc<Mutex<Vec<String>>>,
    running: Arc<AtomicBool>,
    reader_handle: Mutex<Option<JoinHandle<()>>>,
    writer_handle: Mutex<Option<JoinHandle<()>>>,
    publisher: Mutex<Publisher>
}

impl Conn {
    pub fn new(stream: TcpStream) -> Arc<Self> {
        stream.set_read_timeout(Some(Duration::from_secs(5))).ok();

        let reader = stream.try_clone().unwrap();
        let writer = stream;
        
        let conn = Arc::new(Self {
            reader: Arc::new(Mutex::new(reader)),
            writer: Arc::new(Mutex::new(writer)),
            send_messages: Arc::new((Mutex::new(VecDeque::new()), Condvar::new())),
            received_messages: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(AtomicBool::new(true)),
            reader_handle: Mutex::new(None),
            writer_handle: Mutex::new(None),
            publisher: Mutex::new(Publisher::default())
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
        self.running.store(false, Ordering::SeqCst);

        let (_, cvar) = &*self.send_messages;
        cvar.notify_all();

        if let Ok(reader) = self.reader.lock() {
            let _ = reader.shutdown(Shutdown::Both);
        }
        if let Ok(writer) = self.writer.lock() {
            let _ = writer.shutdown(Shutdown::Both);
        }

        if let Some(handle) = self.reader_handle.lock().unwrap().take() {
            let _ = handle.join();
        }
        if let Some(handle) = self.writer_handle.lock().unwrap().take() {
            let _ = handle.join();
        }
    }

    pub fn events(&self) -> LockResult<MutexGuard<'_, Publisher>> {
        self.publisher.lock()
    }

    // TODO: Fix timeout
    fn reader(&self) {
        let mut buf= [0u8; 128];
        let mut buf = [0u8; 128];
        let mut consecutive_timeouts = 0u32;
        const MAX_CONSECUTIVE_TIMEOUTS: u32 = 3;

        while self.running.load(Ordering::SeqCst) {
            let mut reader = match self.reader.lock() {
                Ok(s) => s,
                Err(_) => {
                    eprintln!("Failed to acquire reader lock");
                    break;
                }
            };

            match reader.read(&mut buf) {
                Ok(0) => {
                    break;
                }
                Ok(size) => {
                    consecutive_timeouts = 0;
                    drop(reader);

                    let msg = String::from_utf8_lossy(&buf[..size]).to_string();
                    if let Ok(pub_guard) = self.publisher.lock() {
                        pub_guard.notify(Event::MsgReceived, msg.clone());
                    }

                    if let Ok(mut msgs) = self.received_messages.lock() {
                        msgs.push(msg);
                    }
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut => {
                    drop(reader);
                    consecutive_timeouts += 1;

                    if consecutive_timeouts >= MAX_CONSECUTIVE_TIMEOUTS {
                        println!("Too many consecutive timeouts, closing connection");
                        break;
                    }

                    thread::sleep(Duration::from_millis(10));
                    continue;
                }
                Err(e) => {
                    drop(reader);
                    eprintln!("Error reading from a stream: {e}");
                    break;
                }
        }
        }
        self.running.store(false, Ordering::SeqCst);
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
