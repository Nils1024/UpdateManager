#[cfg(test)]
mod tests {
    use update_manager::util;

    #[test]
    fn test_string_building() {
        assert_eq!(util::constants::PROGRAM_NAME.to_owned() + util::constants::CONFIG_FILE_EXTENSION, "upman.json");
    }
}