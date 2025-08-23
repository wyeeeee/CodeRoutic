use crate::config::types::Config;
use crate::server::middleware::claude_auth;
use axum::{
    middleware,
    response::Response,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ServerSetup;

impl ServerSetup {
    pub async fn create_server(config: Config) -> Router {
        // 创建应用状态
        let app_state = Arc::new(RwLock::new(config));
        
        // 创建路由
        let app = Router::new()
            .route("/api/config", get(Self::get_config))
            .route("/api/config", post(Self::save_config))
            .route("/api/transformers", get(Self::get_transformers))
            .route("/api/restart", post(Self::restart_service))
            .route("/api/update/check", get(Self::check_update))
            .route("/api/update/perform", post(Self::perform_update))
            // Claude API endpoints - requires authentication
            .route("/v1/messages", post(Self::claude_messages))
            .route_layer(middleware::from_fn_with_state(
                app_state.clone(),
                claude_auth::claude_auth_with_state
            ))
            .with_state(app_state);
            
        app
    }
    
    async fn get_config() -> String {
        // TODO: 实现获取配置的逻辑
        "{}".to_string()
    }
    
    async fn save_config() -> String {
        // TODO: 实现保存配置的逻辑
        r#"{"success": true, "message": "Config saved successfully"}"#.to_string()
    }
    
    async fn get_transformers() -> String {
        // TODO: 实现获取转换器的逻辑
        r#"{"transformers": []}"#.to_string()
    }
    
    async fn restart_service() -> String {
        // TODO: 实现重启服务的逻辑
        r#"{"success": true, "message": "Service restart initiated"}"#.to_string()
    }
    
    async fn check_update() -> String {
        // TODO: 实现检查更新的逻辑
        r#"{"hasUpdate": false}"#.to_string()
    }
    
    async fn perform_update() -> String {
        // TODO: 实现执行更新的逻辑
        r#"{"success": true, "message": "Update performed successfully"}"#.to_string()
    }
    
    async fn claude_messages(
        _state: axum::extract::State<Arc<RwLock<Config>>>,
        _payload: axum::Json<serde_json::Value>,
    ) -> Response {
        // TODO: 实现Claude消息处理逻辑
        // 这里暂时返回一个简单的响应
        
        let response = serde_json::json!({
            "id": "msg_123456789",
            "type": "message",
            "role": "assistant",
            "content": [
                {
                    "type": "text",
                    "text": "Hello! I'm your AI assistant. This is a placeholder response from the CodeRoutic server."
                }
            ],
            "model": "claude-3-sonnet-20240229",
            "stop_reason": "end_turn",
            "stop_sequence": null,
            "usage": {
                "input_tokens": 25,
                "output_tokens": 18
            }
        });
        
        Response::builder()
            .status(axum::http::StatusCode::OK)
            .header("content-type", "application/json")
            .body(axum::body::Body::from(serde_json::to_string(&response).unwrap()))
            .unwrap()
    }
}