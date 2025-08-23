use crate::config::types::Config;
use crate::config::constants::*;
use anyhow::{Context, Result};
use serde_json;
use std::fs;
use std::io::{self, Write};
use std::env;
use chrono::Utc;

pub struct ConfigManager;

impl ConfigManager {
    pub fn init_dir() -> Result<()> {
        let config_dir = get_config_dir();
        let plugins_dir = get_plugins_dir();
        let logs_dir = get_logs_dir();
        
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .with_context(|| format!("Failed to create config directory: {:?}", config_dir))?;
        }
        
        if !plugins_dir.exists() {
            fs::create_dir_all(&plugins_dir)
                .with_context(|| format!("Failed to create plugins directory: {:?}", plugins_dir))?;
        }
        
        if !logs_dir.exists() {
            fs::create_dir_all(&logs_dir)
                .with_context(|| format!("Failed to create logs directory: {:?}", logs_dir))?;
        }
        
        Ok(())
    }
    
    pub fn read_config_file() -> Result<Config> {
        let config_file_path = get_config_file_path();
        
        if config_file_path.exists() {
            let config_content = fs::read_to_string(&config_file_path)
                .with_context(|| format!("Failed to read config file: {:?}", config_file_path))?;
            
            let config: Config = serde_json::from_str(&config_content)
                .with_context(|| format!("Failed to parse config file: {:?}", config_file_path))?;
            
            let interpolated_config = Self::interpolate_env_vars(config)
                .with_context(|| "Failed to interpolate environment variables")?;
            
            Ok(interpolated_config)
        } else {
            // 如果配置文件不存在，提示用户进行初始设置
            Self::prompt_initial_setup()
        }
    }
    
    pub fn write_config_file(config: &Config) -> Result<()> {
        Self::init_dir()?;
        let config_file_path = get_config_file_path();
        let config_json = serde_json::to_string_pretty(config)?;
        fs::write(&config_file_path, config_json)
            .with_context(|| format!("Failed to write config file: {:?}", config_file_path))?;
        Ok(())
    }
    
    pub fn backup_config_file() -> Result<Option<String>> {
        let config_file_path = get_config_file_path();
        
        if config_file_path.exists() {
            // 生成备份文件名，使用时间戳
            let timestamp = Utc::now().format("%Y-%m-%dT%H-%M-%S%.3fZ").to_string();
            let backup_path = format!("{}.{}.bak", config_file_path.to_string_lossy(), timestamp);
            
            // 复制配置文件到备份位置
            fs::copy(&config_file_path, &backup_path)
                .with_context(|| format!("Failed to backup config file to: {}", backup_path))?;
            
            // 清理旧的备份文件，只保留最近的3个
            Self::cleanup_old_backups()
                .with_context(|| "Failed to cleanup old backups")?;
            
            Ok(Some(backup_path))
        } else {
            Ok(None)
        }
    }
    
    fn cleanup_old_backups() -> Result<()> {
        let config_file_path = get_config_file_path();
        let config_dir = config_file_path.parent()
            .with_context(|| "Failed to get config directory")?;
        let config_file_name = config_file_path.file_name()
            .with_context(|| "Failed to get config file name")?;
        
        // 读取目录中的所有文件
        let entries = fs::read_dir(config_dir)
            .with_context(|| format!("Failed to read config directory: {:?}", config_dir))?;
        
        // 收集所有备份文件
        let mut backup_files: Vec<_> = entries
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                if let Ok(file_name) = entry.file_name().into_string() {
                    file_name.starts_with(config_file_name.to_str().unwrap_or("")) 
                        && file_name.ends_with(".bak")
                } else {
                    false
                }
            })
            .collect();
        
        // 如果备份文件数量超过3个，则删除最旧的
        if backup_files.len() > 3 {
            // 按修改时间排序，最新的在前
            backup_files.sort_by(|a, b| {
                let a_time = a.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                let b_time = b.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                b_time.cmp(&a_time)
            });
            
            // 删除超过3个的旧备份
            for old_backup in backup_files.iter().skip(3) {
                if let Err(e) = fs::remove_file(old_backup.path()) {
                    eprintln!("Warning: Failed to remove old backup file {:?}: {}", old_backup.path(), e);
                }
            }
        }
        
        Ok(())
    }
    
    fn prompt_initial_setup() -> Result<Config> {
        println!("Config file not found. Please enter initial configuration:");
        
        print!("Enter Provider Name: ");
        io::stdout().flush().context("Failed to flush stdout")?;
        let mut name = String::new();
        io::stdin().read_line(&mut name).context("Failed to read provider name")?;
        let name = name.trim().to_string();
        
        if name.is_empty() {
            return Err(anyhow::anyhow!("Provider name cannot be empty"));
        }
        
        print!("Enter Provider API KEY: ");
        io::stdout().flush().context("Failed to flush stdout")?;
        let mut api_key = String::new();
        io::stdin().read_line(&mut api_key).context("Failed to read API key")?;
        let api_key = api_key.trim().to_string();
        
        print!("Enter Provider URL: ");
        io::stdout().flush().context("Failed to flush stdout")?;
        let mut base_url = String::new();
        io::stdin().read_line(&mut base_url).context("Failed to read provider URL")?;
        let base_url = base_url.trim().to_string();
        
        if base_url.is_empty() {
            return Err(anyhow::anyhow!("Provider URL cannot be empty"));
        }
        
        print!("Enter MODEL Name: ");
        io::stdout().flush().context("Failed to flush stdout")?;
        let mut model = String::new();
        io::stdin().read_line(&mut model).context("Failed to read model name")?;
        let model = model.trim().to_string();
        
        if model.is_empty() {
            return Err(anyhow::anyhow!("Model name cannot be empty"));
        }
        
        let config = Config {
            api_key: None,
            proxy_url: None,
            log: Some(true),
            log_level: Some("debug".to_string()),
            host: Some("127.0.0.1".to_string()),
            port: Some(3456),
            non_interactive_mode: Some(false),
            api_timeout_ms: Some(600000),
            custom_router_path: None,
            providers: vec![crate::config::types::Provider {
                name: name.clone(),
                api_base_url: base_url,
                api_key: api_key,
                models: vec![model.clone()],
                transformer: None,
            }],
            router: crate::config::types::RouterConfig {
                default: format!("{},{}", name, model),
                background: None,
                think: None,
                long_context: None,
                long_context_threshold: Some(60000),
                web_search: None,
            },
            transformers: None,
            extra: std::collections::HashMap::new(),
        };
        
        Self::write_config_file(&config)?;
        Ok(config)
    }
    
    fn interpolate_env_vars(config: Config) -> Result<Config> {
        // 将Config转换为serde_json::Value以便递归处理
        let config_value = serde_json::to_value(config)
            .context("Failed to convert config to JSON value")?;
        let interpolated_value = Self::interpolate_env_vars_value(config_value)
            .context("Failed to interpolate environment variables in config")?;
        let interpolated_config = serde_json::from_value(interpolated_value)
            .context("Failed to convert interpolated JSON value back to config")?;
        Ok(interpolated_config)
    }
    
    fn interpolate_env_vars_value(value: serde_json::Value) -> Result<serde_json::Value> {
        match value {
            serde_json::Value::String(s) => {
                // 替换 $VAR_NAME 或 ${VAR_NAME} 格式的环境变量
                let interpolated = Self::interpolate_string(&s);
                Ok(serde_json::Value::String(interpolated))
            }
            serde_json::Value::Array(arr) => {
                let interpolated_arr: Result<Vec<_>, _> = arr
                    .into_iter()
                    .map(Self::interpolate_env_vars_value)
                    .collect();
                interpolated_arr.map(serde_json::Value::Array)
                    .context("Failed to interpolate environment variables in array")
            }
            serde_json::Value::Object(obj) => {
                let interpolated_obj: Result<serde_json::Map<_, _>, _> = obj
                    .into_iter()
                    .map(|(key, val)| {
                        Self::interpolate_env_vars_value(val)
                            .map(|interpolated_val| (key, interpolated_val))
                    })
                    .collect();
                interpolated_obj.map(serde_json::Value::Object)
                    .context("Failed to interpolate environment variables in object")
            }
            _ => Ok(value),
        }
    }
    
    fn interpolate_string(s: &str) -> String {
        // 使用正则表达式匹配 $VAR_NAME 或 ${VAR_NAME} 格式
        let re = regex::Regex::new(r"\$\{([^}]+)\}|\$([A-Z_][A-Z0-9_]*)")
            .expect("Failed to compile regex for environment variable interpolation");
        re.replace_all(s, |caps: &regex::Captures| {
            let var_name_braced = caps.get(1).map_or("", |m| m.as_str()); // ${VAR_NAME} 格式
            let var_name_unbraced = caps.get(2).map_or("", |m| m.as_str()); // $VAR_NAME 格式
            
            // 确定实际的变量名和格式类型
            let (var_name, is_braced) = if !var_name_braced.is_empty() {
                (var_name_braced, true)
            } else {
                (var_name_unbraced, false)
            };
            
            env::var(var_name).unwrap_or_else(|_| {
                // 如果环境变量不存在，保持原始字符串
                if is_braced {
                    format!("${{{}}}", var_name)
                } else {
                    format!("${}", var_name)
                }
            })
        })
        .to_string()
    }
}