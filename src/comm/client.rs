use std::fs::File;
use std::io::{BufWriter, Write};
use std::net::TcpStream;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use crate::comm::conn::Conn;
use crate::comm::conn_event::{ConnEventType};
use crate::comm::conn_state::ConnState;
use crate::comm::session::Session;
use crate::util;
use crate::util::config::get_config;

pub fn connect() {
    if !util::config::does_config_exists() {
        get_config();

        if !util::config::does_config_exists() {
            eprintln!("Failed to create config file");
            return;
        }
    }

    let conn = Conn::new(
        TcpStream::connect(
            format!("{}:{}",
                    get_config().get(util::constants::CONFIG_ADDRESS_KEY).unwrap_or(&util::constants::STD_ADDRESS.to_string()),
                    get_config().get(util::constants::CONFIG_PORT_KEY).unwrap_or(&util::constants::STD_PORT.to_string()))).unwrap());

    let session = Session {
        conn: conn.clone(),
        state: ConnState::Connected,
    };

    let session_ref = Arc::new(Mutex::new(session));
    let is_meta_data = Arc::new(AtomicBool::new(true));
    let remaining_bytes = Arc::new(AtomicUsize::new(0));
    let file_stream = Arc::new(Mutex::new(None::<BufWriter<File>>));

    if let Ok(mut publisher) = conn.events() {
        publisher.subscribe(ConnEventType::MsgReceived, move |event| {
            println!("Message received: {:?}", event.payload);

            if let Ok(mut current_session) = session_ref.lock() {
                if current_session.state == ConnState::Connected {
                    let hash = util::hash::get_dir_hash(Path::new("./"));

                    if String::from_utf8_lossy(&*event.payload) != hash {
                        event.source.send_msg_string(hash);
                        current_session.change_state(ConnState::Update)
                    }
                } else if current_session.state == ConnState::Update {
                    if is_meta_data.load(Ordering::Acquire) {
                        if let Ok(meta_data) = json::parse(&*String::from_utf8_lossy(&*event.payload)) {
                            if let Some(size_val) = meta_data["size"].as_usize() {
                                remaining_bytes.store(size_val, Ordering::Release);
                            }
                            if let Some(name_val) = meta_data["name"].as_str() {
                                let file = File::create(name_val);
                                *file_stream.lock().unwrap() = Some(BufWriter::new(file.unwrap()));
                            }

                            is_meta_data.store(false, Ordering::Release);
                        }
                    } else {
                        let mut stream = file_stream.lock().unwrap().take().unwrap();
                        stream.write_all(&*event.payload).unwrap();

                        remaining_bytes.store(remaining_bytes.load(Ordering::Acquire) - event.payload.len(), Ordering::Release);

                        if remaining_bytes.load(Ordering::Acquire) <= 0 {
                            let _ = stream.flush();
                            is_meta_data.store(true, Ordering::Release);
                        }
                    }
                }
            }
        });
    }

    conn.send_msg_string(util::constants::GREETING_MSG.to_owned());

    conn.wait_for_shutdown();

    conn.close();
}
