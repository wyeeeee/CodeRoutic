//! 路由器测试模块
//! 
//! 该模块包含针对路由逻辑的各种测试用例，主要测试模型选择逻辑，
//! 包括指定模型、默认模型、背景任务模型、思考模型、长上下文模型等场景。

#[cfg(test)]
mod router_tests {
    use code_routic::config::types::{Config, Provider, RouterConfig};
    use code_routic::router::route_logic::{RouteLogic, RouteRequest, RequestBody, SystemMessage, Tool};
    
    #[test]
    fn test_get_specified_provider_model() {
        let config = create_test_config();
        let req = RouteRequest {
            body: RequestBody {
                model: Some("openrouter,anthropic/claude-3-haiku".to_string()),
                system: None,
                thinking: None,
                tools: None,
                metadata: None,
            },
            session_id: None,
        };
        
        let result = RouteLogic::get_use_model(&req, 1000, &config, None);
        assert_eq!(result, "openrouter,anthropic/claude-3-haiku");
    }
    
    #[test]
    fn test_get_default_model() {
        let config = create_test_config();
        let req = RouteRequest {
            body: RequestBody {
                model: Some("claude-3-haiku".to_string()),
                system: None,
                thinking: None,
                tools: None,
                metadata: None,
            },
            session_id: None,
        };
        
        let result = RouteLogic::get_use_model(&req, 1000, &config, None);
        assert_eq!(result, "openrouter,anthropic/claude-sonnet-4");
    }
    
    #[test]
    fn test_get_background_model() {
        let mut config = create_test_config();
        config.router.background = Some("openrouter,anthropic/claude-3-opus".to_string());
        
        let req = RouteRequest {
            body: RequestBody {
                model: Some("claude-3-5-haiku-20241022".to_string()),
                system: None,
                thinking: None,
                tools: None,
                metadata: None,
            },
            session_id: None,
        };
        
        let result = RouteLogic::get_use_model(&req, 1000, &config, None);
        assert_eq!(result, "openrouter,anthropic/claude-3-opus");
    }
    
    #[test]
    fn test_get_think_model() {
        let mut config = create_test_config();
        config.router.think = Some("openrouter,anthropic/claude-3-sonnet".to_string());
        
        let req = RouteRequest {
            body: RequestBody {
                model: Some("claude-3-haiku".to_string()),
                system: None,
                thinking: Some(true),
                tools: None,
                metadata: None,
            },
            session_id: None,
        };
        
        let result = RouteLogic::get_use_model(&req, 1000, &config, None);
        assert_eq!(result, "openrouter,anthropic/claude-3-sonnet");
    }
    
    #[test]
    fn test_get_long_context_model_by_token_count() {
        let mut config = create_test_config();
        config.router.long_context = Some("openrouter,anthropic/claude-3-sonnet".to_string());
        config.router.long_context_threshold = Some(10000);
        
        let req = RouteRequest {
            body: RequestBody {
                model: Some("claude-3-haiku".to_string()),
                system: None,
                thinking: None,
                tools: None,
                metadata: None,
            },
            session_id: None,
        };
        
        let result = RouteLogic::get_use_model(&req, 15000, &config, None);
        assert_eq!(result, "openrouter,anthropic/claude-3-sonnet");
    }
    
    #[test]
    fn test_extract_subagent_model() {
        let system_message = vec![
            SystemMessage { text: Some("normal message".to_string()) },
            SystemMessage { 
                text: Some("<CCR-SUBAGENT-MODEL>openrouter,anthropic/claude-3-opus</CCR-SUBAGENT-MODEL> other content".to_string()) 
            },
        ];
        
        let req = RouteRequest {
            body: RequestBody {
                model: Some("claude-3-haiku".to_string()),
                system: Some(system_message),
                thinking: None,
                tools: None,
                metadata: None,
            },
            session_id: None,
        };
        
        let config = create_test_config();
        let result = RouteLogic::get_use_model(&req, 1000, &config, None);
        assert_eq!(result, "openrouter,anthropic/claude-3-opus");
    }
    
    #[test]
    fn test_get_web_search_model() {
        let mut config = create_test_config();
        config.router.web_search = Some("openrouter,anthropic/claude-3-sonnet".to_string());
        
        let tools = vec![
            Tool {
                tool_type: Some("web_search".to_string()),
            }
        ];
        
        let req = RouteRequest {
            body: RequestBody {
                model: Some("claude-3-haiku".to_string()),
                system: None,
                thinking: None,
                tools: Some(tools),
                metadata: None,
            },
            session_id: None,
        };
        
        let result = RouteLogic::get_use_model(&req, 1000, &config, None);
        assert_eq!(result, "openrouter,anthropic/claude-3-sonnet");
    }
    
    fn create_test_config() -> Config {
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
            providers: vec![
                Provider {
                    name: "openrouter".to_string(),
                    api_base_url: "https://openrouter.ai/api/v1".to_string(),
                    api_key: "test_key".to_string(),
                    models: vec![
                        "anthropic/claude-3-haiku".to_string(),
                        "anthropic/claude-3-sonnet".to_string(),
                        "anthropic/claude-3-opus".to_string(),
                    ],
                    transformer: None,
                }
            ],
            router: RouterConfig {
                default: "openrouter,anthropic/claude-sonnet-4".to_string(),
                background: None,
                think: None,
                long_context: None,
                long_context_threshold: Some(60000),
                web_search: None,
            },
            transformers: None,
            extra: std::collections::HashMap::new(),
        }
    }
}