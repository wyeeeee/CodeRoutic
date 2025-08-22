use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "APIKEY")]
    pub api_key: Option<String>,
    
    #[serde(rename = "PROXY_URL")]
    pub proxy_url: Option<String>,
    
    #[serde(rename = "LOG")]
    pub log: Option<bool>,
    
    #[serde(rename = "LOG_LEVEL")]
    pub log_level: Option<String>,
    
    #[serde(rename = "HOST")]
    pub host: Option<String>,
    
    #[serde(rename = "PORT")]
    pub port: Option<u16>,
    
    #[serde(rename = "NON_INTERACTIVE_MODE")]
    pub non_interactive_mode: Option<bool>,
    
    #[serde(rename = "API_TIMEOUT_MS")]
    pub api_timeout_ms: Option<u64>,
    
    #[serde(rename = "CUSTOM_ROUTER_PATH")]
    pub custom_router_path: Option<String>,
    
    #[serde(rename = "Providers")]
    pub providers: Vec<Provider>,
    
    #[serde(rename = "Router")]
    pub router: RouterConfig,
    
    #[serde(rename = "transformers")]
    pub transformers: Option<Vec<TransformerConfig>>,
    
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub name: String,
    #[serde(rename = "api_base_url")]
    pub api_base_url: String,
    #[serde(rename = "api_key")]
    pub api_key: String,
    pub models: Vec<String>,
    pub transformer: Option<Transformer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transformer {
    #[serde(rename = "use")]
    pub use_transformers: Vec<serde_json::Value>,
    #[serde(flatten)]
    pub model_specific: HashMap<String, ModelTransformer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelTransformer {
    #[serde(rename = "use")]
    pub use_transformers: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    pub default: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub think: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub long_context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub long_context_threshold: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_search: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformerConfig {
    pub path: String,
    pub options: Option<serde_json::Value>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            api_key: None,
            proxy_url: None,
            log: Some(true),
            log_level: Some("debug".to_string()),
            host: Some("127.0.0.1".to_string()),
            port: Some(3456),
            non_interactive_mode: Some(false),
            api_timeout_ms: Some(600000),
            custom_router_path: None,
            providers: vec![],
            router: RouterConfig {
                default: "openrouter,anthropic/claude-sonnet-4".to_string(),
                background: None,
                think: None,
                long_context: None,
                long_context_threshold: Some(60000),
                web_search: None,
            },
            transformers: None,
            extra: HashMap::new(),
        }
    }
}