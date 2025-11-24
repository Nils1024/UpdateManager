pub static PROGRAM_NAME: &str = "upman";
pub static CONFIG_FILE_EXTENSION: &str = ".json";
pub static PID_FILE_EXTENSION: &str = ".pid";

pub static UNIX_KILL_COMMAND: &str = "kill";
/// Kill signal
pub static UNIX_SIGKILL_ARG: &str = "-9";
/// Termination signal
pub static UNIX_SIGTERM_ARG: &str = "-15";

pub static WIN_KILL_COMMAND: &str = "taskkill";