use std::{collections::VecDeque, io::{ErrorKind, Read, Write}, net::{Shutdown, TcpStream}, sync::{Arc, Condvar, Mutex, atomic::{AtomicBool, Ordering}}, thread::{self, JoinHandle}, time::Duration};
use std::io::BufReader;
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
    }

    pub fn events(&self) -> LockResult<MutexGuard<'_, Publisher>> {
        self.publisher.lock()
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
                    self.publisher.lock().unwrap().notify(Event::MsgReceived, msg);
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
