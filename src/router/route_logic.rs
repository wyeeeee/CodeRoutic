use crate::config::types::{Config, Provider};
use serde_json::Value;
use std::collections::HashMap;

pub struct RouteLogic;

impl RouteLogic {
    pub fn get_use_model(
        req: &RouteRequest,
        token_count: usize,
        config: &Config,
        last_usage: Option<&Usage>,
    ) -> String {
        // 如果请求中的模型包含逗号，说明已经指定了提供商和模型
        if let Some(model_str) = &req.body.model {
            if model_str.contains(',') {
                let parts: Vec<&str> = model_str.split(',').collect();
                if parts.len() == 2 {
                    let provider_name = parts[0];
                    let model_name = parts[1];
                    
                    // 查找匹配的提供商和模型
                    for provider in &config.providers {
                        if provider.name.to_lowercase() == provider_name.to_lowercase() {
                            for model in &provider.models {
                                if model.to_lowercase() == model_name.to_lowercase() {
                                    return format!("{},{}", provider.name, model);
                                }
                            }
                        }
                    }
                }
                return model_str.clone();
            }
        }
        
        // 检查是否需要使用长上下文模型
        let long_context_threshold = config.router.long_context_threshold.unwrap_or(60000);
        let last_usage_threshold = last_usage
            .map(|usage| usage.input_tokens > long_context_threshold as usize && token_count > 20000)
            .unwrap_or(false);
        let token_count_threshold = token_count > long_context_threshold as usize;
        
        if (last_usage_threshold || token_count_threshold) 
            && config.router.long_context.is_some() {
            return config.router.long_context.as_ref().unwrap().clone();
        }
        
        // 检查子代理模型
        if let Some(system) = &req.body.system {
            if system.len() > 1 {
                if let Some(text) = system[1].text.as_ref() {
                    if text.starts_with("<CCR-SUBAGENT-MODEL>") {
                        if let Some(model) = Self::extract_subagent_model(text) {
                            return model;
                        }
                    }
                }
            }
        }
        
        // 如果模型是 claude-3-5-haiku，使用后台模型
        if let Some(model) = &req.body.model {
            if model.starts_with("claude-3-5-haiku") && config.router.background.is_some() {
                return config.router.background.as_ref().unwrap().clone();
            }
        }
        
        // 如果存在思考模式，使用思考模型
        if req.body.thinking.is_some() && config.router.think.is_some() {
            return config.router.think.as_ref().unwrap().clone();
        }
        
        // 检查是否需要使用网络搜索模型
        if let Some(tools) = &req.body.tools {
            for tool in tools {
                if let Some(tool_type) = &tool.tool_type {
                    if tool_type.starts_with("web_search") && config.router.web_search.is_some() {
                        return config.router.web_search.as_ref().unwrap().clone();
                    }
                }
            }
        }
        
        // 返回默认模型
        config.router.default.clone()
    }
    
    fn extract_subagent_model(text: &str) -> Option<String> {
        let start = "<CCR-SUBAGENT-MODEL>";
        let end = "</CCR-SUBAGENT-MODEL>";
        
        if let Some(start_idx) = text.find(start) {
            let start_pos = start_idx + start.len();
            if let Some(end_idx) = text[start_pos..].find(end) {
                let end_pos = start_pos + end_idx;
                return Some(text[start_pos..end_pos].to_string());
            }
        }
        None
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