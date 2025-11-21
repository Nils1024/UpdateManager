use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{ErrorKind, Write};
use std::process::{Child, Command, Stdio};

pub fn start_new_process(program: impl AsRef<OsStr>, description: &str) {
    let mut command = Command::new(program);

    command.stdout(Stdio::null()).stdin(Stdio::null()).stderr(Stdio::null());

    match command.spawn() {
        Ok(child) => {
            create_pid_file(child.id(), description);
        }
        Err(e) => {
            println!("Error creating process: {}", e);
        }
    }
}

fn create_pid_file(pid: u32, description: &str) {
    let file_name = format!("{}.pid", description);
    let pid_file = File::create(file_name);

    if let Ok(mut pid_file) = pid_file {
        if let Ok(file_write) = pid_file.write_all(pid.to_string().as_bytes()) {
            return;
        }
    }
}

fn shutdown_process() {

}

fn delete_pid_file() {

}


fn kill_process(pid: u32, force: bool) -> bool {
    let mut cmd;

    #[cfg(not(target_os = "windows"))] // Linux and macOS
    {
        cmd = Command::new("kill");

        if force {
            cmd.arg("-9");
        } else {
            cmd.arg("-15");
        }

        cmd.arg(pid.to_string());
    }

    #[cfg(target_os = "windows")]
    {
        cmd = Command::new("taskkill");

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
