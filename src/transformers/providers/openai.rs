use serde::{Deserialize, Serialize};
use crate::transformers::error::{TransformerError, TransformerResult};
use crate::transformers::provider_trait::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: Option<f64>,
    max_tokens: Option<u32>,
    stream: Option<bool>,
    tools: Option<Vec<OpenAITool>>,
    tool_choice: Option<OpenAIToolChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: Option<String>,
    name: Option<String>,
    tool_calls: Option<Vec<OpenAIToolCall>>,
    tool_call_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAITool {
    #[serde(rename = "type")]
    tool_type: String,
    function: OpenAIFunctionDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIFunctionDefinition {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum OpenAIToolChoice {
    Auto(String),
    None(String),
    Required(String),
    Specific(OpenAIToolChoiceSpecific),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIToolChoiceSpecific {
    #[serde(rename = "type")]
    choice_type: String,
    function: OpenAIFunctionChoice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIFunctionChoice {
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIToolCall {
    id: String,
    #[serde(rename = "type")]
    tool_type: String,
    function: OpenAIFunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIFunctionCall {
    name: String,
    arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIChoice {
    index: u32,
    message: OpenAIMessage,
    finish_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIStreamChunk {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<OpenAIStreamChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIStreamChoice {
    index: u32,
    delta: OpenAIStreamDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIStreamDelta {
    role: Option<String>,
    content: Option<String>,
    tool_calls: Option<Vec<OpenAIStreamToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIStreamToolCall {
    index: u32,
    id: Option<String>,
    #[serde(rename = "type")]
    tool_type: Option<String>,
    function: Option<OpenAIStreamFunctionCall>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIStreamFunctionCall {
    name: Option<String>,
    arguments: Option<String>,
}

pub struct OpenAITransformer;

impl OpenAITransformer {
    pub fn new() -> Self {
        Self
    }

    fn convert_message_to_universal(msg: &OpenAIMessage) -> TransformerResult<ChatMessage> {
        let content = match msg.content.clone() {
            Some(text) => MessageContent::Text(text),
            None => MessageContent::Text(String::new()),
        };

        Ok(ChatMessage {
            role: msg.role.clone(),
            content,
            name: msg.name.clone(),
        })
    }

    fn convert_message_from_universal(msg: &ChatMessage) -> TransformerResult<OpenAIMessage> {
        let content = match &msg.content {
            MessageContent::Text(text) => Some(text.clone()),
            MessageContent::Parts(parts) => {
                let text_parts: Vec<String> = parts
                    .iter()
                    .filter(|p| p.part_type == "text" && p.text.is_some())
                    .map(|p| p.text.as_ref().unwrap().clone())
                    .collect();
                Some(text_parts.join(""))
            }
        };

        Ok(OpenAIMessage {
            role: msg.role.clone(),
            content,
            name: msg.name.clone(),
            tool_calls: None,
            tool_call_id: None,
        })
    }

    fn convert_tool_to_universal(tool: &OpenAITool) -> TransformerResult<Tool> {
        Ok(Tool {
            tool_type: tool.tool_type.clone(),
            function: FunctionDefinition {
                name: tool.function.name.clone(),
                description: tool.function.description.clone(),
                parameters: tool.function.parameters.clone(),
            },
        })
    }

    fn convert_tool_from_universal(tool: &Tool) -> TransformerResult<OpenAITool> {
        Ok(OpenAITool {
            tool_type: tool.tool_type.clone(),
            function: OpenAIFunctionDefinition {
                name: tool.function.name.clone(),
                description: tool.function.description.clone(),
                parameters: tool.function.parameters.clone(),
            },
        })
    }

    fn convert_tool_choice_to_universal(choice: &OpenAIToolChoice) -> TransformerResult<ToolChoice> {
        match choice {
            OpenAIToolChoice::Auto(_) => Ok(ToolChoice::Auto("auto".to_string())),
            OpenAIToolChoice::None(_) => Ok(ToolChoice::None("none".to_string())),
            OpenAIToolChoice::Required(_) => Ok(ToolChoice::Required("required".to_string())),
            OpenAIToolChoice::Specific(spec) => Ok(ToolChoice::Specific(ToolChoiceSpecific {
                choice_type: spec.choice_type.clone(),
                function: FunctionChoice {
                    name: spec.function.name.clone(),
                },
            })),
        }
    }

    fn convert_tool_choice_from_universal(choice: &ToolChoice) -> TransformerResult<OpenAIToolChoice> {
        match choice {
            ToolChoice::Auto(_) => Ok(OpenAIToolChoice::Auto("auto".to_string())),
            ToolChoice::None(_) => Ok(OpenAIToolChoice::None("none".to_string())),
            ToolChoice::Required(_) => Ok(OpenAIToolChoice::Required("required".to_string())),
            ToolChoice::Specific(spec) => Ok(OpenAIToolChoice::Specific(OpenAIToolChoiceSpecific {
                choice_type: spec.choice_type.clone(),
                function: OpenAIFunctionChoice {
                    name: spec.function.name.clone(),
                },
            })),
        }
    }
}

impl ProviderTransformer for OpenAITransformer {
    fn provider_name(&self) -> &'static str {
        "openai"
    }

    fn to_universal_request(&self, request: &serde_json::Value) -> TransformerResult<ChatRequest> {
        let openai_request: OpenAIRequest = serde_json::from_value(request.clone())
            .map_err(|e| TransformerError::Deserialization(e.to_string()))?;

        let messages: Result<Vec<ChatMessage>, TransformerError> = openai_request
            .messages
            .iter()
            .map(|msg| Self::convert_message_to_universal(msg))
            .collect();

        let tools = openai_request
            .tools
            .map(|tools| {
                tools
                    .iter()
                    .map(|tool| Self::convert_tool_to_universal(tool))
                    .collect::<TransformerResult<Vec<Tool>>>()
            })
            .transpose()?;

        let tool_choice = openai_request
            .tool_choice
            .map(|choice| Self::convert_tool_choice_to_universal(&choice))
            .transpose()?;

        Ok(ChatRequest {
            model: openai_request.model,
            messages: messages?,
            temperature: openai_request.temperature,
            max_tokens: openai_request.max_tokens,
            stream: openai_request.stream.unwrap_or(false),
            tools,
            tool_choice,
            provider_metadata: None,
        })
    }

    fn from_universal_request(&self, request: &ChatRequest) -> TransformerResult<serde_json::Value> {
        let messages: Result<Vec<OpenAIMessage>, TransformerError> = request
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
                    .collect::<TransformerResult<Vec<OpenAITool>>>()
            })
            .transpose()?;

        let tool_choice = request
            .tool_choice
            .as_ref()
            .map(|choice| Self::convert_tool_choice_from_universal(choice))
            .transpose()?;

        let openai_request = OpenAIRequest {
            model: request.model.clone(),
            messages: messages?,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            stream: Some(request.stream),
            tools,
            tool_choice,
        };

        serde_json::to_value(openai_request)
            .map_err(|e| TransformerError::Serialization(e.to_string()))
    }

    fn to_universal_response(&self, response: &serde_json::Value) -> TransformerResult<ChatResponse> {
        let openai_response: OpenAIResponse = serde_json::from_value(response.clone())
            .map_err(|e| TransformerError::Deserialization(e.to_string()))?;

        let message = Self::convert_message_to_universal(&openai_response.choices[0].message.clone())?;

        let tool_calls = openai_response.choices[0].message.tool_calls.clone().map(|calls| {
            calls
                .into_iter()
                .map(|call| ToolCall {
                    id: Some(call.id.clone()),
                    tool_type: call.tool_type,
                    function: FunctionCall {
                        name: call.function.name,
                        arguments: call.function.arguments,
                    },
                })
                .collect()
        });

        let choice = Choice {
            index: openai_response.choices[0].index,
            message,
            finish_reason: openai_response.choices[0].finish_reason.clone(),
            tool_calls,
        };

        Ok(ChatResponse {
            id: openai_response.id,
            object: openai_response.object,
            created: openai_response.created,
            model: openai_response.model,
            choices: vec![choice],
            usage: Usage {
                prompt_tokens: openai_response.usage.prompt_tokens,
                completion_tokens: openai_response.usage.completion_tokens,
                total_tokens: openai_response.usage.total_tokens,
            },
            provider_metadata: None,
        })
    }

    fn from_universal_response(&self, response: &ChatResponse) -> TransformerResult<serde_json::Value> {
        let message = Self::convert_message_from_universal(&response.choices[0].message.clone())?;

        let _tool_calls = response.choices[0].tool_calls.clone().map(|calls| {
            calls
                .into_iter()
                .map(|call| OpenAIToolCall {
                    id: call.id.unwrap_or_default(),
                    tool_type: call.tool_type,
                    function: OpenAIFunctionCall {
                        name: call.function.name,
                        arguments: call.function.arguments,
                    },
                })
                .collect::<Vec<OpenAIToolCall>>()
        });

        let choice = OpenAIChoice {
            index: response.choices[0].index,
            message,
            finish_reason: response.choices[0].finish_reason.clone(),
        };

        let openai_response = OpenAIResponse {
            id: response.id.clone(),
            object: response.object.clone(),
            created: response.created,
            model: response.model.clone(),
            choices: vec![choice],
            usage: OpenAIUsage {
                prompt_tokens: response.usage.prompt_tokens,
                completion_tokens: response.usage.completion_tokens,
                total_tokens: response.usage.total_tokens,
            },
        };

        serde_json::to_value(openai_response)
            .map_err(|e| TransformerError::Serialization(e.to_string()))
    }

    fn to_universal_stream_chunk(&self, chunk: &serde_json::Value) -> TransformerResult<ChatStreamChunk> {
        let openai_chunk: OpenAIStreamChunk = serde_json::from_value(chunk.clone())
            .map_err(|e| TransformerError::Deserialization(e.to_string()))?;

        let choice = &openai_chunk.choices[0];
        let delta = StreamDelta {
            role: choice.delta.role.clone(),
            content: choice.delta.content.clone(),
            tool_calls: choice.delta.tool_calls.clone().map(|calls| {
                calls
                    .into_iter()
                    .map(|call| StreamToolCall {
                        index: call.index,
                        id: call.id,
                        tool_type: call.tool_type,
                        function: call.function.map(|f| StreamFunctionCall {
                            name: f.name,
                            arguments: f.arguments,
                        }),
                    })
                    .collect()
            }),
        };

        let stream_choice = StreamChoice {
            index: choice.index,
            delta,
            finish_reason: choice.finish_reason.clone(),
        };

        Ok(ChatStreamChunk {
            id: openai_chunk.id,
            object: openai_chunk.object,
            created: openai_chunk.created,
            model: openai_chunk.model,
            choices: vec![stream_choice],
            provider_metadata: None,
        })
    }

    fn from_universal_stream_chunk(&self, chunk: &ChatStreamChunk) -> TransformerResult<serde_json::Value> {
        let choice = &chunk.choices[0];
        let delta = OpenAIStreamDelta {
            role: choice.delta.role.clone(),
            content: choice.delta.content.clone(),
            tool_calls: choice.delta.tool_calls.clone().map(|calls| {
                calls
                    .into_iter()
                    .map(|call| OpenAIStreamToolCall {
                        index: call.index,
                        id: call.id,
                        tool_type: call.tool_type,
                        function: call.function.map(|f| OpenAIStreamFunctionCall {
                            name: f.name,
                            arguments: f.arguments,
                        }),
                    })
                    .collect()
            }),
        };

        let openai_choice = OpenAIStreamChoice {
            index: choice.index,
            delta,
            finish_reason: choice.finish_reason.clone(),
        };

        let openai_chunk = OpenAIStreamChunk {
            id: chunk.id.clone(),
            object: chunk.object.clone(),
            created: chunk.created,
            model: chunk.model.clone(),
            choices: vec![openai_choice],
        };

        serde_json::to_value(openai_chunk)
            .map_err(|e| TransformerError::Serialization(e.to_string()))
    }
}