use std::env;
use std::path::PathBuf;
use std::string::ToString;

pub static PROGRAM_NAME: &str = "upman";
pub static CONFIG_FILE_EXTENSION: &str = ".json";
/// File extension for files storing a process id
pub static PID_FILE_EXTENSION: &str = ".pid";
pub static SERVER_PROCESS_DESCRIPTION: &str = "server";
pub static UPDATES_FOLDER_NAME: &str = "updates";

// ----- Standard configurations -----

/// Standard Server Port
pub static STD_PORT: &str = "4455";
/// Loopback Address
pub static STD_ADDRESS: &str = "127.0.0.1";
/// Any IP
pub static STD_SERVER_ADDRESS: &str = "0.0.0.0";

// ----- Config keys -----

pub static CONFIG_PORT_KEY: &str = "port";
pub static CONFIG_ADDRESS_KEY: &str = "address";

// ----- Communication constants -----

pub static GREETING_MSG: &str = "ClientHello\n";
pub static FILE_NAME_KEY: &str = "name";
pub static FILE_SIZE_KEY: &str = "size";
pub static FILE_IS_EXECUTABLE_KEY: &str = "is_app";

// ----- Commands for killing processes -----

/// Unix command to shut down processes
pub static UNIX_KILL_COMMAND: &str = "kill";
/// Kill signal argument for the unix kill command
pub static UNIX_SIGKILL_ARG: &str = "-9";
/// Termination signal argument for the unix kill command
pub static UNIX_SIGTERM_ARG: &str = "-15";

/// Windows command to shut down processes
pub static WIN_KILL_COMMAND: &str = "taskkill";

// ----- General arguments -----

pub static ARG_HELP: &str = "help";

// ----- Server command line arguments -----
pub static ARG_START_SERVER_PROCESS: &str = "start";
pub static ARG_STOP_SERVER_PROCESS: &str = "stop";
pub static ARG_START_SERVER: &str = "start-server";

// ----- Error messages -----

// ----- Resource bundle identifiers -----
// RBC = Resource Bundle Constant

pub static RBC_UPDATES_FOLDER_EMPTY: &str = "UpdatesFolderEmpty";
pub static RBC_ADD_FILES_TO_UPDATES: &str = "AddFilesToUpdatesFolder";

// ----- Runtime Constants -----

pub fn get_exe_dir() -> PathBuf {
    let exe_dir = env::current_exe().unwrap();
    exe_dir.parent().unwrap().to_path_buf()
}