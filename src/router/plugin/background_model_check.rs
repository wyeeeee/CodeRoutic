use crate::config::types::Config;
use crate::router::route_logic::{RouteRequest, Usage};

pub fn check_background_model(
    req: &RouteRequest,
    _token_count: usize,
    config: &Config,
    _last_usage: Option<&Usage>,
) -> Option<String> {
    // 如果模型是 claude-3-5-haiku，使用后台模型
    if let Some(model) = &req.body.model {
        if model.starts_with("claude-3-5-haiku") && config.router.background.is_some() {
            return Some(config.router.background.as_ref().unwrap().clone());
        }
    }
    None
}