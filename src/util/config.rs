use std::{fs::{File}, path::Path};

use crate::util;

pub fn does_config_exists() -> bool {
    return Path::new(&get_config_name()).exists();
}

pub fn read_config(path: &Path) {

}

pub fn write_default_config(path: &Path) {
    let mut file = File::create(get_config_name());
}

fn get_config_name() -> String {
    return util::constants::PROGRAM_NAME.to_owned() + util::constants::CONFIG_FILE_EXTENSION;
}