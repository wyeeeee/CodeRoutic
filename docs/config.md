# 配置管理模块

配置管理模块负责处理 CodeRoutic 的配置文件，包括读取、写入、备份和环境变量插值等功能。

## 功能特性

### 1. 配置文件初始化
- 自动创建配置目录结构
- 创建必要的子目录（plugins、logs等）

### 2. 配置文件读取
- 读取 JSON 格式的配置文件
- 支持环境变量插值
- 配置文件不存在时引导用户进行初始设置

### 3. 配置文件写入
- 将配置数据以格式化的 JSON 形式写入文件
- 自动创建必要的目录结构

### 4. 配置文件备份
- 创建带时间戳的配置文件备份
- 自动清理旧备份，只保留最近的3个备份文件

### 5. 环境变量插值
- 支持 `$VAR_NAME` 和 `${VAR_NAME}` 格式的环境变量引用
- 递归处理配置对象中的所有字符串值
- 环境变量不存在时保持原始字符串

## API 说明

### ConfigManager

#### `init_dir() -> Result<()>`
初始化配置目录结构，创建必要的子目录。

#### `read_config_file() -> Result<Config>`
读取并解析配置文件，支持环境变量插值。

#### `write_config_file(config: &Config) -> Result<()>`
将配置写入文件。

#### `backup_config_file() -> Result<Option<String>>`
创建配置文件备份，返回备份文件路径。

## 数据结构

### Config
主要的配置数据结构，包含以下字段：
- `api_key`: API 密钥
- `proxy_url`: 代理 URL
- `log`: 是否启用日志
- `log_level`: 日志级别
- `host`: 服务器主机地址
- `port`: 服务器端口
- `providers`: 模型提供商配置列表
- `router`: 路由配置
- `transformers`: 转换器配置
- `extra`: 其他自定义配置

## 使用示例

```rust
use code_routic::config::config_manager::ConfigManager;
use code_routic::config::types::Config;

// 初始化配置目录
ConfigManager::init_dir()?;

// 读取配置
let config = ConfigManager::read_config_file()?;

// 写入配置
let new_config = Config::default();
ConfigManager::write_config_file(&new_config)?;

// 备份配置
let backup_path = ConfigManager::backup_config_file()?;
```

## 环境变量插值示例

配置文件中可以使用环境变量：

```json
{
  "api_key": "$OPENAI_API_KEY",
  "proxy_url": "${HTTP_PROXY}"
}
```

当环境变量 `OPENAI_API_KEY` 设置为 `sk-xxx` 时，配置会被插值为：

```json
{
  "api_key": "sk-xxx",
  "proxy_url": "${HTTP_PROXY}"
}
```

## 测试

模块包含完整的单元测试，可以通过以下命令运行：

```bash
cargo test config
```

测试覆盖了：
- 目录初始化
- 配置文件读写
- 环境变量插值
- 配置文件备份