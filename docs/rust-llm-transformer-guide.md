# Rust LLM API 转换器实现指南

## 项目概述

本文档为在 Rust 中实现 LLM API 转换器提供详细的技术规范，涵盖 OpenAI、Anthropic 和 Gemini 三家提供商的格式转换。包括数据结构设计、流式处理、工具调用等核心功能的实现原理。

## 各提供商 API 格式分析

### OpenAI API 格式

#### 请求格式
```json
{
  "model": "gpt-4",
  "messages": [
    {
      "role": "system",
      "content": "You are a helpful assistant."
    },
    {
      "role": "user",
      "content": "Hello world"
    }
  ],
  "temperature": 0.7,
  "max_tokens": 1000,
  "stream": false,
  "tools": [
    {
      "type": "function",
      "function": {
        "name": "get_weather",
        "description": "Get weather information",
        "parameters": {
          "type": "object",
          "properties": {
            "location": {
              "type": "string",
              "description": "City name"
            }
          },
          "required": ["location"]
        }
      }
    }
  ],
  "tool_choice": "auto"
}
```

#### 响应格式（非流式）
```json
{
  "id": "chatcmpl-123",
  "object": "chat.completion",
  "created": 1677652288,
  "model": "gpt-4",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "Hello! How can I help you today?",
        "tool_calls": [
          {
            "id": "call_abc123",
            "type": "function",
            "function": {
              "name": "get_weather",
              "arguments": "{\"location\": \"Boston\"}"
            }
          }
        ]
      },
      "finish_reason": "tool_calls"
    }
  ],
  "usage": {
    "prompt_tokens": 82,
    "completion_tokens": 17,
    "total_tokens": 99
  }
}
```

#### 流式响应格式
```json
{
  "id": "chatcmpl-123",
  "object": "chat.completion.chunk",
  "created": 1677652288,
  "model": "gpt-4",
  "choices": [
    {
      "index": 0,
      "delta": {
        "role": "assistant",
        "content": "Hello"
      },
      "finish_reason": null
    }
  ]
}
```

### Anthropic API 格式

#### 请求格式
```json
{
  "model": "claude-3-sonnet-20240229",
  "max_tokens": 1000,
  "messages": [
    {
      "role": "user",
      "content": [
        {
          "type": "text",
          "text": "Hello world"
        }
      ]
    }
  ],
  "stream": false,
  "tools": [
    {
      "name": "get_weather",
      "description": "Get weather information",
      "input_schema": {
        "type": "object",
        "properties": {
          "location": {
            "type": "string",
            "description": "City name"
          }
        },
        "required": ["location"]
      }
    }
  ],
  "tool_choice": {
    "type": "auto"
  }
}
```

#### 响应格式（非流式）
```json
{
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
        "location": "Boston"
      }
    }
  ],
  "model": "claude-3-sonnet-20240229",
  "stop_reason": "tool_use",
  "usage": {
    "input_tokens": 82,
    "output_tokens": 17
  }
}
```

#### 流式响应格式
```json
{
  "type": "content_block_delta",
  "index": 0,
  "delta": {
    "type": "text_delta",
    "text": "Hello"
  }
}
```

### Gemini API 格式

#### 请求格式
```json
{
  "contents": [
    {
      "role": "user",
      "parts": [
        {
          "text": "Hello world"
        }
      ]
    }
  ],
  "generationConfig": {
    "temperature": 0.7,
    "maxOutputTokens": 1000
  },
  "tools": [
    {
      "function_declarations": [
        {
          "name": "get_weather",
          "description": "Get weather information",
          "parameters": {
            "type": "object",
            "properties": {
              "location": {
                "type": "string",
                "description": "City name"
              }
            },
            "required": ["location"]
          }
        }
      ]
    }
  ],
  "tool_config": {
    "function_calling_config": {
      "mode": "AUTO"
    }
  }
}
```

#### 响应格式（非流式）
```json
{
  "candidates": [
    {
      "content": {
        "parts": [
          {
            "text": "I'll help you get the weather information."
          },
          {
            "functionCall": {
              "name": "get_weather",
              "args": {
                "location": "Boston"
              }
            }
          }
        ],
        "role": "model"
      },
      "finishReason": "STOP",
      "index": 0
    }
  ],
  "usageMetadata": {
    "promptTokenCount": 82,
    "candidatesTokenCount": 17,
    "totalTokenCount": 99
  }
}
```

#### 流式响应格式
```json
{
  "candidates": [
    {
      "index": 0,
      "content": {
        "parts": [
          {
            "text": "Hello"
          }
        ],
        "role": "model"
      },
      "finishReason": null,
      "safetyRatings": []
    }
  ]
}
```