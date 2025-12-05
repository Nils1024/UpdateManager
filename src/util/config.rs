use std::{fs, fs::{File}, io, path::Path};
use std::collections::HashMap;
use std::io::{Write};
use std::sync::OnceLock;
use json::{object, JsonValue};
use crate::util;

static CONFIG: OnceLock<HashMap<String, String>> = OnceLock::new();

pub fn get_config() -> &'static HashMap<String, String> {
    CONFIG.get_or_init(|| {
        let mut map = HashMap::new();

        init_config(&mut map);

        map
    })
}

fn init_config(config_map: &mut HashMap<String, String>) {
    let binding = get_config_name();
    let path = Path::new(&binding);

    if does_config_exists() {
        let config = read_config(path);

        for entry in config.entries() {
            config_map.insert(entry.0.to_string(), entry.1.to_string());
        }
    } else {
        match write_default_config(path) {
            Ok(_) => init_config(config_map),
            Err(_) => panic!("Unable to create default config file"),
        }
    }
}

pub fn does_config_exists() -> bool {
    Path::new(&get_config_name()).exists()
}

pub fn read_config(path: &Path) -> JsonValue{
    if let Ok(content) = fs::read_to_string(path) {
        json::parse(&content).unwrap()
    } else {
        eprintln!("Failed to read config file");
        JsonValue::Null
    }
}

pub fn write_default_config(path: &Path) -> io::Result<File> {
    let mut file = File::create(path)?;
    file.write_all(get_default_config().to_string().as_bytes())?;
    Ok(file)
}

fn get_default_config() -> JsonValue {
    let default_data = object! {
        util::constants::CONFIG_PORT_KEY => util::constants::STD_PORT,
        util::constants::CONFIG_ADDRESS_KEY => util::constants::STD_ADDRESS,
    };

    default_data
}

pub fn get_config_name() -> String {
    util::constants::PROGRAM_NAME.to_owned() + util::constants::CONFIG_FILE_EXTENSION
}