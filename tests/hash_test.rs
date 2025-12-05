use tempfile::tempdir;
use std::fs;
use update_manager::util::hash::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_dir_hash() {
        let temp_dir = tempdir().unwrap();
        let dir_path = temp_dir.path();

        fs::write(dir_path.join("a.txt"), b"Content A").unwrap();
        fs::write(dir_path.join("b.txt"), b"Content B").unwrap();
        fs::write(dir_path.join("c.txt"), b"Content C").unwrap();

        // Content A = 02f67ccd1094983cb438874466ce795ddf13ec4989dbd10eebfcf3ab2c8c04ca
        // Content B = 4b73e0d3a959744c282112e98bba96ac6da2fbc10ebfa6043e7f04eaa6058b07
        // Content C = c6e39a777a1da3a1af88b5024be9c249b535f9190052a356b862772705171b49
        // Combined (Sorted) = 02f67ccd1094983cb438874466ce795ddf13ec4989dbd10eebfcf3ab2c8c04ca4b73e0d3a959744c282112e98bba96ac6da2fbc10ebfa6043e7f04eaa6058b07c6e39a777a1da3a1af88b5024be9c249b535f9190052a356b862772705171b49
        // Combined Hash = 7de444e56ba5513b607441b7ec938d4023b215f51a18d33ccbe63623be79d203

        let expected_combined_hash = "7de444e56ba5513b607441b7ec938d4023b215f51a18d33ccbe63623be79d203";

        let actual_combined_hash = get_dir_hash(dir_path);

        assert_eq!(actual_combined_hash, expected_combined_hash);
    }
}