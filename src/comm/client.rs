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

pub fn connect() -> bool {
    let addr = format!(
        "{}:{}",
        get_config()
            .get(util::constants::CONFIG_ADDRESS_KEY)
            .unwrap_or(&util::constants::STD_ADDRESS.to_string()),
        get_config()
            .get(util::constants::CONFIG_PORT_KEY)
            .unwrap_or(&util::constants::STD_PORT.to_string())
    );

    let stream = match TcpStream::connect(addr) {
        Ok(stream) => stream,
        Err(_) => return false,
    };

    let conn = Conn::new(stream);

    let session = Session {
        conn: conn.clone(),
        state: ConnState::Connected,
    };

    let session_ref = Arc::new(Mutex::new(session));
    let is_meta_data = Arc::new(AtomicBool::new(true));
    let remaining_bytes = Arc::new(AtomicUsize::new(0));
    let file_stream = Arc::new(Mutex::new(None::<BufWriter<File>>));
    let metadata_buffer = Arc::new(Mutex::new(Vec::new()));

    if let Ok(mut publisher) = conn.events() {
        publisher.subscribe(ConnEventType::MsgReceived, move |event| {
            println!("Message received: {:?}", event.payload);

            if let Ok(mut current_session) = session_ref.lock() {
                if current_session.state == ConnState::Connected {
                    let hash = util::hash::get_dir_hash(Path::new("./"));

                    if String::from_utf8_lossy(&*event.payload) != hash {
                        event.source.send_msg_string(hash);
                        current_session.change_state(ConnState::Update)
                    } else {
                        event.source.close();
                    }
                } else if current_session.state == ConnState::Update {
                    if is_meta_data.load(Ordering::Acquire) {
                        if let Ok(meta_data) = json::parse(&*String::from_utf8_lossy(&event.payload[0..(event.payload.len() - 1)])) {
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
                        let remaining_bytes_val = remaining_bytes.load(Ordering::Acquire);
                        let payload_len = event.payload.len();

                        if remaining_bytes_val >= payload_len {
                            if let Some(stream) = file_stream.lock().unwrap().as_mut() {
                                stream.write_all(&event.payload).unwrap();

                                let new_remaining = remaining_bytes_val - payload_len;
                                remaining_bytes.store(new_remaining, Ordering::Release);

                                if new_remaining == 0 {
                                    stream.flush().unwrap();
                                    is_meta_data.store(true, Ordering::Release);
                                }
                            }
                        } else {
                            if let Some(stream) = file_stream.lock().unwrap().as_mut() {
                                stream.write_all(&event.payload[0..remaining_bytes_val]).unwrap();
                                let _ = stream.flush();
                            }

                            let leftover_slice = &event.payload[remaining_bytes_val..];
                            let mut guard = metadata_buffer.lock().unwrap();

                            match leftover_slice.iter().position(|&b| b == 0) {
                                Some(zero_byte_index) => {
                                    guard.extend_from_slice(&leftover_slice[..zero_byte_index]);
                                    let json = String::from_utf8_lossy(&guard);

                                    if let Ok(meta_data) = json::parse(&json) {
                                        let mut new_size = 0;

                                        if let Some(size_val) = meta_data["size"].as_usize() {
                                            new_size = size_val;
                                        }
                                        if let Some(name_val) = meta_data["name"].as_str() {
                                            let file = File::create(name_val);
                                            let mut writer = BufWriter::new(file.unwrap());

                                            let new_file_data_start = zero_byte_index + 1;

                                            if new_file_data_start < leftover_slice.len() {
                                                let data_for_new_file = &leftover_slice[new_file_data_start..];
                                                writer.write_all(data_for_new_file).unwrap();

                                                let written = data_for_new_file.len();

                                                if written >= new_size {
                                                    writer.flush().unwrap();
                                                    is_meta_data.store(true, Ordering::Release);
                                                    remaining_bytes.store(0, Ordering::Release);
                                                } else {
                                                    remaining_bytes.store(new_size - written, Ordering::Release);
                                                }
                                            } else {
                                                remaining_bytes.store(new_size, Ordering::Release);
                                            }

                                            *file_stream.lock().unwrap() = Some(writer);
                                        }
                                    }

                                    guard.clear();
                                },
                                None => {
                                    guard.extend_from_slice(leftover_slice);
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    conn.send_msg_string(util::constants::GREETING_MSG.to_owned());

    conn.wait_for_shutdown();

    conn.close();

    true
}
