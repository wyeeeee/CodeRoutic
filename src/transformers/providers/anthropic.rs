use serde::{Deserialize, Serialize};
use crate::transformers::error::{TransformerError, TransformerResult};
use crate::transformers::provider_trait::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<AnthropicMessage>,
    temperature: Option<f64>,
    stream: Option<bool>,
    tools: Option<Vec<AnthropicTool>>,
    tool_choice: Option<AnthropicToolChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: Vec<AnthropicContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum AnthropicContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnthropicTool {
    name: String,
    description: String,
    input_schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnthropicToolChoice {
    #[serde(rename = "type")]
    choice_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnthropicResponse {
    id: String,
    #[serde(rename = "type")]
    response_type: String,
    role: String,
    content: Vec<AnthropicContent>,
    model: String,
    stop_reason: Option<String>,
    usage: AnthropicUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnthropicStreamChunk {
    #[serde(rename = "type")]
    chunk_type: String,
    index: Option<u32>,
    delta: Option<AnthropicStreamDelta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum AnthropicStreamDelta {
    #[serde(rename = "text_delta")]
    TextDelta { text: String },
    #[serde(rename = "tool_use_delta")]
    ToolUseDelta { partial_json: String },
}

pub struct AnthropicTransformer;

impl AnthropicTransformer {
    pub fn new() -> Self {
        Self
    }

    fn convert_message_to_universal(msg: &AnthropicMessage) -> TransformerResult<ChatMessage> {
        let parts: Vec<MessagePart> = msg.content.iter().map(|content| match content {
            AnthropicContent::Text { text } => MessagePart {
                part_type: "text".to_string(),
                text: Some(text.clone()),
                tool_use_id: None,
                tool_name: None,
                tool_input: None,
                image_url: None,
            },
            AnthropicContent::ToolUse { id, name, input } => MessagePart {
                part_type: "tool_use".to_string(),
                text: None,
                tool_use_id: Some(id.clone()),
                tool_name: Some(name.clone()),
                tool_input: Some(input.clone()),
                image_url: None,
            },
            AnthropicContent::ToolResult { tool_use_id, content } => MessagePart {
                part_type: "tool_result".to_string(),
                text: Some(content.clone()),
                tool_use_id: Some(tool_use_id.clone()),
                tool_name: None,
                tool_input: None,
                image_url: None,
            },
        }).collect();

        Ok(ChatMessage {
            role: msg.role.clone(),
            content: MessageContent::Parts(parts),
            name: None,
        })
    }

    fn convert_message_from_universal(msg: &ChatMessage) -> TransformerResult<AnthropicMessage> {
        let content = match &msg.content {
            MessageContent::Text(text) => {
                vec![AnthropicContent::Text { text: text.clone() }]
            },
            MessageContent::Parts(parts) => {
                parts.iter().map(|part| {
                    match part.part_type.as_str() {
                        "text" => AnthropicContent::Text { 
                            text: part.text.clone().unwrap_or_default() 
                        },
                        "tool_use" => AnthropicContent::ToolUse {
                            id: part.tool_use_id.clone().unwrap_or_default(),
                            name: part.tool_name.clone().unwrap_or_default(),
                            input: part.tool_input.clone().unwrap_or(serde_json::Value::Null),
                        },
                        "tool_result" => AnthropicContent::ToolResult {
                            tool_use_id: part.tool_use_id.clone().unwrap_or_default(),
                            content: part.text.clone().unwrap_or_default(),
                        },
                        _ => AnthropicContent::Text { 
                            text: part.text.clone().unwrap_or_default() 
                        },
                    }
                }).collect()
            }
        };

        Ok(AnthropicMessage {
            role: msg.role.clone(),
            content,
        })
    }

    fn convert_tool_to_universal(tool: &AnthropicTool) -> TransformerResult<Tool> {
        Ok(Tool {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: tool.name.clone(),
                description: tool.description.clone(),
                parameters: tool.input_schema.clone(),
            },
        })
    }

    fn convert_tool_from_universal(tool: &Tool) -> TransformerResult<AnthropicTool> {
        Ok(AnthropicTool {
            name: tool.function.name.clone(),
            description: tool.function.description.clone(),
            input_schema: tool.function.parameters.clone(),
        })
    }

    fn convert_tool_choice_to_universal(choice: &AnthropicToolChoice) -> TransformerResult<ToolChoice> {
        match choice.choice_type.as_str() {
            "auto" => Ok(ToolChoice::Auto("auto".to_string())),
            "any" => Ok(ToolChoice::Required("any".to_string())),
            _ => Ok(ToolChoice::Auto("auto".to_string())),
        }
    }

    fn convert_tool_choice_from_universal(choice: &ToolChoice) -> TransformerResult<AnthropicToolChoice> {
        match choice {
            ToolChoice::Auto(_) => Ok(AnthropicToolChoice { choice_type: "auto".to_string() }),
            ToolChoice::Required(_) => Ok(AnthropicToolChoice { choice_type: "any".to_string() }),
            ToolChoice::None(_) => Ok(AnthropicToolChoice { choice_type: "auto".to_string() }),
            ToolChoice::Specific(_) => Ok(AnthropicToolChoice { choice_type: "auto".to_string() }),
        }
    }

    fn extract_text_from_content(content: &Vec<AnthropicContent>) -> String {
        content.iter()
            .filter_map(|c| match c {
                AnthropicContent::Text { text } => Some(text.clone()),
                _ => None,
            })
            .collect()
    }

    fn extract_tool_calls_from_content(content: &Vec<AnthropicContent>) -> Vec<ToolCall> {
        content.iter()
            .filter_map(|c| match c {
                AnthropicContent::ToolUse { id, name, input } => {
                    Some(ToolCall {
                        id: Some(id.clone()),
                        tool_type: "function".to_string(),
                        function: FunctionCall {
                            name: name.clone(),
                            arguments: serde_json::to_string(input).unwrap_or_default(),
                        },
                    })
                },
                _ => None,
            })
            .collect()
    }
}

impl ProviderTransformer for AnthropicTransformer {
    fn provider_name(&self) -> &'static str {
        "anthropic"
    }

    fn to_universal_request(&self, request: &serde_json::Value) -> TransformerResult<ChatRequest> {
        let anthropic_request: AnthropicRequest = serde_json::from_value(request.clone())
            .map_err(|e| TransformerError::Deserialization(e.to_string()))?;

        let messages: Result<Vec<ChatMessage>, TransformerError> = anthropic_request
            .messages
            .iter()
            .map(|msg| Self::convert_message_to_universal(msg))
            .collect();

        let tools = anthropic_request
            .tools
            .map(|tools| {
                tools
                    .iter()
                    .map(|tool| Self::convert_tool_to_universal(tool))
                    .collect::<TransformerResult<Vec<Tool>>>()
            })
            .transpose()?;

        let tool_choice = anthropic_request
            .tool_choice
            .map(|choice| Self::convert_tool_choice_to_universal(&choice))
            .transpose()?;

        Ok(ChatRequest {
            model: anthropic_request.model,
            messages: messages?,
            temperature: anthropic_request.temperature,
            max_tokens: Some(anthropic_request.max_tokens),
            stream: anthropic_request.stream.unwrap_or(false),
            tools,
            tool_choice,
            provider_metadata: None,
        })
    }

    fn from_universal_request(&self, request: &ChatRequest) -> TransformerResult<serde_json::Value> {
        let messages: Result<Vec<AnthropicMessage>, TransformerError> = request
            .messages
            .iter()
            .map(|msg| Self::convert_message_from_universal(msg))
            .collect();

        let tools = request
            .tools
            .as_ref()
            .map(|tools| {
                tools
                    .iter()
                    .map(|tool| Self::convert_tool_from_universal(tool))
                    .collect::<TransformerResult<Vec<AnthropicTool>>>()
            })
            .transpose()?;

        let tool_choice = request
            .tool_choice
            .as_ref()
            .map(|choice| Self::convert_tool_choice_from_universal(choice))
            .transpose()?;

        let anthropic_request = AnthropicRequest {
            model: request.model.clone(),
            max_tokens: request.max_tokens.unwrap_or(1000),
            messages: messages?,
            temperature: request.temperature,
            stream: Some(request.stream),
            tools,
            tool_choice,
        };

        serde_json::to_value(anthropic_request)
            .map_err(|e| TransformerError::Serialization(e.to_string()))
    }

    fn to_universal_response(&self, response: &serde_json::Value) -> TransformerResult<ChatResponse> {
        let anthropic_response: AnthropicResponse = serde_json::from_value(response.clone())
            .map_err(|e| TransformerError::Deserialization(e.to_string()))?;

        let tool_calls = Self::extract_tool_calls_from_content(&anthropic_response.content);
        let tool_calls = if tool_calls.is_empty() { None } else { Some(tool_calls) };

        let message = Self::convert_message_to_universal(&AnthropicMessage {
            role: anthropic_response.role,
            content: anthropic_response.content.clone(),
        })?;

        let finish_reason = anthropic_response.stop_reason.unwrap_or("end_turn".to_string());

        let choice = Choice {
            index: 0,
            message,
            finish_reason,
            tool_calls,
        };

        Ok(ChatResponse {
            id: anthropic_response.id,
            object: anthropic_response.response_type,
            created: chrono::Utc::now().timestamp() as u64,
            model: anthropic_response.model,
            choices: vec![choice],
            usage: Usage {
                prompt_tokens: anthropic_response.usage.input_tokens,
                completion_tokens: anthropic_response.usage.output_tokens,
                total_tokens: anthropic_response.usage.input_tokens + anthropic_response.usage.output_tokens,
            },
            provider_metadata: None,
        })
    }

    fn from_universal_response(&self, response: &ChatResponse) -> TransformerResult<serde_json::Value> {
        let message = Self::convert_message_from_universal(&response.choices[0].message.clone())?;

        let stop_reason = match response.choices[0].finish_reason.as_str() {
            "tool_calls" => Some("tool_use".to_string()),
            "stop" => Some("end_turn".to_string()),
            _ => None,
        };

        let anthropic_response = AnthropicResponse {
            id: response.id.clone(),
            response_type: "message".to_string(),
            role: "assistant".to_string(),
            content: message.content,
            model: response.model.clone(),
            stop_reason,
            usage: AnthropicUsage {
                input_tokens: response.usage.prompt_tokens,
                output_tokens: response.usage.completion_tokens,
            },
        };

        serde_json::to_value(anthropic_response)
            .map_err(|e| TransformerError::Serialization(e.to_string()))
    }

    fn to_universal_stream_chunk(&self, chunk: &serde_json::Value) -> TransformerResult<ChatStreamChunk> {
        let anthropic_chunk: AnthropicStreamChunk = serde_json::from_value(chunk.clone())
            .map_err(|e| TransformerError::Deserialization(e.to_string()))?;

        match &anthropic_chunk.delta {
            Some(AnthropicStreamDelta::TextDelta { text }) => {
                let delta = StreamDelta {
                    role: Some("assistant".to_string()),
                    content: Some(text.clone()),
                    tool_calls: None,
                };
                let choice = StreamChoice {
                    index: anthropic_chunk.index.unwrap_or(0),
                    delta,
                    finish_reason: None,
                };
                return Ok(ChatStreamChunk {
                    id: "stream".to_string(),
                    object: "chat.completion.chunk".to_string(),
                    created: chrono::Utc::now().timestamp() as u64,
                    model: "anthropic".to_string(),
                    choices: vec![choice],
                    provider_metadata: None,
                });
            },
            Some(AnthropicStreamDelta::ToolUseDelta { .. }) => {
                let delta = StreamDelta {
                    role: Some("assistant".to_string()),
                    content: None,
                    tool_calls: Some(vec![StreamToolCall {
                        index: 0,
                        id: Some("tool_".to_string()),
                        tool_type: Some("function".to_string()),
                        function: Some(StreamFunctionCall {
                            name: None,
                            arguments: None,
                        }),
                    }]),
                };
                let choice = StreamChoice {
                    index: anthropic_chunk.index.unwrap_or(0),
                    delta,
                    finish_reason: None,
                };
                return Ok(ChatStreamChunk {
                    id: "stream".to_string(),
                    object: "chat.completion.chunk".to_string(),
                    created: chrono::Utc::now().timestamp() as u64,
                    model: "anthropic".to_string(),
                    choices: vec![choice],
                    provider_metadata: None,
                });
            },
            None => {
                return Ok(ChatStreamChunk {
                    id: "stream".to_string(),
                    object: "chat.completion.chunk".to_string(),
                    created: chrono::Utc::now().timestamp() as u64,
                    model: "anthropic".to_string(),
                    choices: vec![],
                    provider_metadata: None,
                });
            }
        }
    }

    fn from_universal_stream_chunk(&self, chunk: &ChatStreamChunk) -> TransformerResult<serde_json::Value> {
        let choice = &chunk.choices[0];
        
        if let Some(content) = &choice.delta.content {
            let anthropic_chunk = AnthropicStreamChunk {
                chunk_type: "content_block_delta".to_string(),
                index: Some(choice.index),
                delta: Some(AnthropicStreamDelta::TextDelta { 
                    text: content.clone() 
                }),
            };
            return serde_json::to_value(anthropic_chunk)
                .map_err(|e| TransformerError::Serialization(e.to_string()));
        }

        let anthropic_chunk = AnthropicStreamChunk {
            chunk_type: "message_stop".to_string(),
            index: None,
            delta: None,
        };

        serde_json::to_value(anthropic_chunk)
            .map_err(|e| TransformerError::Serialization(e.to_string()))
    }
}