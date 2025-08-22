use crate::config::types::Config;
use crate::router::route_logic::{RouteRequest, Usage};

pub fn check_long_context(
    req: &RouteRequest,
    token_count: usize,
    config: &Config,
    last_usage: Option<&Usage>,
) -> Option<String> {
    // 检查是否需要使用长上下文模型
    let long_context_threshold = config.router.long_context_threshold.unwrap_or(60000);
    let last_usage_threshold = last_usage
        .map(|usage| usage.input_tokens > long_context_threshold as usize && token_count > 20000)
        .unwrap_or(false);
    let token_count_threshold = token_count > long_context_threshold as usize;
    
    if (last_usage_threshold || token_count_threshold) 
        && config.router.long_context.is_some() {
        Some(config.router.long_context.as_ref().unwrap().clone())
    } else {
        None
    }
}