use crate::config::types::Config;
use crate::router::route_logic::{RouteRequest, Usage};

pub fn check_web_search(
    req: &RouteRequest,
    _token_count: usize,
    config: &Config,
    _last_usage: Option<&Usage>,
) -> Option<String> {
    // 检查是否需要使用网络搜索模型
    if let Some(tools) = &req.body.tools {
        for tool in tools {
            if let Some(tool_type) = &tool.tool_type {
                if tool_type.starts_with("web_search") && config.router.web_search.is_some() {
                    return Some(config.router.web_search.as_ref().unwrap().clone());
                }
            }
        }
    }
    None
}