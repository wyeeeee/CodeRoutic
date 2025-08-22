use std::path::PathBuf;
use std::env;

pub const APP_NAME: &str = "code_routic";
pub const CONFIG_DIR_NAME: &str = ".code-routic";

pub fn get_home_dir() -> PathBuf {
    dirs::home_dir().expect("Failed to get home directory")
}

pub fn get_config_dir() -> PathBuf {
    get_home_dir().join(CONFIG_DIR_NAME)
}

pub fn get_config_file_path() -> PathBuf {
    get_config_dir().join("config.json")
}

pub fn get_plugins_dir() -> PathBuf {
    get_config_dir().join("plugins")
}

pub fn get_logs_dir() -> PathBuf {
    get_config_dir().join("logs")
}

pub fn get_pid_file_path() -> PathBuf {
    get_config_dir().join(".code-routic.pid")
}

pub fn get_reference_count_file_path() -> PathBuf {
    let temp_dir = env::temp_dir();
    temp_dir.join("code-routic-reference-count.txt")
}