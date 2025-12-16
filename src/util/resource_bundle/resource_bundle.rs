use std::collections::HashMap;
use std::sync::OnceLock;
use crate::util;
use crate::util::resource_bundle::locale::get_locale;

static EN_US: OnceLock<HashMap<&str, &str>> = OnceLock::new();
static DE_DE: OnceLock<HashMap<&str, &str>> = OnceLock::new();

pub fn get_en_us() -> &'static HashMap<&'static str, &'static str> {
    EN_US.get_or_init(|| {
        let mut map = HashMap::new();

        init_language_map(&mut map, "en_US");

        map
    })
}

pub fn get_de_de() -> &'static HashMap<&'static str, &'static str> {
    DE_DE.get_or_init(|| {
        let mut map = HashMap::new();

        init_language_map(&mut map, "de_DE");

        map
    })
}

fn init_language_map(language_map: &mut HashMap<&str, &str>, language: &str) {
    if language == "en_US" {
        language_map.insert(util::constants::RBC_UPDATES_CREATED, "Created the updates folder under: ");
        language_map.insert(util::constants::RBC_UPDATES_CREATED_ADD_FILES, "Put your files in there and restart the app.");
        language_map.insert(util::constants::RBC_UPDATES_FOLDER_EMPTY, "Updates directory is empty.");
        language_map.insert(util::constants::RBC_ADD_FILES_TO_UPDATES, "Add the files, you want to distribute to: ");
        language_map.insert(util::constants::RBC_CONFIG_CREATED, "Config file created.");
        language_map.insert(util::constants::RBC_CONTINUE, "Would you like to continue? [y/n]");
        language_map.insert(util::constants::RBC_CONNECTION_FAILED, "Failed to connect to server.");
        language_map.insert(util::constants::RBC_CONNECTION_FAILED_SOLUTION, "Make sure the server is running or you configured the correct address in the upman.json");
    } else if language == "de_DE" {
        
    }
}

pub fn get_string(identifier: &str) -> &'static str {
    if get_locale().unwrap() == "de_DE" {
        get_de_de().get(identifier).cloned().unwrap_or_default()
    } else {
        get_en_us().get(identifier).cloned().unwrap_or_default()
    }
}