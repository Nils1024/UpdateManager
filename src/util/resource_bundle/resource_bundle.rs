use std::collections::HashMap;
use std::sync::OnceLock;
use crate::util;

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
        language_map.insert(util::constants::RBC_UPDATES_FOLDER_EMPTY, "Updates directory is empty.");
        language_map.insert(util::constants::RBC_ADD_FILES_TO_UPDATES, "Add the files, you want to distribute to: ");
    } else if language == "de_DE" {
        
    }
}

pub fn get_string(identifier: &str) -> &'static str {
    get_en_us().get(identifier).cloned().unwrap_or_default()
}