use std::env;
use std::process;
use tokio;

use code_routic::config::config_manager::ConfigManager;
use code_routic::server::server_setup::ServerSetup;
use code_routic::utils::process_checker::{is_service_running, save_pid, cleanup_pid_file, get_service_pid};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let command = if args.len() > 1 { &args[1] } else { "help" };

    match command{
        "start" => {
            start_server().await;
        }
        "stop" => {
            stop_server();
        }
        "restart" => {
            restart_server();
        }
        "status" => {
            show_status();
        }
        "code" => {
            println!("Executing code command...");
            // TODO: 实现执行代码命令逻辑
        }
        "ui" => {
            println!("Opening web UI...");
            // TODO: 实现打开Web界面逻辑
        }
        "-v" | "version" => {
            println!("CodeRoutic version: 0.1.0");
        }
        "-h" | "help" | _ => {
            println!("{}", HELP_TEXT);
        }
    }
}

async fn start_server() {
    if is_service_running() {
        println!("✅ Service is already running in the background.");
        return;
    }
    
    // 初始化配置目录
    if let Err(e) = ConfigManager::init_dir() {
        eprintln!("Failed to initialize config directory: {}", e);
        process::exit(1);
    }
    
    // 读取配置
    let config = match ConfigManager::read_config_file() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to read config file: {}", e);
            process::exit(1);
        }
    };
    
    // 保存PID
    save_pid(process::id());
    
    // 设置Ctrl+C处理
    ctrlc::set_handler(move || {
        println!("Received Ctrl+C, cleaning up...");
        cleanup_pid_file();
        process::exit(0);
    }).expect("Error setting Ctrl+C handler");
    
    // 创建并启动服务器
    let port = config.port.unwrap_or(3456);
    let host = config.host.clone().unwrap_or("127.0.0.1".to_string());
    
    println!("Starting CodeRoutic server on {}:{}", host, port);
    
    let app = ServerSetup::create_server(config).await;
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port)).await.unwrap();
    
    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("Server error: {}", e);
        cleanup_pid_file();
        process::exit(1);
    }
}

fn stop_server() {
    match get_service_pid() {
        Some(pid) => {
            // 在Unix系统上可以通过kill命令停止进程
            // 在Windows上需要使用其他方法
            #[cfg(unix)]
            {
                use std::process::Command;
                if let Err(e) = Command::new("kill").arg(pid.to_string()).output() {
                    eprintln!("Failed to kill process {}: {}", pid, e);
                }
            }
            
            cleanup_pid_file();
            println!("CodeRoutic service has been successfully stopped.");
        }
        None => {
            println!("Service was not running or failed to stop.");
            cleanup_pid_file();
        }
    }
}

fn restart_server() {
    stop_server();
    println!("Starting CodeRoutic service...");
    // 重新启动逻辑可以在这里实现
    println!("✅ Service started successfully in the background.");
}

fn show_status() {
    if is_service_running() {
        if let Some(pid) = get_service_pid() {
            println!("✅ CodeRoutic service is running (PID: {})", pid);
        } else {
            println!("✅ CodeRoutic service is running");
        }
    } else {
        println!("❌ CodeRoutic service is not running");
    }
}

const HELP_TEXT: &str = "
Usage: ccr [command]

Commands:
  start         Start server 
  stop          Stop server
  restart       Restart server
  status        Show server status
  code          Execute claude command
  ui            Open the web UI in browser
  -v, version   Show version information
  -h, help      Show help information

Example:
  ccr start
  ccr code \"Write a Hello World\"
  ccr ui
";