use axum::{
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::config::types::Config;

pub async fn claude_auth_with_state(
    State(state): State<Arc<RwLock<Config>>>,
    req: Request,
    next: Next,
) -> Response {
    // 从配置中读取API密钥
    let config = state.read().await;
    let expected_api_key = config.api_key.as_ref();
    
    match expected_api_key {
        Some(expected_key) => {
            // 如果配置了API密钥，则验证API密钥
            let auth_header = req.headers()
                .get("authorization")
                .or_else(|| req.headers().get("x-api-key"))
                .and_then(|header| header.to_str().ok());
            
            let auth_token = match auth_header {
                Some(header) => {
                    if header.starts_with("Bearer ") {
                        header[7..].trim().to_string()
                    } else {
                        header.trim().to_string()
                    }
                }
                None => {
                    return (
                        axum::http::StatusCode::UNAUTHORIZED,
                        "API key is required".to_string(),
                    ).into_response();
                }
            };
            
            if auth_token != *expected_key {
                return (
                    axum::http::StatusCode::UNAUTHORIZED,
                    "Invalid API key".to_string(),
                ).into_response();
            }
        }
        None => {
            // 如果没有配置API密钥，只允许localhost访问
            let host = req.headers()
                .get("host")
                .and_then(|header| header.to_str().ok())
                .unwrap_or("");
            
            let port = config.port.unwrap_or(3456);
            let allowed_hosts = [
                format!("127.0.0.1:{}", port),
                format!("localhost:{}", port),
                "127.0.0.1".to_string(),
                "localhost".to_string(),
            ];
            
            // 检查请求来源是否为localhost
            let is_local = allowed_hosts.iter().any(|allowed| host.starts_with(allowed));
            
            if !is_local {
                return (
                    axum::http::StatusCode::FORBIDDEN,
                    "Access denied: Only localhost access is allowed when no API key is configured".to_string(),
                ).into_response();
            }
        }
    }
    
    // 继续处理请求
    next.run(req).await
}