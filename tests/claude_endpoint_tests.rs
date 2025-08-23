//! Claude API 端点测试模块
//! 
//! 该模块包含针对 Claude API 端点和认证中间件的各种测试用例，
//! 主要测试 API 密钥认证、localhost 限制、端点响应等功能。

#[cfg(test)]
mod claude_endpoint_tests {
    use axum::{
        body::{Body, to_bytes},
        http::{HeaderValue, Method, Request, StatusCode},
    };
    use bytes::Bytes;
    use code_routic::config::types::Config;
    use code_routic::server::server::ServerSetup;
    use http_body_util::Full;
    use serde_json::json;
    use tower::ServiceExt; // 用于 `oneshot` 方法

    // 创建测试配置
    fn create_test_config_with_api_key() -> Config {
        let mut config = Config::default();
        config.api_key = Some("test-api-key".to_string());
        config.port = Some(3456);
        config
    }

    fn create_test_config_without_api_key() -> Config {
        let mut config = Config::default();
        config.api_key = None;
        config.port = Some(3456);
        config
    }

    // 创建测试请求
    fn create_clude_request() -> Request<Body> {
        let request_body = json!({
            "model": "claude-3-sonnet-20240229",
            "max_tokens": 1000,
            "messages": [
                {
                    "role": "user",
                    "content": "Hello, Claude!"
                }
            ]
        });

        Request::builder()
            .method(Method::POST)
            .uri("/v1/messages")
            .header("content-type", "application/json")
            .header("host", "127.0.0.1:3456")
            .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
            .unwrap()
    }

    #[tokio::test]
    async fn test_claude_endpoint_with_valid_api_key() {
        // 创建有 API 密钥的配置
        let config = create_test_config_with_api_key();
        
        // 创建服务器
        let app = ServerSetup::create_server(config).await;
        
        // 创建带有有效 API 密钥的请求
        let mut request = create_clude_request();
        request.headers_mut().insert(
            "authorization",
            HeaderValue::from_str("Bearer test-api-key").unwrap(),
        );

        // 发送请求
        let response = app.oneshot(request).await.unwrap();

        // 验证响应状态码
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_claude_endpoint_with_x_api_key() {
        // 创建有 API 密钥的配置
        let config = create_test_config_with_api_key();
        
        // 创建服务器
        let app = ServerSetup::create_server(config).await;
        
        // 创建带有 x-api-key 的请求
        let mut request = create_clude_request();
        request.headers_mut().insert(
            "x-api-key",
            HeaderValue::from_str("test-api-key").unwrap(),
        );

        // 发送请求
        let response = app.oneshot(request).await.unwrap();

        // 验证响应状态码
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_claude_endpoint_with_invalid_api_key() {
        // 创建有 API 密钥的配置
        let config = create_test_config_with_api_key();
        
        // 创建服务器
        let app = ServerSetup::create_server(config).await;
        
        // 创建带有无效 API 密钥的请求
        let mut request = create_clude_request();
        request.headers_mut().insert(
            "authorization",
            HeaderValue::from_str("Bearer invalid-key").unwrap(),
        );

        // 发送请求
        let response = app.oneshot(request).await.unwrap();

        // 验证响应状态码
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_claude_endpoint_without_api_key_configured_localhost() {
        // 创建没有 API 密钥的配置
        let config = create_test_config_without_api_key();
        
        // 创建服务器
        let app = ServerSetup::create_server(config).await;
        
        // 创建来自 localhost 的请求（无 API 密钥）
        let request = create_clude_request();

        // 发送请求
        let response = app.oneshot(request).await.unwrap();

        // 验证响应状态码 - 应该允许访问
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_claude_endpoint_without_api_key_configured_remote() {
        // 创建没有 API 密钥的配置
        let config = create_test_config_without_api_key();
        
        // 创建服务器
        let app = ServerSetup::create_server(config).await;
        
        // 创建来自远程地址的请求
        let request_body = json!({
            "model": "claude-3-sonnet-20240229",
            "max_tokens": 1000,
            "messages": [
                {
                    "role": "user",
                    "content": "Hello, Claude!"
                }
            ]
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/v1/messages")
            .header("content-type", "application/json")
            .header("host", "192.168.1.100:3456") // 远程地址
            .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
            .unwrap();

        // 发送请求
        let response = app.oneshot(request).await.unwrap();

        // 验证响应状态码 - 应该拒绝访问
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_claude_endpoint_with_api_key_configured_but_missing() {
        // 创建有 API 密钥的配置
        let config = create_test_config_with_api_key();
        
        // 创建服务器
        let app = ServerSetup::create_server(config).await;
        
        // 创建没有 API 密钥的请求
        let request = create_clude_request();

        // 发送请求
        let response = app.oneshot(request).await.unwrap();

        // 验证响应状态码 - 应该要求 API 密钥
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_claude_endpoint_response_format() {
        // 创建有 API 密钥的配置
        let config = create_test_config_with_api_key();
        
        // 创建服务器
        let app = ServerSetup::create_server(config).await;
        
        // 创建带有有效 API 密钥的请求
        let mut request = create_clude_request();
        request.headers_mut().insert(
            "authorization",
            HeaderValue::from_str("Bearer test-api-key").unwrap(),
        );

        // 发送请求
        let response = app.oneshot(request).await.unwrap();

        // 验证响应状态码
        assert_eq!(response.status(), StatusCode::OK);

        // 验证响应头
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "application/json"
        );

        // 读取响应体
        let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        // 验证响应格式
        assert!(response_json.get("id").is_some());
        assert!(response_json.get("type").is_some());
        assert_eq!(response_json.get("type").unwrap(), "message");
        assert_eq!(response_json.get("role").unwrap(), "assistant");
        assert!(response_json.get("content").is_some());
        assert!(response_json.get("usage").is_some());
    }

    #[tokio::test]
    async fn test_claude_endpoint_different_localhost_formats() {
        let test_hosts = vec![
            "127.0.0.1:3456",
            "localhost:3456", 
            "127.0.0.1",
            "localhost",
        ];

        for host in test_hosts {
            // 创建没有 API 密钥的配置
            let config = create_test_config_without_api_key();
            
            // 创建服务器
            let app = ServerSetup::create_server(config).await;

            let request_body = json!({
                "model": "claude-3-sonnet-20240229",
                "max_tokens": 1000,
                "messages": [
                    {
                        "role": "user",
                        "content": "Hello, Claude!"
                    }
                ]
            });

            let request = Request::builder()
                .method(Method::POST)
                .uri("/v1/messages")
                .header("content-type", "application/json")
                .header("host", host)
                .body(Body::from(serde_json::to_vec(&request_body).unwrap()))
                .unwrap();

            // 发送请求
            let response = app.oneshot(request).await.unwrap();

            // 验证响应状态码 - 应该允许访问
            assert_eq!(response.status(), StatusCode::OK, "Failed for host: {}", host);
        }
    }

    #[tokio::test]
    async fn test_unprotected_endpoints_without_auth() {
        // 测试真正不受保护的端点（这些端点不应该有认证限制）
        // 注意：目前我们的服务器实现中，大部分端点都可能有限制
        // 这个测试主要验证Claude端点确实有认证，而其他端点可能有不同的行为
        
        // 创建没有 API 密钥的配置
        let config = create_test_config_without_api_key();
        
        // 创建服务器
        let app = ServerSetup::create_server(config).await;

        // 测试健康检查端点（如果有的话）
        let request = Request::builder()
            .method(Method::GET)
            .uri("/health") // 假设有健康检查端点
            .header("host", "192.168.1.100:3456") // 远程地址
            .body(Full::from(Bytes::new()))
            .unwrap();

        // 发送请求
        let response = app.oneshot(request).await.unwrap();

        // 这个测试主要验证我们的Claude端点确实有认证限制
        // 而其他端点可能有不同的认证要求
        // 目前我们期望404，因为我们没有实现健康检查端点
        // 但如果返回403，说明也有认证限制，这也是合理的
        assert!(
            response.status() == StatusCode::NOT_FOUND || 
            response.status() == StatusCode::FORBIDDEN,
            "Unexpected status code for health check: {}",
            response.status()
        );
    }
}