use serde::{Deserialize, Serialize};
use crate::transformers::error::{TransformerError, TransformerResult};
use crate::transformers::providers::provider_trait::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(rename = "generationConfig")]
    generation_config: Option<GeminiGenerationConfig>,
    tools: Option<Vec<GeminiTool>>,
    #[serde(rename = "toolConfig")]
    tool_config: Option<GeminiToolConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum GeminiPart {
    Text { text: String },
    FunctionCall { function_call: GeminiFunctionCall },
    FunctionResponse { 
        function_response: GeminiFunctionResponse 
    },
    InlineData { 
        inline_data: GeminiInlineData 
    },
    FileData { 
        file_data: GeminiFileData 
    },
    // Generic fallback for other formats
    #[serde(skip)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiFunctionCall {
    name: String,
    args: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiFunctionResponse {
    name: String,
    response: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiInlineData {
    mime_type: String,
    data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiFileData {
    mime_type: String,
    file_uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct GeminiGenerationConfig {
    temperature: Option<f64>,
    #[serde(rename = "maxOutputTokens")]
    max_output_tokens: Option<u32>,
    top_p: Option<f64>,
    top_k: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiTool {
    function_declarations: Vec<GeminiFunctionDeclaration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiFunctionDeclaration {
    name: String,
    description: String,
    parameters: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiToolConfig {
    function_calling_config: GeminiFunctionCallingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiFunctionCallingConfig {
    mode: String,
    allowed_function_names: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
    #[serde(rename = "usageMetadata")]
    usage_metadata: GeminiUsageMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
    #[serde(rename = "finishReason")]
    finish_reason: Option<String>,
    index: u32,
    safety_ratings: Option<Vec<GeminiSafetyRating>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiSafetyRating {
    category: String,
    probability: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiUsageMetadata {
    #[serde(rename = "promptTokenCount")]
    prompt_token_count: u32,
    #[serde(rename = "candidatesTokenCount")]
    candidates_token_count: u32,
    #[serde(rename = "totalTokenCount")]
    total_token_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiStreamChunk {
    candidates: Option<Vec<GeminiCandidate>>,
}

pub struct GeminiTransformer;

impl GeminiTransformer {
    pub fn new() -> Self {
        Self
    }

    fn convert_content_to_universal(content: &GeminiContent) -> TransformerResult<ChatMessage> {
        let parts: Vec<MessagePart> = content.parts.iter().map(|part| match part {
            GeminiPart::Text { text } => MessagePart {
                part_type: "text".to_string(),
                text: Some(text.clone()),
                tool_use_id: None,
                tool_name: None,
                tool_input: None,
                image_url: None,
            },
            GeminiPart::FunctionCall { function_call } => MessagePart {
                part_type: "function_call".to_string(),
                text: None,
                tool_use_id: None,
                tool_name: Some(function_call.name.clone()),
                tool_input: Some(function_call.args.clone()),
                image_url: None,
            },
            GeminiPart::FunctionResponse { function_response } => MessagePart {
                part_type: "function_response".to_string(),
                text: Some(serde_json::to_string(&function_response.response).unwrap_or_default()),
                tool_use_id: None,
                tool_name: Some(function_response.name.clone()),
                tool_input: None,
                image_url: None,
            },
            GeminiPart::InlineData { inline_data } => MessagePart {
                part_type: "image".to_string(),
                text: None,
                tool_use_id: None,
                tool_name: None,
                tool_input: None,
                image_url: Some(ImageUrl {
                    url: format!("data:{};base64,{}", inline_data.mime_type, inline_data.data),
                    detail: None,
                }),
            },
            GeminiPart::FileData { file_data } => MessagePart {
                part_type: "file".to_string(),
                text: None,
                tool_use_id: None,
                tool_name: None,
                tool_input: None,
                image_url: Some(ImageUrl {
                    url: file_data.file_uri.clone(),
                    detail: None,
                }),
            },
            GeminiPart::Unknown => MessagePart {
                part_type: "unknown".to_string(),
                text: None,
                tool_use_id: None,
                tool_name: None,
                tool_input: None,
                image_url: None,
            },
        }).collect();

        let role = match content.role.as_str() {
            "user" => "user".to_string(),
            "model" => "assistant".to_string(),
            "function" => "tool".to_string(),
            _ => "user".to_string(),
        };

        Ok(ChatMessage {
            role,
            content: MessageContent::Parts(parts),
            name: None,
        })
    }

    fn convert_content_from_universal(msg: &ChatMessage) -> TransformerResult<GeminiContent> {
        let role = match msg.role.as_str() {
            "user" => "user".to_string(),
            "assistant" => "model".to_string(),
            "tool" => "function".to_string(),
            _ => "user".to_string(),
        };

        let parts = match &msg.content {
            MessageContent::Text(text) => {
                vec![GeminiPart::Text { text: text.clone() }]
            },
            MessageContent::Parts(parts) => {
                parts.iter().map(|part| {
                    match part.part_type.as_str() {
                        "text" => GeminiPart::Text { 
                            text: part.text.clone().unwrap_or_default() 
                        },
                        "function_call" => GeminiPart::FunctionCall {
                            function_call: GeminiFunctionCall {
                                name: part.tool_name.clone().unwrap_or_default(),
                                args: part.tool_input.clone().unwrap_or(serde_json::Value::Null),
                            },
                        },
                        "function_response" => GeminiPart::FunctionResponse {
                            function_response: GeminiFunctionResponse {
                                name: part.tool_name.clone().unwrap_or_default(),
                                response: serde_json::from_str(&part.text.clone().unwrap_or_default())
                                    .unwrap_or(serde_json::Value::Null),
                            },
                        },
                        "image" => {
                            if let Some(image_url) = &part.image_url {
                                if image_url.url.starts_with("data:") {
                                    let parts: Vec<&str> = image_url.url.splitn(2, ',').collect();
                                    if parts.len() == 2 {
                                        let header_parts: Vec<&str> = parts[0].splitn(2, ';').collect();
                                        if header_parts.len() == 2 {
                                            let mime_type = header_parts[0].strip_prefix("data:").unwrap_or("image/png");
                                            let data = header_parts[1].strip_prefix("base64,").unwrap_or(parts[1]);
                                            return GeminiPart::InlineData {
                                                inline_data: GeminiInlineData {
                                                    mime_type: mime_type.to_string(),
                                                    data: data.to_string(),
                                                },
                                            };
                                        }
                                    }
                                }
                            }
                            GeminiPart::Text { text: "[Image]".to_string() }
                        },
                        _ => GeminiPart::Text { 
                            text: part.text.clone().unwrap_or_default() 
                        },
                    }
                }).collect()
            }
        };

        Ok(GeminiContent { role, parts })
    }

    fn convert_tool_to_universal(tool: &GeminiTool) -> TransformerResult<Vec<Tool>> {
        let mut tools = Vec::new();
        for func in &tool.function_declarations {
            tools.push(Tool {
                tool_type: "function".to_string(),
                function: FunctionDefinition {
                    name: func.name.clone(),
                    description: func.description.clone(),
                    parameters: func.parameters.clone().unwrap_or(serde_json::json!({})),
                },
            });
        }
        Ok(tools)
    }

    fn convert_tool_from_universal(tools: &Vec<Tool>) -> TransformerResult<GeminiTool> {
        let function_declarations: Result<Vec<GeminiFunctionDeclaration>, TransformerError> = tools
            .iter()
            .map(|tool| Ok(GeminiFunctionDeclaration {
                name: tool.function.name.clone(),
                description: tool.function.description.clone(),
                parameters: Some(tool.function.parameters.clone()),
            }))
            .collect();

        Ok(GeminiTool {
            function_declarations: function_declarations?,
        })
    }

    fn convert_tool_choice_to_universal(config: &GeminiToolConfig) -> TransformerResult<ToolChoice> {
        match config.function_calling_config.mode.as_str() {
            "AUTO" => Ok(ToolChoice::Auto("auto".to_string())),
            "ANY" => Ok(ToolChoice::Required("any".to_string())),
            "NONE" => Ok(ToolChoice::None("none".to_string())),
            _ => Ok(ToolChoice::Auto("auto".to_string())),
        }
    }

    fn convert_tool_choice_from_universal(choice: &ToolChoice) -> TransformerResult<GeminiToolConfig> {
        let mode = match choice {
            ToolChoice::Auto(_) => "AUTO",
            ToolChoice::Required(_) => "ANY",
            ToolChoice::None(_) => "NONE",
            ToolChoice::Specific(_) => "AUTO",
        };

        Ok(GeminiToolConfig {
            function_calling_config: GeminiFunctionCallingConfig {
                mode: mode.to_string(),
                allowed_function_names: None,
            },
        })
    }

    fn extract_tool_calls_from_parts(parts: &Vec<GeminiPart>) -> Vec<ToolCall> {
        parts.iter()
            .filter_map(|part| match part {
                GeminiPart::FunctionCall { function_call } => {
                    Some(ToolCall {
                        id: None,
                        tool_type: "function".to_string(),
                        function: FunctionCall {
                            name: function_call.name.clone(),
                            arguments: serde_json::to_string(&function_call.args).unwrap_or_default(),
                        },
                    })
                },
                _ => None,
            })
            .collect()
    }

    fn extract_text_from_parts(parts: &Vec<GeminiPart>) -> String {
        parts.iter()
            .filter_map(|part| match part {
                GeminiPart::Text { text } => Some(text.clone()),
                _ => None,
            })
            .collect()
    }
}

impl ProviderTransformer for GeminiTransformer {
    fn provider_name(&self) -> &'static str {
        "gemini"
    }

    fn to_universal_request(&self, request: &serde_json::Value) -> TransformerResult<ChatRequest> {
        let gemini_request: GeminiRequest = serde_json::from_value(request.clone())
            .map_err(|e| TransformerError::Deserialization(e.to_string()))?;

        let messages: Result<Vec<ChatMessage>, TransformerError> = gemini_request
            .contents
            .iter()
            .map(|content| Self::convert_content_to_universal(content))
            .collect();

        let mut universal_tools = Vec::new();
        if let Some(gemini_tools) = &gemini_request.tools {
            for tool in gemini_tools {
                let converted_tools = Self::convert_tool_to_universal(tool)?;
                universal_tools.extend(converted_tools);
            }
        }
        let tools = if universal_tools.is_empty() { None } else { Some(universal_tools) };

        let tool_choice = gemini_request
            .tool_config
            .map(|config| Self::convert_tool_choice_to_universal(&config))
            .transpose()?;

        let generation_config = gemini_request.generation_config.unwrap_or(GeminiGenerationConfig {
        temperature: None,
        max_output_tokens: None,
        top_p: None,
        top_k: None,
    });

        Ok(ChatRequest {
            model: "gemini".to_string(),
            messages: messages?,
            temperature: generation_config.temperature,
            max_tokens: generation_config.max_output_tokens,
            stream: false,
            tools,
            tool_choice,
            provider_metadata: None,
        })
    }

    fn from_universal_request(&self, request: &ChatRequest) -> TransformerResult<serde_json::Value> {
        let contents: Result<Vec<GeminiContent>, TransformerError> = request
            .messages
            .iter()
            .map(|msg| Self::convert_content_from_universal(msg))
            .collect();

        let tools = if let Some(universal_tools) = &request.tools {
            Some(vec![Self::convert_tool_from_universal(universal_tools)?])
        } else {
            None
        };

        let tool_config = request
            .tool_choice
            .as_ref()
            .map(|choice| Self::convert_tool_choice_from_universal(choice))
            .transpose()?;

        let generation_config = GeminiGenerationConfig {
            temperature: request.temperature,
            max_output_tokens: request.max_tokens,
            top_p: None,
            top_k: None,
        };

        let gemini_request = GeminiRequest {
            contents: contents?,
            generation_config: Some(generation_config),
            tools,
            tool_config,
        };

        serde_json::to_value(gemini_request)
            .map_err(|e| TransformerError::Serialization(e.to_string()))
    }

    fn to_universal_response(&self, response: &serde_json::Value) -> TransformerResult<ChatResponse> {
        let gemini_response: GeminiResponse = serde_json::from_value(response.clone())
            .map_err(|e| TransformerError::Deserialization(e.to_string()))?;

        let candidate = &gemini_response.candidates[0];
        let message = Self::convert_content_to_universal(&candidate.content)?;

        let tool_calls = Self::extract_tool_calls_from_parts(&candidate.content.parts);
        let tool_calls = if tool_calls.is_empty() { None } else { Some(tool_calls) };

        let finish_reason = candidate.finish_reason.clone().unwrap_or("STOP".to_string());

        let choice = Choice {
            index: candidate.index,
            message,
            finish_reason,
            tool_calls,
        };

        Ok(ChatResponse {
            id: "gemini_response".to_string(),
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp() as u64,
            model: "gemini".to_string(),
            choices: vec![choice],
            usage: Usage {
                prompt_tokens: gemini_response.usage_metadata.prompt_token_count,
                completion_tokens: gemini_response.usage_metadata.candidates_token_count,
                total_tokens: gemini_response.usage_metadata.total_token_count,
            },
            provider_metadata: None,
        })
    }

    fn from_universal_response(&self, response: &ChatResponse) -> TransformerResult<serde_json::Value> {
        let message = Self::convert_content_from_universal(&response.choices[0].message.clone())?;

        let finish_reason = match response.choices[0].finish_reason.as_str() {
            "stop" => Some("STOP".to_string()),
            "tool_calls" => Some("STOP".to_string()),
            _ => None,
        };

        let candidate = GeminiCandidate {
            content: message,
            finish_reason,
            index: response.choices[0].index,
            safety_ratings: None,
        };

        let gemini_response = GeminiResponse {
            candidates: vec![candidate],
            usage_metadata: GeminiUsageMetadata {
                prompt_token_count: response.usage.prompt_tokens,
                candidates_token_count: response.usage.completion_tokens,
                total_token_count: response.usage.total_tokens,
            },
        };

        serde_json::to_value(gemini_response)
            .map_err(|e| TransformerError::Serialization(e.to_string()))
    }

    fn to_universal_stream_chunk(&self, chunk: &serde_json::Value) -> TransformerResult<ChatStreamChunk> {
        let gemini_chunk: GeminiStreamChunk = serde_json::from_value(chunk.clone())
            .map_err(|e| TransformerError::Deserialization(e.to_string()))?;

        if let Some(candidates) = gemini_chunk.candidates {
            if !candidates.is_empty() {
                let candidate = &candidates[0];
                let _message = Self::convert_content_to_universal(&candidate.content)?;

                let text_content = Self::extract_text_from_parts(&candidate.content.parts);
                
                let delta = StreamDelta {
                    role: Some("assistant".to_string()),
                    content: Some(text_content),
                    tool_calls: None,
                };

                let choice = StreamChoice {
                    index: candidate.index,
                    delta,
                    finish_reason: candidate.finish_reason.clone(),
                };

                return Ok(ChatStreamChunk {
                    id: "gemini_stream".to_string(),
                    object: "chat.completion.chunk".to_string(),
                    created: chrono::Utc::now().timestamp() as u64,
                    model: "gemini".to_string(),
                    choices: vec![choice],
                    provider_metadata: None,
                });
            }
        }

        Ok(ChatStreamChunk {
            id: "gemini_stream".to_string(),
            object: "chat.completion.chunk".to_string(),
            created: chrono::Utc::now().timestamp() as u64,
            model: "gemini".to_string(),
            choices: vec![],
            provider_metadata: None,
        })
    }

    fn from_universal_stream_chunk(&self, chunk: &ChatStreamChunk) -> TransformerResult<serde_json::Value> {
        let choice = &chunk.choices[0];
        
        if let Some(content) = &choice.delta.content {
            let candidate = GeminiCandidate {
                content: GeminiContent {
                    role: "model".to_string(),
                    parts: vec![GeminiPart::Text { text: content.clone() }],
                },
                finish_reason: None,
                index: choice.index,
                safety_ratings: None,
            };

            let gemini_chunk = GeminiStreamChunk {
                candidates: Some(vec![candidate]),
            };

            return serde_json::to_value(gemini_chunk)
                .map_err(|e| TransformerError::Serialization(e.to_string()));
        }

        let gemini_chunk = GeminiStreamChunk {
            candidates: None,
        };

        serde_json::to_value(gemini_chunk)
            .map_err(|e| TransformerError::Serialization(e.to_string()))
    }
}