# CodeRoutic

CodeRoutic 是一个用 Rust 编写的 Claude Code Router，用于将 Claude Code 请求路由到不同的模型并自定义任何请求。

## 项目结构

```
src/
├── bin/                  # 可执行文件目录
├── config/               # 配置管理模块
│   ├── config_manager.rs # 配置管理器
│   └── types.rs          # 配置相关类型定义
├── core/                 # 核心模块
│   ├── constants.rs      # 常量定义
│   └── error.rs          # 错误处理
├── lib.rs                # 库模块声明
├── main.rs               # 主入口点
├── router/               # 路由模块
│   ├── route_handler.rs  # 路由处理器
│   └── route_logic.rs    # 路由逻辑
├── server/               # 服务器模块
│   ├── api_handlers.rs   # API处理函数
│   └── server_setup.rs   # 服务器设置
├── transformers/         # 转换器模块
│   ├── transformer_manager.rs # 转换器管理器
│   └── types.rs          # 转换器相关类型定义
└── utils/                # 工具模块
    ├── cache.rs          # 缓存工具
    ├── logger.rs         # 日志工具
    ├── process_checker.rs# 进程检查工具
    └── tokenizer.rs      # 分词器工具
```

## 模块说明

### config 模块
处理配置文件的读取、解析和管理。

### core 模块
包含项目的核心常量和错误处理机制。

### router 模块
实现请求路由逻辑，根据配置将请求分发到不同的模型提供商。

### server 模块
基于 axum 框架构建的 Web 服务器，处理 HTTP 请求。

### transformers 模块
实现请求和响应的转换逻辑，确保与不同模型提供商 API 的兼容性。

### utils 模块
提供各种工具函数，包括缓存、日志、进程检查和分词器等。

## 构建和运行

```bash
# 构建项目
cargo build

# 运行项目
cargo run
```