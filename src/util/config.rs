use std::{fs, fs::{File}, io, path::Path};
use std::io::Write;
use crate::util;

pub fn does_config_exists() -> bool {
    return Path::new(&get_config_name()).exists();
}

pub fn read_config(path: &Path) {
    if let Ok(content) = fs::read_to_string(path) {
        println!("{:?}", json::parse(&content).unwrap());
    } else {
        eprintln!("Failed to read config file");
    }
}

pub fn write_default_config(path: &Path) -> io::Result<File> {
    let mut file = File::create(path)?;
    file.write_all(b"{}")?;
    Ok(file)
}

pub fn get_config_name() -> String {
    util::constants::PROGRAM_NAME.to_owned() + util::constants::CONFIG_FILE_EXTENSION
}