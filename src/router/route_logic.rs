use crate::config::types::Config;
use crate::router::plugin::{
    check_long_context, check_subagent_model, check_background_model,
    check_think_model, check_web_search
};

pub struct RouteLogic;

impl RouteLogic {
    pub fn get_use_model(
        req: &RouteRequest,
        token_count: usize,
        config: &Config,
        last_usage: Option<&Usage>,
    ) -> String {
        // 定义路由检查函数列表
        let checkers: Vec<fn(&RouteRequest, usize, &Config, Option<&Usage>) -> Option<String>> = vec![
            check_long_context,
            check_subagent_model,
            check_background_model,
            check_think_model,
            check_web_search,
        ];
        
        // 按顺序执行检查函数
        for checker in checkers {
            if let Some(model) = checker(req, token_count, config, last_usage) {
                return model;
            }
        }
        
        // 如果所有检查都失败，返回默认模型
        config.router.default.clone()
    }
}

#[derive(Debug, Clone)]
pub struct RouteRequest {
    pub body: RequestBody,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RequestBody {
    pub model: Option<String>,
    pub system: Option<Vec<SystemMessage>>,
    pub thinking: Option<bool>,
    pub tools: Option<Vec<Tool>>,
    pub metadata: Option<Metadata>,
}

#[derive(Debug, Clone)]
pub struct SystemMessage {
    pub text: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Tool {
    pub tool_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Metadata {
    pub user_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Usage {
    pub input_tokens: usize,
}