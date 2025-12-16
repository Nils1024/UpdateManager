use std::env;

pub fn get_locale() -> Option<String> {
    Some(env::var("LANG").unwrap())
}