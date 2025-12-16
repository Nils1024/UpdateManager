use std::{env, fs};
use std::cell::RefCell;
use std::collections::HashMap;
use std::net::{TcpListener};
use std::path::{PathBuf};
use std::sync::{Arc, Mutex};
use crate::comm::conn::{new_thread_for_closing_conn, Conn};
use crate::comm::conn_event::{ConnEventType};
use crate::{util};
use crate::comm::conn_state::ConnState;
use crate::comm::protocol::{get_different_files, send_greeting_answer};
use crate::comm::session::Session;
use crate::util::files::is_excluded;

pub struct Server {
    hashes: HashMap<String, String>,
    socket: TcpListener,
    sessions: Vec<Arc<Mutex<Session>>>
}

impl Server {
    pub fn new(addr: String) -> Self {
        Server {
            hashes: Self::get_file_hashes(Self::get_updates_folder()),
            socket: TcpListener::bind(addr).unwrap(),
            sessions: Vec::new()
        }
    }

    /// Starts the server and waits for incoming connections.
    pub fn start(&mut self) -> std::io::Result<()> {
        for stream in self.socket.incoming() {
            let new_conn = Conn::new(stream?);

            println!("New client: {}", new_conn.get_address());

            let hashes = self.get_hashes();

            let session = Session {
                conn: new_conn.clone(),
                state: ConnState::Connected,
                nonce: 0,
                buffer: Vec::new()
            };

            let session_ref = Arc::new(Mutex::new(session));
            self.sessions.push(session_ref.clone());

            let session_for_callback = session_ref.clone();

            if let Ok(mut publisher) = new_conn.events() {
                publisher.subscribe(ConnEventType::MsgReceived, move |event| {
                    if let Ok(mut current_session) = session_for_callback.lock() {
                        match current_session.state {
                            ConnState::Connected => {
                                if let Ok(greeting) = String::from_utf8(Vec::from(&*event.payload))
                                    && greeting == util::constants::GREETING_MSG {
                                    current_session.change_state(ConnState::HandshakeCompleted);

                                    let nonce = send_greeting_answer(event.source, hashes.clone());
                                    current_session.nonce = nonce;
                                } else {
                                    current_session.change_state(ConnState::Finished);

                                    new_thread_for_closing_conn(current_session.conn.clone());
                                }
                            }
                            ConnState::HandshakeCompleted => {
                                current_session.buffer.extend_from_slice(&event.payload);

                                if let Some(different_files) = get_different_files(&current_session.buffer) {
                                    let update_folder = Self::get_updates_folder();
                                    for file in different_files {
                                        let mut path = update_folder.clone();
                                        path.push(file);

                                        event.source.send_file(&*path);
                                    }

                                    current_session.change_state(ConnState::Finished);
                                    new_thread_for_closing_conn(current_session.conn.clone());
                                }
                            }
                            ConnState::Update => {}
                            ConnState::Finished => {
                                new_thread_for_closing_conn(current_session.conn.clone());
                            }
                        }
                    }
                });
            }
        }

        Ok(())
    }

    pub fn get_hashes(&self) -> HashMap<String, String> {
        self.hashes.clone()
    }

    fn get_file_hashes(path: PathBuf) -> HashMap<String, String> {
        let file_hashes: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());

        util::files::walk_file_tree(path.as_path(), &|entry| {
            if is_excluded(entry) {
                return
            }

            let path = entry.path().to_str().unwrap().to_string();
            let mut base_path = env::current_dir().unwrap();
            base_path.push(util::constants::UPDATES_FOLDER_NAME);
            let absolute_file_path = fs::canonicalize(path).unwrap();
            let relative_path = absolute_file_path.strip_prefix(&base_path)
                .unwrap_or(&absolute_file_path);
            let hash = util::hash::get_file_hash(&entry.path());

            file_hashes.borrow_mut().insert(relative_path.to_str().unwrap().to_string(), hash);
        }).expect("failed to walk_file_tree");

        file_hashes.into_inner()
    }

    fn get_updates_folder() -> PathBuf {
        let exe_dir = env::current_exe().unwrap();
        let mut update_dir = exe_dir.parent().unwrap().to_path_buf();
        update_dir.push("updates");

        update_dir
    }
}
