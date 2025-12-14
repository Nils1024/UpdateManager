use std::cmp::min;
use std::fs;
use std::fs::{File, Permissions};
use std::io::{BufWriter, Write};
use std::net::TcpStream;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use crate::comm::conn::Conn;
use crate::comm::conn_event::{ConnEventType};
use crate::comm::conn_state::ConnState;
use crate::comm::session::Session;
use crate::util;
use crate::util::config::get_config;

struct Metadata {
    size: usize,
    name: String,
    permissions: Permissions
}

struct FileTransfer {
    metadata: Option<Metadata>,
    received: usize,
    file_stream: Option<BufWriter<File>>,
    buffer: Vec<u8>,
}

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
    let file_transfer_data = Arc::new(Mutex::new(None::<FileTransfer>));

    *file_transfer_data.lock().unwrap() = Some(FileTransfer {
        metadata: None,
        buffer: Vec::new(),
        received: 0,
        file_stream: None,
    });

    if let Ok(mut publisher) = conn.events() {
        publisher.subscribe(ConnEventType::MsgReceived, move |event| {
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
                    let mut guard = file_transfer_data.lock().unwrap();
                    let current_file_transfer = guard.as_mut().unwrap();

                    current_file_transfer.buffer.extend_from_slice(&event.payload);

                    loop {
                        let mut made_progress = false;

                        if is_meta_data.load(Ordering::Acquire) {
                            if let Some(zero_index) = get_zero_byte_index(&current_file_transfer.buffer) {
                                if let Some(meta_data) = get_meta_data(&current_file_transfer.buffer) {
                                    current_file_transfer.metadata = Some(meta_data);
                                    current_file_transfer.buffer.drain(0..=zero_index);
                                    current_file_transfer.received = 0;

                                    let filename = &current_file_transfer.metadata.as_ref().unwrap().name;
                                    let path = Path::new(filename);

                                    if let Some(parent) = path.parent() {
                                        if let Err(e) = fs::create_dir_all(parent) {
                                            eprintln!("Failed to create folder: {}", e);
                                        }
                                    }

                                    match File::create(path) {
                                        Ok(file) => {
                                            let perms = &current_file_transfer.metadata.as_ref().unwrap().permissions;
                                            let _ = file.set_permissions(perms.clone());
                                            current_file_transfer.file_stream = Some(BufWriter::new(file));
                                        },
                                        Err(e) => {
                                            eprintln!("Failed to create file: {}", e);
                                        }
                                    }

                                    is_meta_data.store(false, Ordering::Release);
                                    println!("Transferring File : {}", current_file_transfer.metadata.as_ref().unwrap().name);

                                    made_progress = true;
                                }
                            }
                        }

                        if !is_meta_data.load(Ordering::Acquire) {
                            if !current_file_transfer.buffer.is_empty() && current_file_transfer.file_stream.is_some() {
                                let total_size = current_file_transfer.metadata.as_ref().unwrap().size;
                                let written = current_file_transfer.received;
                                let bytes_remaining = total_size - written;
                                let bytes_to_write = min(current_file_transfer.buffer.len(), bytes_remaining);

                                if bytes_to_write > 0 {
                                    let stream = current_file_transfer.file_stream.as_mut().unwrap();
                                    let _ = stream.write_all(&current_file_transfer.buffer[0..bytes_to_write]);

                                    current_file_transfer.received += bytes_to_write;
                                    current_file_transfer.buffer.drain(0..bytes_to_write);
                                    made_progress = true;
                                }

                                if current_file_transfer.received >= total_size {
                                    let stream = current_file_transfer.file_stream.as_mut().unwrap();
                                    let _ = stream.flush();

                                    current_file_transfer.file_stream = None;
                                    current_file_transfer.received = 0;
                                    is_meta_data.store(true, Ordering::Release);
                                }
                            }
                        }

                        if !made_progress {
                            break;
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

fn get_meta_data(slice: &[u8]) -> Option<Metadata> {
    let size;
    let name;
    let permissions;

    match get_zero_byte_index(slice) {
        Some(zero_byte_index) => {
            let json = String::from_utf8_lossy(&slice[..zero_byte_index]);

            if let Ok(meta_data) = json::parse(&json)
            && let Some(name_val) = meta_data["name"].as_str()
            && let Some(size_val) = meta_data["size"].as_usize()
            && let Some(is_app_val) = meta_data["is_app"].as_bool() {
                size = size_val;
                name = name_val.to_string();

                if is_app_val {
                    permissions = Permissions::from_mode(0o777);
                } else {
                    permissions = Permissions::from_mode(0o666);
                }

                return Option::from(Metadata { size, name, permissions });
            }
        },
        _ => {}
    }

    None
}

fn get_zero_byte_index(slice: &[u8]) -> Option<usize> {
    slice.iter().position(|&b| b == 0)
}
