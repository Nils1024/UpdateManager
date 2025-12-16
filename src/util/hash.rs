use std::cell::RefCell;
use std::fs::{self};
use std::path::{Path, PathBuf};
use crate::util;
use crate::util::files::walk_file_tree;

/// Returns a single hash of all elements in the directory
pub fn get_dir_hash(dir: &Path) -> String {
    let hashes = RefCell::new(Vec::new());

    walk_file_tree(dir, &|entry| {
        if util::files::is_excluded(entry) {
            return;
        }

        if entry.path().is_file() {
            hashes.borrow_mut().push(get_file_hash(&entry.path()));
        }
    }).unwrap();

    let mut hashes = hashes.into_inner();
    hashes.sort();

    sha256::digest(hashes.join(""))
}

/// Returns the hash of the file by reading its content
pub fn get_file_hash(file: &PathBuf) -> String {
    sha256::digest(fs::read(file).unwrap())
}