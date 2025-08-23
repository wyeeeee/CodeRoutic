use crate::config::constants::*;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;

static LOG_ENABLED: Mutex<Option<bool>> = Mutex::new(None);
static LOG_LEVEL: Mutex<Option<String>> = Mutex::new(None);

pub fn configure_logging(log_enabled: Option<bool>, log_level: Option<String>) {
    let mut enabled = LOG_ENABLED.lock().unwrap();
    *enabled = log_enabled;
    
    let mut level = LOG_LEVEL.lock().unwrap();
    *level = log_level;
}

pub fn log(args: &[&str]) {
    let enabled = LOG_ENABLED.lock().unwrap();
    if let Some(false) = *enabled {
        return;
    }
    
    // 如果日志配置未设置，默认启用
    let is_enabled = enabled.unwrap_or(true);
    if !is_enabled {
        return;
    }
    
    let timestamp = chrono::Utc::now().to_rfc3339();
    let log_message = format!("[{}] {}\n", timestamp, args.join(" "));
    
    // 确保日志目录存在
    let config_dir = get_config_dir();
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir).unwrap_or_else(|_| {
            eprintln!("Failed to create config directory");
        });
    }
    
    // 追加到日志文件
    let log_file_path = config_dir.join("code-routic.log");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file_path)
        .unwrap_or_else(|_| {
            eprintln!("Failed to open log file");
            return std::fs::File::create(config_dir.join("code-routic.log")).unwrap();
        });
        
    if let Err(e) = file.write_all(log_message.as_bytes()) {
        eprintln!("Failed to write to log file: {}", e);
    }
}