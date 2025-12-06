use std::{env, io};
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};

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

pub fn is_excluded(entry: &DirEntry) -> bool {
    if let Some(name) = entry.path().file_name().and_then(|s| s.to_str()) {
        if name == "upman.json" || name == ".DS_Store" {
            return true
        }
    }

    let exe_canon: PathBuf  = match env::current_exe().and_then(|p| fs::canonicalize(p)) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Unable to get current_exe: {}", e);
            return true
        }
    };

    let entry_canon = match fs::canonicalize(entry.path()) {
        Ok(c) => c,
        Err(_) => {
            return true;
        }
    };
    if entry_canon == exe_canon {
        return true
    }

    false
}