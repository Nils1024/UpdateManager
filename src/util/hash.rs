use std::cell::RefCell;
use std::fs::{self, DirEntry};
use std::path::Path;

use crate::util::files::walk_file_tree;

// Returns a single hash of all elements in the directory
pub fn get_dir_hash(dir: &Path) -> String {
    let hashes = RefCell::new(Vec::new());

    walk_file_tree(dir, &|entry| {
        hashes.borrow_mut().push(get_file_hash(entry));
    }).unwrap();

    let mut hashes = hashes.into_inner();
    hashes.sort();

    return sha256::digest(hashes.join(""));
}

fn get_file_hash(file: &DirEntry) -> String {
    return sha256::digest(fs::read(file.path()).unwrap());
}