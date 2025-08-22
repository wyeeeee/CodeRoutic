use crate::config::types::Config;
use crate::router::route_logic::{RouteLogic, RouteRequest, Usage};
use std::collections::HashMap;

pub struct RouteHandler;

impl RouteHandler {
    pub fn handle_route(
        req: &mut RouteRequest,
        config: &Config,
        session_usage_cache: &HashMap<String, Usage>,
    ) -> String {
        // 解析sessionId从metadata.user_id
        let session_id = req
            .body
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.user_id.as_ref())
            .and_then(|user_id| {
                if let Some(pos) = user_id.find("_session_") {
                    Some(user_id[pos + 9..].to_string())
                } else {
                    None
                }
            });

        // 获取上一次的使用情况
        let last_usage = session_id
            .as_ref()
            .and_then(|id| session_usage_cache.get(id));

        // 计算token数量
        let token_count = Self::calculate_token_count(req);

        // 使用路由逻辑获取应该使用的模型
        RouteLogic::get_use_model(req, token_count, config, last_usage)
    }

    fn calculate_token_count(req: &RouteRequest) -> usize {
        // 简化实现，实际应该根据消息内容计算token数量
        // 这里只是一个示例实现
        let mut token_count = 0;
        
        // 如果有system消息，增加token计数
        if let Some(system) = &req.body.system {
            for msg in system {
                if let Some(text) = &msg.text {
                    // 简单估算：每个字符约0.25个token
                    token_count += (text.len() as f64 * 0.25) as usize;
                }
            }
        }
        
        token_count
    }
}