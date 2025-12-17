use std::{collections::VecDeque, env, fs, io::{ErrorKind, Read, Write}, net::{Shutdown, TcpStream}, sync::{Arc, Condvar, Mutex, atomic::{AtomicBool, Ordering}}, thread::{self, JoinHandle}, time::Duration};
use std::fs::File;
use std::io::BufReader;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::sync::{LockResult, MutexGuard};
use json::object;
use crate::comm::conn_event::ConnEvent;
use crate::comm::conn_event::ConnEventType::MsgReceived;
use crate::util;
use crate::util::constants::get_exe_dir;
use crate::util::observer::observer::Publisher;

#[derive(Clone)]
pub struct Conn {
    reader: Arc<Mutex<TcpStream>>,
    writer: Arc<Mutex<TcpStream>>,
    send_messages: Arc<(Mutex<VecDeque<Vec<u8>>>, Condvar)>,
    received_messages: Arc<Mutex<Vec<Vec<u8>>>>,
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

    pub fn send_msg_string(&self, msg: String) {
        let (lock, cvar) = &*self.send_messages;
        let mut queue = lock.lock().unwrap();
        queue.push_back(msg.as_bytes().to_vec());
        cvar.notify_all();
    }

    pub fn send_msg(&self, msg: Vec<u8>) {
        let (lock, cvar) = &*self.send_messages;
        let mut queue = lock.lock().unwrap();
        queue.push_back(msg);
        cvar.notify_all();
    }

    pub fn send_file(&self, path: &Path) {
        let (lock, cvar) = &*self.send_messages;
        let mut queue = lock.lock().unwrap();

        if let Ok(file) = File::open(path) {
            if let Ok(meta_data) = file.metadata() {
                let base_path = get_exe_dir().join(util::constants::UPDATES_FOLDER_NAME);
                let absolute_file_path = fs::canonicalize(path).unwrap();
                let relative_path = absolute_file_path.strip_prefix(&base_path)
                    .unwrap_or(&absolute_file_path);
                
                let meta_data_json = object! {
                    "name": relative_path.to_str().unwrap().replace("\\", "/"),
                    "size": meta_data.len(),
                    "is_app": meta_data.permissions().mode() & 0o111 != 0
                };

                let mut meta_data_bytes = meta_data_json.to_string().into_bytes();
                meta_data_bytes.push(0);
                queue.push_back(meta_data_bytes);

                let file_bytes = fs::read(path).unwrap();

                queue.push_back(file_bytes);
            }
        }

        cvar.notify_all();
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

    pub fn events(&self) -> LockResult<MutexGuard<'_, Publisher<ConnEvent>>> {
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

    pub fn wait_until_msg_queue_is_empty(&self) {
        let (lock, cvar) = &*self.send_messages;

        let queue = lock.lock().unwrap();

        drop(cvar.wait_while(queue, |q| {
            !q.is_empty() && self.running.load(Ordering::SeqCst)
        }));
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
                    let msg = buf[..size].to_vec();
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
                    if let Err(e) = stream.write_all(&*msg) {
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

pub fn new_thread_for_closing_conn(conn: Arc<Conn>) {
    thread::spawn(move || {
        conn.wait_until_msg_queue_is_empty();
        conn.close();
    });
}
