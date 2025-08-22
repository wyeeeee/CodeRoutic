# 路由模块

路由模块负责处理 CodeRoutic 的请求路由逻辑，根据请求内容和配置决定使用哪个模型提供商和模型。

## 功能特性

### 1. 模型路由
- 根据请求内容动态选择合适的模型
- 支持长上下文模型自动切换
- 支持子代理模型指定
- 支持后台模型指定
- 支持思考模型指定
- 支持网络搜索模型指定

### 2. 插件化架构
- 路由检查逻辑模块化，每个检查逻辑独立成文件
- 支持轻松添加新的路由检查逻辑
- 路由检查顺序可配置

### 3. 默认路由
- 所有检查失败时自动回退到默认模型
- 支持指定提供商和模型的完整路径

## API 说明

### RouteHandler

#### `handle_route(req: &mut RouteRequest, config: &Config, session_usage_cache: &HashMap<String, Usage>) -> String`
处理路由请求，返回应该使用的模型字符串（格式：`提供商名称,模型名称`）。

### RouteLogic

#### `get_use_model(req: &RouteRequest, token_count: usize, config: &Config, last_usage: Option<&Usage>) -> String`
核心路由逻辑，按顺序检查各种路由条件并返回合适的模型。

## 插件系统

路由模块采用插件化设计，每个路由检查逻辑都是一个独立的插件：

### 长上下文检查插件
检查请求是否需要使用长上下文模型，基于token数量和配置阈值。

### 子代理模型检查插件
检查请求中是否指定了子代理模型，通过特定的系统消息标记。

### 后台模型检查插件
检查请求模型是否为特定模型（如 claude-3-5-haiku），并路由到后台模型。

### 思考模型检查插件
检查请求是否启用了思考模式，并路由到思考模型。

### 网络搜索模型检查插件
检查请求是否需要使用网络搜索工具，并路由到网络搜索模型。

## 使用示例

```rust
use code_routic::router::route_handler::RouteHandler;
use code_routic::router::route_logic::{RouteRequest, RequestBody};

// 创建路由请求
let mut req = RouteRequest {
    body: RequestBody {
        model: Some("claude-3-5-sonnet".to_string()),
        system: None,
        thinking: None,
        tools: None,
        metadata: None,
    },
    session_id: None,
};

// 处理路由
let model = RouteHandler::handle_route(&mut req, &config, &session_usage_cache);
```

## 添加新的路由检查逻辑

1. 在 `src/router/plugin/` 目录下创建新的检查逻辑文件
2. 实现检查函数，签名格式为：
   ```rust
   pub fn check_xxx(
       req: &RouteRequest,
       token_count: usize,
       config: &Config,
       last_usage: Option<&Usage>,
   ) -> Option<String>
   ```
3. 在 `src/router/plugin/mod.rs` 中导出新函数
4. 在 `src/router/route_logic.rs` 的检查函数列表中添加新函数

## 测试

模块包含完整的单元测试，可以通过以下命令运行：

```bash
cargo test router
```

测试覆盖了：
- 各种路由检查逻辑
- 默认路由回退
- 插件系统的正确性