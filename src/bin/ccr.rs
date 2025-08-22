use std::env;
use std::process;

mod config;
mod core;
mod router;
mod server;
mod transformers;
mod utils;

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = if args.len() > 1 { &args[1] } else { "help" };

    match command.as_str() {
        "start" => {
            println!("Starting CodeRoutic server...");
            // TODO: 实现启动服务器逻辑
        }
        "stop" => {
            println!("Stopping CodeRoutic server...");
            // TODO: 实现停止服务器逻辑
        }
        "restart" => {
            println!("Restarting CodeRoutic server...");
            // TODO: 实现重启服务器逻辑
        }
        "status" => {
            println!("Showing CodeRoutic server status...");
            // TODO: 实现显示服务器状态逻辑
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