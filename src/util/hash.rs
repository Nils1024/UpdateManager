use std::cell::RefCell;
use std::fs::{self, DirEntry};
use std::path::Path;
use crate::util;
use crate::util::files::walk_file_tree;

/// Returns a single hash of all elements in the directory
pub fn get_dir_hash(dir: &Path) -> String {
    let hashes = RefCell::new(Vec::new());

    walk_file_tree(dir, &|entry| {
        if entry.file_name().to_str().unwrap() == util::config::get_config_name() {
            return;
        }
        
        hashes.borrow_mut().push(get_file_hash(entry));
    }).unwrap();

    let mut hashes = hashes.into_inner();
    hashes.sort();

    sha256::digest(hashes.join(""))
}

/// Returns the hash of the file by reading its content
fn get_file_hash(file: &DirEntry) -> String {
    sha256::digest(fs::read(file.path()).unwrap())
}