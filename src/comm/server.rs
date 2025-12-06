use std::{env, fs, thread};
use std::net::{TcpListener};
use std::path::Path;
use std::sync::{Arc, Mutex};
use crate::comm::conn::Conn;
use crate::comm::conn_event::{ConnEventType};
use crate::{util};
use crate::comm::conn_state::ConnState;
use crate::comm::session::Session;

pub struct Server {
    dir_hash: String,
    socket: TcpListener,
    sessions: Vec<Arc<Mutex<Session>>>
}

impl Server {
    pub fn new(addr: String) -> Self {
        Server {
            dir_hash: util::hash::get_dir_hash(Path::new("./")),
            socket: TcpListener::bind(addr).unwrap(),
            sessions: Vec::new()
        }
    }

    /// Starts the server and waits for incoming connections.
    pub fn start(&mut self) -> std::io::Result<()> {
        for stream in self.socket.incoming() {
            let new_conn = Conn::new(stream?);

            println!("New client: {}", new_conn.get_address());

            let hash = self.get_hash();

            let session = Session {
                conn: new_conn.clone(),
                state: ConnState::Connected,
            };

            let session_ref = Arc::new(Mutex::new(session));
            self.sessions.push(session_ref.clone());

            let session_for_callback = session_ref.clone();

            if let Ok(mut publisher) = new_conn.events() {
                publisher.subscribe(ConnEventType::MsgReceived, move |event| {
                    println!("Message received: {:?}", event.payload);

                    if let Ok(mut current_session) = session_for_callback.lock() {
                        if current_session.state == ConnState::HandshakeCompleted {
                            if String::from_utf8_lossy(&*event.payload) != hash.to_string() {
                                current_session.change_state(ConnState::Update);

                                let exe_dir = env::current_exe().unwrap();

                                util::files::walk_file_tree(exe_dir.parent().unwrap(), &|entry| {
                                    if util::files::is_excluded(entry) {
                                        return
                                    }

                                    event.source.send_file(&*entry.path());
                                }).expect("Failed to walk_file_tree");

                                current_session.change_state(ConnState::Finished);

                                let conn_clone = current_session.conn.clone();
                                thread::spawn(move || {
                                    conn_clone.wait_until_msg_queue_is_empty();
                                    conn_clone.close();
                                });
                            } else {
                                current_session.change_state(ConnState::Finished);
                            }
                        }

                        if String::from_utf8_lossy(&*event.payload) == util::constants::GREETING_MSG {
                            current_session.change_state(ConnState::HandshakeCompleted);
                            event.source.send_msg_string(hash.to_string());
                        }
                    }
                });
            }
        }

        Ok(())
    }

    pub fn get_hash(&self) -> String {
        self.dir_hash.clone()
    }
}
