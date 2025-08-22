use code_routic::config::types::{Config, Provider, RouterConfig, Transformer, ModelTransformer, TransformerConfig};
use std::collections::HashMap;

/// 配置测试模块
/// 包含对配置类型和配置管理器的序列化和默认值测试
#[cfg(test)]
mod config_types_tests {
    use super::*;

    /// 测试 Config 结构体的默认值实现
    /// 验证默认配置是否符合预期值：
    /// - 端口: 3456
    /// - 主机: "127.0.0.1"
    /// - 日志: true
    /// - 日志级别: "debug"
    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.port, Some(3456));
        assert_eq!(config.host, Some("127.0.0.1".to_string()));
        assert_eq!(config.log, Some(true));
        assert_eq!(config.log_level, Some("debug".to_string()));
    }

    /// 测试 Config 结构体的序列化和反序列化功能
    /// 构造一个完整的 Config 实例，将其序列化为 JSON 字符串，
    /// 然后从 JSON 字符串反序列化回 Config 实例，
    /// 验证序列化过程是否保持数据完整性
    #[test]
    fn test_config_serialization() {
        let config = Config {
            api_key: Some("test_key".to_string()),
            proxy_url: Some("http://proxy.test".to_string()),
            log: Some(true),
            log_level: Some("info".to_string()),
            host: Some("localhost".to_string()),
            port: Some(8080),
            non_interactive_mode: Some(false),
            api_timeout_ms: Some(30000),
            custom_router_path: Some("/custom/path".to_string()),
            providers: vec![Provider {
                name: "test_provider".to_string(),
                api_base_url: "http://api.test".to_string(),
                api_key: "provider_key".to_string(),
                models: vec!["test_model".to_string()],
                transformer: Some(Transformer {
                    use_transformers: vec![serde_json::Value::String("transformer1".to_string())],
                    model_specific: {
                        let mut map = HashMap::new();
                        map.insert("test_model".to_string(), ModelTransformer {
                            use_transformers: vec![serde_json::Value::String("model_transformer".to_string())],
                        });
                        map
                    },
                }),
            }],
            router: RouterConfig {
                default: "test_provider,test_model".to_string(),
                background: Some("background_provider".to_string()),
                think: Some("think_provider".to_string()),
                long_context: Some("long_context_provider".to_string()),
                long_context_threshold: Some(50000),
                web_search: Some("web_search_provider".to_string()),
            },
            transformers: Some(vec![TransformerConfig {
                path: "/path/to/transformer".to_string(),
                options: Some(serde_json::Value::String("transformer_options".to_string())),
            }]),
            extra: {
                let mut map = HashMap::new();
                map.insert("custom_field".to_string(), serde_json::Value::String("custom_value".to_string()));
                map
            },
        };

        let json = serde_json::to_string_pretty(&config).unwrap();
        let parsed_config: Config = serde_json::from_str(&json).unwrap();
        
        assert_eq!(config.api_key, parsed_config.api_key);
        assert_eq!(config.port, parsed_config.port);
        assert_eq!(config.host, parsed_config.host);
        assert_eq!(config.providers.len(), parsed_config.providers.len());
    }
}

/// 配置管理器测试模块
/// 测试配置管理器的序列化功能
#[cfg(test)]
mod config_manager_tests {
    use super::*;

    /// 测试配置管理器的序列化功能
    /// 使用默认配置实例，将其序列化为 JSON 字符串，
    /// 然后从 JSON 字符串反序列化回来，
    /// 验证基本配置字段是否保持一致
    #[test]
    fn test_config_manager_serialization() {
        let config = Config::default();
        let json = serde_json::to_string_pretty(&config).unwrap();
        let parsed_config: Config = serde_json::from_str(&json).unwrap();
        
        assert_eq!(config.port, parsed_config.port);
        assert_eq!(config.host, parsed_config.host);
        assert_eq!(config.log, parsed_config.log);
        assert_eq!(config.log_level, parsed_config.log_level);
    }
}