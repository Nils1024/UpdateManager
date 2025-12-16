use std::collections::HashMap;
use std::fs::{File, Permissions};
use std::io::BufWriter;
use std::os::unix::fs::PermissionsExt;
use json::{object, JsonValue};
use rand::Rng;
use crate::comm::conn::Conn;
use crate::util;

pub struct InitialMetadata {
    pub(crate) nonce: u32,
    pub(crate) hashes: HashMap<String, String>,
}

pub struct FileMetadata {
    pub(crate) size: usize,
    pub(crate) name: String,
    pub(crate) permissions: Permissions
}

pub struct FileTransfer {
    pub(crate) metadata: Option<FileMetadata>,
    pub(crate) received: usize,
    pub(crate) file_stream: Option<BufWriter<File>>,
    pub(crate) buffer: Vec<u8>,
}

pub fn get_file_meta_data(slice: &[u8]) -> Option<FileMetadata> {
    let size;
    let name;
    let permissions;

    match get_zero_byte_index(slice) {
        Some(zero_byte_index) => {
            let json = String::from_utf8_lossy(&slice[..zero_byte_index]);

            if let Ok(meta_data) = json::parse(&json)
                && let Some(name_val) = meta_data[util::constants::FILE_NAME_KEY].as_str()
                && let Some(size_val) = meta_data[util::constants::FILE_SIZE_KEY].as_usize()
                && let Some(is_app_val) = meta_data[util::constants::FILE_IS_EXECUTABLE_KEY].as_bool() {
                size = size_val;
                name = name_val.to_string();

                if is_app_val {
                    permissions = Permissions::from_mode(0o777);
                } else {
                    permissions = Permissions::from_mode(0o666);
                }

                return Option::from(FileMetadata { size, name, permissions });
            }
        },
        _ => {}
    }

    None
}
pub fn get_zero_byte_index(slice: &[u8]) -> Option<usize> {
    slice.iter().position(|&b| b == 0)
}

pub fn send_greeting_answer(conn: Conn, file_hashes: HashMap<String, String>) -> u32 {
    let mut rng = rand::rng();

    let mut updates_dir = util::constants::get_exe_dir();
    updates_dir.push(util::constants::UPDATES_FOLDER_NAME);

    let nonce = rng.random_range(1..255);

    let initial_meta_data = object! {
        "nonce": nonce,
        "files": file_hashes
    };

    let mut bytes = initial_meta_data.dump().into_bytes();
    bytes.push(0);

    conn.send_msg(bytes);

    nonce
}

pub fn get_initial_meta_data(slice: &[u8]) -> Option<InitialMetadata>{
    let zero_byte_index = get_zero_byte_index(slice)?;
    let json_str = String::from_utf8_lossy(&slice[..zero_byte_index]);

    let meta_data = json::parse(&json_str).ok()?;

    let nonce = meta_data["nonce"].as_u32()?;

    let hashes = match &meta_data["files"] {
        JsonValue::Object(obj) => {
            let mut map = HashMap::new();
            for (k, v) in obj.iter() {
                map.insert(k.to_string(), v.as_str()?.to_string());
            }
            map
        }
        _ => return None,
    };

    Some(InitialMetadata { nonce, hashes })
}

pub fn send_different_files(conn: Conn, files: Vec<String>) {
    let different_files = object! {
        "files": files
    };

    let mut bytes = different_files.dump().into_bytes();
    bytes.push(0);

    conn.send_msg(bytes);
}

pub fn get_different_files(slice: &[u8]) -> Option<Vec<String>> {
    let zero_byte_index = get_zero_byte_index(slice)?;
    let json_str = String::from_utf8_lossy(&slice[..zero_byte_index]);

    let meta = json::parse(&json_str).ok()?;

    match &meta["files"] {
        JsonValue::Array(arr) => {
            let mut result = Vec::with_capacity(arr.len());
            for v in arr {
                result.push(v.as_str()?.to_string());
            }
            Some(result)
        }
        _ => None,
    }
}