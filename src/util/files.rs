use std::io;
use std::fs::{self, DirEntry};
use std::path::Path;

/// Walks down the entire file tree and visits every file
pub fn walk_file_tree(dir: &Path, cb: &dyn Fn(&DirEntry)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                walk_file_tree(&path, cb)?;
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}

pub fn print_file_name(file: &DirEntry) {
    println!("Name: {}", file.file_name().to_string_lossy());
}