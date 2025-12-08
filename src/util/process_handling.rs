use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::{Error, Read, Write};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};
#[cfg(not(target_os = "windows"))]
use crate::util::constants::{PID_FILE_EXTENSION, UNIX_KILL_COMMAND, UNIX_SIGKILL_ARG, UNIX_SIGTERM_ARG};
#[cfg(target_os = "windows")]
use crate::util::constants::{PID_FILE_EXTENSION, WIN_KILL_COMMAND};

unsafe extern "C" {
    fn kill(pid: i32, sig: i32) -> i32;
}

pub fn start_new_process(program: impl AsRef<OsStr>, args: Vec<&str>, description: &str) {
    let program_path = Path::new(program.as_ref());

    if !program_path.exists() {
        println!("Executable '{}' does not exist", program.as_ref().to_string_lossy());
        return;
    }

    let mut command = Command::new(program);

    command.stdout(Stdio::null()).stdin(Stdio::null()).stderr(Stdio::null());
    command.args(args);

    match command.spawn() {
        Ok(child) => {
            create_pid_file(child.id(), description);
        }
        Err(e) => {
            println!("Error creating process: {}", e);
        }
    }
}

pub fn shutdown_process(description: &str) {
    let pid = read_pid_file(description);

    kill_process(pid, false);
    delete_pid_file(description);
}

pub fn is_process_running(description: &str) -> bool {
    let pid = read_pid_file(description);

    if pid == 0 {
        return false
    }

    unsafe {
        kill(pid as i32, 0) == 0
    }
}

pub fn execute(exe: &str, args: &[&str]) -> Error {
    Command::new(exe).args(args).exec()
}

/// Writes the given process id (pid) to a {description}.pid file in
/// the little endian (le) format
fn create_pid_file(pid: u32, description: &str) {
    let file_name = get_pid_file_name(description);
    let pid_file = File::create(file_name);

    if let Ok(mut pid_file) = pid_file {
        if let Ok(_) = pid_file.write_all(pid.to_le_bytes().as_ref()) {
            return;
        }
    }
}

fn delete_pid_file(description: &str) -> bool {
    let file_name = get_pid_file_name(description);

    fs::remove_file(file_name).is_ok()
}

/// Reads the first 4 bytes of the {description}.pid file in the little endian format.
fn read_pid_file(description: &str) -> u32 {
    let file_name = get_pid_file_name(description);

    if let Ok(mut pid_file) = File::open(file_name.clone()) {
        let mut buffer = [0u8; 4];

        match pid_file.read_exact(&mut buffer) {
            Ok(_) => {
                return u32::from_le_bytes(buffer);
            },
            Err(_) => {
                println!("Error reading pid file: {}", file_name);
            }
        }
    }

    0
}

fn get_pid_file_name(description: &str) -> String {
    format!("{}{}", description, PID_FILE_EXTENSION)
}

fn kill_process(pid: u32, force: bool) -> bool {
    let mut cmd;

    #[cfg(not(target_os = "windows"))] // Linux and macOS
    {
        cmd = Command::new(UNIX_KILL_COMMAND);

        if force {
            cmd.arg(UNIX_SIGKILL_ARG);
        } else {
            cmd.arg(UNIX_SIGTERM_ARG);
        }

        cmd.arg(pid.to_string());
    }

    #[cfg(target_os = "windows")]
    {
        cmd = Command::new(WIN_KILL_COMMAND);

        if force {
            cmd.arg("/F");
        }

        cmd.args(&["/PID", &pid.to_string(), "/T"]);
    }

    match cmd.status() {
        Ok(status) => {
            if status.success() {
                return true;
            }

            if !force {
                println!("Polite kill failed for PID {}, trying force kill...", pid);
                return kill_process(pid, true);
            }

            false
        }
        Err(e) => {
            eprintln!("Failed to execute kill command: {}", e);
            false
        },
    }
}
