use crate::config::types::Config;
use crate::router::route_logic::{RouteRequest, Usage};

pub fn check_think_model(
    req: &RouteRequest,
    _token_count: usize,
    config: &Config,
    _last_usage: Option<&Usage>,
) -> Option<String> {
    // 如果存在思考模式，使用思考模型
    if req.body.thinking.is_some() && config.router.think.is_some() {
        Some(config.router.think.as_ref().unwrap().clone())
    } else {
        None
    }
}