use std::cmp::PartialEq;
use std::fs;
use std::net::{TcpListener};
use std::path::Path;
use std::sync::{Arc, Mutex};
use crate::comm::conn::Conn;
use crate::comm::conn_event::{ConnEventType};
use crate::util;

pub struct Server {
    dir_hash: String,
    socket: TcpListener,
    sessions: Vec<Arc<Mutex<Session>>>
}

struct Session {
    conn: Arc<Conn>,
    state: ConnState
}

impl Session {
    pub fn change_state(&mut self, new_state: ConnState) {
        self.state = new_state;
    }
}

enum ConnState {
    Connected,
    HandshakeCompleted,
    Update,
    Finished
}

impl PartialEq for ConnState {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ConnState::Connected, ConnState::Connected) => true,
            (ConnState::HandshakeCompleted, ConnState::HandshakeCompleted) => true,
            (ConnState::Update, ConnState::Update) => true,
            (ConnState::Finished, ConnState::Finished) => true,
            _ => false
        }
    }
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

                                //TODO: Send files
                                util::files::walk_file_tree(Path::new("./"), &|entry| {
                                    event.source.send_msg_string(entry.file_name().to_str().unwrap().to_string());
                                }).expect("Failed to walk_file_tree");

                                current_session.change_state(ConnState::Finished);
                            } else {
                                current_session.change_state(ConnState::Finished);
                            }
                        }

                        if String::from_utf8_lossy(&*event.payload) == "ClientHello\n" {
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
