use crate::config::types::Config;
use crate::router::route_logic::{RouteRequest, Usage};

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

pub fn check_subagent_model(
    req: &RouteRequest,
    _token_count: usize,
    _config: &Config,
    _last_usage: Option<&Usage>,
) -> Option<String> {
    // 检查子代理模型
    if let Some(system) = &req.body.system {
        if system.len() > 1 {
            if let Some(text) = system[1].text.as_ref() {
                if text.starts_with("<CCR-SUBAGENT-MODEL>") {
                    return extract_subagent_model(text);
                }
            }
        }
    }
    None
}