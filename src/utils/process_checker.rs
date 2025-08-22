use crate::core::constants::*;
use std::fs;
use std::process;

pub fn is_service_running() -> bool {
    if !get_pid_file_path().exists() {
        return false;
    }
    
    match fs::read_to_string(get_pid_file_path()) {
        Ok(pid_str) => {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                // 在Unix系统上，可以通过发送信号0来检查进程是否存在
                #[cfg(unix)]
                {
                    unsafe {
                        libc::kill(pid as i32, 0) == 0
                    }
                }
                // 在Windows上，暂时简化实现
                #[cfg(windows)]
                {
                    true
                }
            } else {
                cleanup_pid_file();
                false
            }
        }
        Err(_) => {
            cleanup_pid_file();
            false
        }
    }
}

pub fn save_pid(pid: u32) {
    if let Err(e) = fs::write(get_pid_file_path(), pid.to_string()) {
        eprintln!("Failed to save PID: {}", e);
    }
}

pub fn cleanup_pid_file() {
    if get_pid_file_path().exists() {
        if let Err(e) = fs::remove_file(get_pid_file_path()) {
            eprintln!("Failed to cleanup PID file: {}", e);
        }
    }
}

pub fn get_service_pid() -> Option<u32> {
    if !get_pid_file_path().exists() {
        return None;
    }
    
    match fs::read_to_string(get_pid_file_path()) {
        Ok(pid_str) => {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                Some(pid)
            } else {
                None
            }
        }
        Err(_) => None,
    }
}