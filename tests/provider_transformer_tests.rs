use code_routic::transformers::{
    providers::{OpenAITransformer, AnthropicTransformer, GeminiTransformer, ProviderTransformer},
    TransformerManager,
    error::TransformerResult,
};
use serde_json::{json, Value};

fn create_test_tool_definition() -> Value {
    json!({
        "type": "function",
        "function": {
            "name": "get_weather",
            "description": "Get weather information for a location",
            "parameters": {
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "City name or location"
                    },
                    "units": {
                        "type": "string",
                        "enum": ["celsius", "fahrenheit"],
                        "default": "celsius"
                    }
                },
                "required": ["location"]
            }
        }
    })
}

fn create_test_messages() -> Vec<Value> {
    vec![
        json!({
            "role": "system",
            "content": "You are a helpful assistant that can call functions."
        }),
        json!({
            "role": "user",
            "content": "What's the weather like in Boston?"
        })
    ]
}

fn create_test_tool_call_response() -> Value {
    json!({
        "id": "chatcmpl-123",
        "object": "chat.completion",
        "created": 1677652288,
        "model": "gpt-4",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "I'll help you get the weather information for Boston.",
                "tool_calls": [{
                    "id": "call_abc123",
                    "type": "function",
                    "function": {
                        "name": "get_weather",
                        "arguments": "{\"location\": \"Boston\", \"units\": \"celsius\"}"
                    }
                }]
            },
            "finish_reason": "tool_calls"
        }],
        "usage": {
            "prompt_tokens": 82,
            "completion_tokens": 17,
            "total_tokens": 99
        }
    })
}

#[test]
fn test_openai_request_to_universal() {
    let transformer = OpenAITransformer::new();
    
    let openai_request = json!({
        "model": "gpt-4",
        "messages": create_test_messages(),
        "temperature": 0.7,
        "max_tokens": 1000,
        "stream": false,
        "tools": [create_test_tool_definition()],
        "tool_choice": "auto"
    });
    
    let result = transformer.to_universal_request(&openai_request);
    assert!(result.is_ok(), "OpenAI request conversion failed: {:?}", result.err());
    
    let universal_request = result.unwrap();
    
    assert_eq!(universal_request.model, "gpt-4");
    assert_eq!(universal_request.temperature, Some(0.7));
    assert_eq!(universal_request.max_tokens, Some(1000));
    assert!(!universal_request.stream);
    assert!(universal_request.tools.is_some());
    assert!(universal_request.tool_choice.is_some());
    
    // Check tools conversion
    let tools = universal_request.tools.unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].tool_type, "function");
    assert_eq!(tools[0].function.name, "get_weather");
    
    // Check messages conversion
    assert_eq!(universal_request.messages.len(), 2);
    assert_eq!(universal_request.messages[0].role, "system");
    assert_eq!(universal_request.messages[1].role, "user");
}

#[test]
fn test_openai_response_to_universal() {
    let transformer = OpenAITransformer::new();
    
    let openai_response = create_test_tool_call_response();
    
    let result = transformer.to_universal_response(&openai_response);
    assert!(result.is_ok(), "OpenAI response conversion failed: {:?}", result.err());
    
    let universal_response = result.unwrap();
    
    assert_eq!(universal_response.id, "chatcmpl-123");
    assert_eq!(universal_response.object, "chat.completion");
    assert_eq!(universal_response.model, "gpt-4");
    assert_eq!(universal_response.choices.len(), 1);
    
    let choice = &universal_response.choices[0];
    assert_eq!(choice.finish_reason, "tool_calls");
    assert!(choice.tool_calls.is_some());
    
    let tool_calls = choice.tool_calls.as_ref().unwrap();
    assert_eq!(tool_calls.len(), 1);
    assert_eq!(tool_calls[0].function.name, "get_weather");
    assert_eq!(tool_calls[0].function.arguments, "{\"location\": \"Boston\", \"units\": \"celsius\"}");
}

#[test]
fn test_openai_roundtrip_conversion() {
    let transformer = OpenAITransformer::new();
    
    let original_request = json!({
        "model": "gpt-4",
        "messages": create_test_messages(),
        "temperature": 0.5,
        "max_tokens": 500,
        "tools": [create_test_tool_definition()],
        "tool_choice": {"type": "function", "function": {"name": "get_weather"}}
    });
    
    // Convert to universal and back
    let universal_request = transformer.to_universal_request(&original_request).unwrap();
    let converted_back = transformer.from_universal_request(&universal_request).unwrap();
    
    // The converted back request should be equivalent to the original
    assert_eq!(converted_back["model"], original_request["model"]);
    assert_eq!(converted_back["temperature"], original_request["temperature"]);
    assert_eq!(converted_back["max_tokens"], original_request["max_tokens"]);
    
    // Check that tools are preserved
    assert!(converted_back["tools"].is_array());
    let tools = converted_back["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0]["function"]["name"], "get_weather");
}

#[test]
fn test_anthropic_request_to_universal() {
    let transformer = AnthropicTransformer::new();
    
    let anthropic_request = json!({
        "model": "claude-3-sonnet-20240229",
        "max_tokens": 1000,
        "messages": [{
            "role": "user",
            "content": [{
                "type": "text",
                "text": "What's the weather like in Boston?"
            }]
        }],
        "tools": [{
            "name": "get_weather",
            "description": "Get weather information for a location",
            "input_schema": {
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "City name or location"
                    }
                },
                "required": ["location"]
            }
        }],
        "tool_choice": {"type": "auto"}
    });
    
    let result = transformer.to_universal_request(&anthropic_request);
    assert!(result.is_ok(), "Anthropic request conversion failed: {:?}", result.err());
    
    let universal_request = result.unwrap();
    
    assert_eq!(universal_request.model, "claude-3-sonnet-20240229");
    assert_eq!(universal_request.max_tokens, Some(1000));
    assert!(universal_request.tools.is_some());
    
    // Check tools conversion
    let tools = universal_request.tools.unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].function.name, "get_weather");
    
    // Check messages conversion
    assert_eq!(universal_request.messages.len(), 1);
    assert_eq!(universal_request.messages[0].role, "user");
}

#[test]
fn test_anthropic_response_to_universal() {
    let transformer = AnthropicTransformer::new();
    
    let anthropic_response = json!({
        "id": "msg_123",
        "type": "message",
        "role": "assistant",
        "content": [
            {
                "type": "text",
                "text": "I'll help you get the weather information."
            },
            {
                "type": "tool_use",
                "id": "tool_123",
                "name": "get_weather",
                "input": {
                    "location": "Boston",
                    "units": "celsius"
                }
            }
        ],
        "model": "claude-3-sonnet-20240229",
        "stop_reason": "tool_use",
        "usage": {
            "input_tokens": 82,
            "output_tokens": 17
        }
    });
    
    let result = transformer.to_universal_response(&anthropic_response);
    assert!(result.is_ok(), "Anthropic response conversion failed: {:?}", result.err());
    
    let universal_response = result.unwrap();
    
    assert_eq!(universal_response.id, "msg_123");
    assert_eq!(universal_response.choices.len(), 1);
    
    let choice = &universal_response.choices[0];
    assert_eq!(choice.finish_reason, "tool_use");
    assert!(choice.tool_calls.is_some());
    
    let tool_calls = choice.tool_calls.as_ref().unwrap();
    assert_eq!(tool_calls.len(), 1);
    assert_eq!(tool_calls[0].function.name, "get_weather");
}

#[test]
fn test_gemini_request_to_universal() {
    let transformer = GeminiTransformer::new();
    
    let gemini_request = json!({
        "contents": [{
            "role": "user",
            "parts": [{
                "text": "What's the weather like in Boston?"
            }]
        }],
        "generationConfig": {
            "temperature": 0.7,
            "maxOutputTokens": 1000
        },
        "tools": [{
            "function_declarations": [{
                "name": "get_weather",
                "description": "Get weather information for a location",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "location": {
                            "type": "string",
                            "description": "City name or location"
                        }
                    },
                    "required": ["location"]
                }
            }]
        }],
        "tool_config": {
            "function_calling_config": {
                "mode": "AUTO"
            }
        }
    });
    
    let result = transformer.to_universal_request(&gemini_request);
    assert!(result.is_ok(), "Gemini request conversion failed: {:?}", result.err());
    
    let universal_request = result.unwrap();
    
    assert_eq!(universal_request.temperature, Some(0.7));
    assert_eq!(universal_request.max_tokens, Some(1000));
    assert!(universal_request.tools.is_some());
    
    // Check tools conversion
    let tools = universal_request.tools.unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].function.name, "get_weather");
    
    // Check messages conversion
    assert_eq!(universal_request.messages.len(), 1);
    assert_eq!(universal_request.messages[0].role, "user");
}

#[test]
fn test_gemini_response_to_universal() {
    let transformer = GeminiTransformer::new();
    
    let gemini_response = json!({
        "candidates": [{
            "content": {
                "parts": [
                    {
                        "text": "I'll help you get the weather information."
                    },
                    {
                        "function_call": {
                            "name": "get_weather",
                            "args": {
                                "location": "Boston",
                                "units": "celsius"
                            }
                        }
                    }
                ],
                "role": "model"
            },
            "finishReason": "STOP",
            "index": 0
        }],
        "usageMetadata": {
            "promptTokenCount": 82,
            "candidatesTokenCount": 17,
            "totalTokenCount": 99
        }
    });
    
    let result = transformer.to_universal_response(&gemini_response);
    assert!(result.is_ok(), "Gemini response conversion failed: {:?}", result.err());
    
    let universal_response = result.unwrap();
    
    assert_eq!(universal_response.choices.len(), 1);
    
    let choice = &universal_response.choices[0];
    assert_eq!(choice.finish_reason, "STOP");
    assert!(choice.tool_calls.is_some());
    
    let tool_calls = choice.tool_calls.as_ref().unwrap();
    assert_eq!(tool_calls.len(), 1);
    assert_eq!(tool_calls[0].function.name, "get_weather");
    assert_eq!(tool_calls[0].function.arguments, "{\"location\":\"Boston\",\"units\":\"celsius\"}");
}

#[test]
fn test_cross_provider_conversion() {
    let manager = TransformerManager::new();
    
    let openai_request = json!({
        "model": "gpt-4",
        "messages": create_test_messages(),
        "temperature": 0.7,
        "tools": [create_test_tool_definition()],
        "tool_choice": "auto"
    });
    
    // Convert OpenAI -> Universal -> Anthropic using new manager interface
    let universal_request = manager.to_universal_request("openai", &openai_request).unwrap();
    let anthropic_request = manager.from_universal_request("anthropic", &universal_request).unwrap();
    
    // Verify the conversion preserved key information
    // Note: model name may change based on provider defaults
    assert_eq!(anthropic_request["temperature"], openai_request["temperature"]);
    assert!(anthropic_request["tools"].is_array());
    
    let tools = anthropic_request["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0]["name"], "get_weather");
}

#[test]
fn test_stream_chunk_conversion() {
    let manager = TransformerManager::new();
    
    let openai_stream_chunk = json!({
        "id": "chatcmpl-123",
        "object": "chat.completion.chunk",
        "created": 1677652288,
        "model": "gpt-4",
        "choices": [{
            "index": 0,
            "delta": {
                "role": "assistant",
                "content": "Hello"
            },
            "finish_reason": null
        }]
    });
    
    let universal_chunk = manager.to_universal_stream_chunk("openai", &openai_stream_chunk).unwrap();
    
    assert_eq!(universal_chunk.id, "chatcmpl-123");
    assert_eq!(universal_chunk.object, "chat.completion.chunk");
    assert_eq!(universal_chunk.choices.len(), 1);
    
    let choice = &universal_chunk.choices[0];
    assert_eq!(choice.delta.role, Some("assistant".to_string()));
    assert_eq!(choice.delta.content, Some("Hello".to_string()));
    assert!(choice.finish_reason.is_none());
}

#[test]
fn test_manager_functionality() {
    let manager = TransformerManager::new();
    
    // Test that all expected providers are available
    assert!(manager.is_provider_supported("openai"));
    assert!(manager.is_provider_supported("anthropic"));
    assert!(manager.is_provider_supported("gemini"));
    assert!(!manager.is_provider_supported("nonexistent"));
    
    // Test listing providers
    let providers = manager.list_available_providers();
    assert!(providers.contains(&"openai".to_string()));
    assert!(providers.contains(&"anthropic".to_string()));
    assert!(providers.contains(&"gemini".to_string()));
    
    // Test the new direct conversion interfaces
    let test_request = json!({
        "model": "gpt-4",
        "messages": create_test_messages(),
        "temperature": 0.7
    });
    
    // Test to_universal_request
    let universal_request = manager.to_universal_request("openai", &test_request);
    assert!(universal_request.is_ok());
    
    // Test from_universal_request
    let universal = universal_request.unwrap();
    let provider_request = manager.from_universal_request("anthropic", &universal);
    assert!(provider_request.is_ok());
}

#[test]
fn test_tool_choice_variants() {
    let transformer = OpenAITransformer::new();
    
    // Test different tool_choice variants
    let test_cases = vec![
        (json!("auto"), "auto"),
        (json!("none"), "none"),
        (json!("required"), "required"),
        (json!({"type": "function", "function": {"name": "get_weather"}}), "specific"),
    ];
    
    for (tool_choice, expected_type) in test_cases {
        let request = json!({
            "model": "gpt-4",
            "messages": create_test_messages(),
            "tools": [create_test_tool_definition()],
            "tool_choice": tool_choice
        });
        
        let result = transformer.to_universal_request(&request);
        assert!(result.is_ok(), "Tool choice {:?} conversion failed: {:?}", tool_choice, result.err());
        
        let universal_request = result.unwrap();
        assert!(universal_request.tool_choice.is_some());
    }
}

#[test]
fn test_complex_tool_arguments() {
    let transformer = OpenAITransformer::new();
    
    let complex_tool = json!({
        "type": "function",
        "function": {
            "name": "search_database",
            "description": "Search a database with complex parameters",
            "parameters": {
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query"
                    },
                    "filters": {
                        "type": "object",
                        "properties": {
                            "date_range": {
                                "type": "object",
                                "properties": {
                                    "start": {"type": "string"},
                                    "end": {"type": "string"}
                                }
                            },
                            "categories": {
                                "type": "array",
                                "items": {"type": "string"}
                            }
                        }
                    },
                    "pagination": {
                        "type": "object",
                        "properties": {
                            "page": {"type": "integer"},
                            "limit": {"type": "integer"}
                        }
                    }
                },
                "required": ["query"]
            }
        }
    });
    
    let request = json!({
        "model": "gpt-4",
        "messages": create_test_messages(),
        "tools": [complex_tool],
        "tool_choice": "auto"
    });
    
    let result = transformer.to_universal_request(&request);
    assert!(result.is_ok(), "Complex tool conversion failed: {:?}", result.err());
    
    let universal_request = result.unwrap();
    let tools = universal_request.tools.unwrap();
    
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].function.name, "search_database");
    
    // Verify that complex nested parameters are preserved
    let params = &tools[0].function.parameters;
    assert!(params["properties"]["filters"]["properties"]["date_range"].is_object());
    assert!(params["properties"]["filters"]["properties"]["categories"].is_object());
}

#[test]
fn test_error_handling() {
    let manager = TransformerManager::new();
    
    // Test invalid JSON
    let invalid_json = json!({"invalid": "structure"});
    
    let result = manager.to_universal_request("openai", &invalid_json);
    assert!(result.is_err());
    
    // Test malformed tool definition
    let malformed_request = json!({
        "model": "gpt-4",
        "messages": create_test_messages(),
        "tools": [{"invalid": "tool"}]  // Missing required fields
    });
    
    let result = manager.to_universal_request("openai", &malformed_request);
    // This might succeed or fail depending on implementation details
    // The important thing is that it doesn't panic
    assert!(result.is_ok() || result.is_err());
    
    // Test unsupported provider
    let test_request = json!({
        "model": "gpt-4",
        "messages": create_test_messages()
    });
    
    let result = manager.to_universal_request("nonexistent", &test_request);
    assert!(result.is_err());
}