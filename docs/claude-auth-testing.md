# Claude 认证中间件测试说明

### 1. 有 API 密钥配置时
- 需要在请求头中提供有效的 API 密钥
- 支持两种认证方式：
  - `Authorization: Bearer <api_key>`
  - `x-api-key: <api_key>`
- 如果没有提供 API 密钥或密钥无效，返回 401 Unauthorized

### 2. 没有 API 密钥配置时
- 只允许来自 localhost 的请求
- 允许的主机格式：
  - `127.0.0.1:<port>`
  - `localhost:<port>`
  - `127.0.0.1`
  - `localhost`
- 如果请求来自非 localhost，返回 403 Forbidden

## 测试场景

### 场景 1: 有 API 密钥配置
```json
{
  "APIKEY": "my-secret-key"
}
```

**有效请求:**
```bash
curl -X POST http://localhost:3456/v1/messages \
  -H "Authorization: Bearer my-secret-key" \
  -H "Content-Type: application/json" \
  -d '{"model": "claude-3-sonnet", "messages": [{"role": "user", "content": "Hello"}]}'
```

**无效请求 (无密钥):**
```bash
curl -X POST http://localhost:3456/v1/messages \
  -H "Content-Type: application/json" \
  -d '{"model": "claude-3-sonnet", "messages": [{"role": "user", "content": "Hello"}]}'
# 返回 401 Unauthorized
```

**无效请求 (错误密钥):**
```bash
curl -X POST http://localhost:3456/v1/messages \
  -H "Authorization: Bearer wrong-key" \
  -H "Content-Type: application/json" \
  -d '{"model": "claude-3-sonnet", "messages": [{"role": "user", "content": "Hello"}]}'
# 返回 401 Unauthorized
```

### 场景 2: 没有 API 密钥配置
```json
{
  "APIKEY": null
}
```

**有效请求 (localhost):**
```bash
curl -X POST http://localhost:3456/v1/messages \
  -H "Content-Type: application/json" \
  -d '{"model": "claude-3-sonnet", "messages": [{"role": "user", "content": "Hello"}]}'
# 允许访问
```

**无效请求 (远程访问):**
```bash
curl -X POST http://192.168.1.100:3456/v1/messages \
  -H "Content-Type: application/json" \
  -d '{"model": "claude-3-sonnet", "messages": [{"role": "user", "content": "Hello"}]}'
# 返回 403 Forbidden
```

## 安全特性

1. **开发友好**: 未配置 API 密钥时允许本地开发
2. **生产安全**: 配置 API 密钥后强制验证
3. **灵活认证**: 支持多种认证头格式
4. **清晰错误**: 提供明确的错误消息

## 实现细节

中间件逻辑：
1. 检查配置中是否有 API 密钥
2. 如果有 API 密钥：
   - 验证请求中的 API 密钥
   - 无效则拒绝访问
3. 如果没有 API 密钥：
   - 检查请求来源是否为 localhost
   - 非本地请求则拒绝访问
4. 验证通过则继续处理请求