use crate::transformers::types::{Transformer, TransformerConfig};
use crate::transformers::error::{TransformerError, TransformerResult};
use crate::transformers::providers::{OpenAITransformer, AnthropicTransformer, GeminiTransformer, ProviderTransformer};
use crate::transformers::providers::provider_trait::{ChatRequest, ChatResponse, ChatStreamChunk};
use std::collections::HashMap;
use serde_json::Value;

pub struct TransformerManager {
    openai: OpenAITransformer,
    anthropic: AnthropicTransformer,
    gemini: GeminiTransformer,
}

impl TransformerManager {
    pub fn new() -> Self {
        Self {
            openai: OpenAITransformer::new(),
            anthropic: AnthropicTransformer::new(),
            gemini: GeminiTransformer::new(),
        }
    }
    
    pub fn get_all_transformers(&self) -> HashMap<String, TransformerConfig> {
        let mut configs = HashMap::new();
        configs.insert("openai".to_string(), TransformerConfig {
            path: "transformers/providers/openai".to_string(),
            options: None,
        });
        configs.insert("anthropic".to_string(), TransformerConfig {
            path: "transformers/providers/anthropic".to_string(),
            options: None,
        });
        configs.insert("gemini".to_string(), TransformerConfig {
            path: "transformers/providers/gemini".to_string(),
            options: None,
        });
        configs
    }

    pub fn apply_transformer(&self, transformer: &Transformer, data: &str) -> TransformerResult<String> {
        // Find the first available transformer for the requested providers
        for provider_name in &transformer.use_transformers {
            match provider_name.as_str() {
                "openai" => {
                    // Parse the input data
                    let input_value: Value = serde_json::from_str(data)
                        .map_err(|e| TransformerError::Deserialization(e.to_string()))?;
                    
                    // Convert to universal format and back to provider format
                    let universal_request = self.openai.to_universal_request(&input_value)?;
                    let provider_request = self.openai.from_universal_request(&universal_request)?;
                    
                    return serde_json::to_string(&provider_request)
                        .map_err(|e| TransformerError::Serialization(e.to_string()));
                }
                "anthropic" => {
                    // Parse the input data
                    let input_value: Value = serde_json::from_str(data)
                        .map_err(|e| TransformerError::Deserialization(e.to_string()))?;
                    
                    // Convert to universal format and back to provider format
                    let universal_request = self.anthropic.to_universal_request(&input_value)?;
                    let provider_request = self.anthropic.from_universal_request(&universal_request)?;
                    
                    return serde_json::to_string(&provider_request)
                        .map_err(|e| TransformerError::Serialization(e.to_string()));
                }
                "gemini" => {
                    // Parse the input data
                    let input_value: Value = serde_json::from_str(data)
                        .map_err(|e| TransformerError::Deserialization(e.to_string()))?;
                    
                    // Convert to universal format and back to provider format
                    let universal_request = self.gemini.to_universal_request(&input_value)?;
                    let provider_request = self.gemini.from_universal_request(&universal_request)?;
                    
                    return serde_json::to_string(&provider_request)
                        .map_err(|e| TransformerError::Serialization(e.to_string()));
                }
                _ => continue,
            }
        }
        
        Err(TransformerError::UnsupportedProvider(
            transformer.use_transformers.join(", ")
        ))
    }

    pub fn transform_request(
        &self, 
        from_provider: &str, 
        to_provider: &str, 
        request: &Value
    ) -> TransformerResult<Value> {
        // Convert from source provider to universal format
        let universal_request = match from_provider {
            "openai" => self.openai.to_universal_request(request)?,
            "anthropic" => self.anthropic.to_universal_request(request)?,
            "gemini" => self.gemini.to_universal_request(request)?,
            _ => return Err(TransformerError::UnsupportedProvider(from_provider.to_string())),
        };
        
        // Convert from universal format to target provider
        match to_provider {
            "openai" => self.openai.from_universal_request(&universal_request),
            "anthropic" => self.anthropic.from_universal_request(&universal_request),
            "gemini" => self.gemini.from_universal_request(&universal_request),
            _ => Err(TransformerError::UnsupportedProvider(to_provider.to_string())),
        }
    }

    pub fn transform_response(
        &self, 
        from_provider: &str, 
        to_provider: &str, 
        response: &Value
    ) -> TransformerResult<Value> {
        // Convert from source provider to universal format
        let universal_response = match from_provider {
            "openai" => self.openai.to_universal_response(response)?,
            "anthropic" => self.anthropic.to_universal_response(response)?,
            "gemini" => self.gemini.to_universal_response(response)?,
            _ => return Err(TransformerError::UnsupportedProvider(from_provider.to_string())),
        };
        
        // Convert from universal format to target provider
        match to_provider {
            "openai" => self.openai.from_universal_response(&universal_response),
            "anthropic" => self.anthropic.from_universal_response(&universal_response),
            "gemini" => self.gemini.from_universal_response(&universal_response),
            _ => Err(TransformerError::UnsupportedProvider(to_provider.to_string())),
        }
    }

    pub fn transform_stream_chunk(
        &self, 
        from_provider: &str, 
        to_provider: &str, 
        chunk: &Value
    ) -> TransformerResult<Value> {
        // Convert from source provider to universal format
        let universal_chunk = match from_provider {
            "openai" => self.openai.to_universal_stream_chunk(chunk)?,
            "anthropic" => self.anthropic.to_universal_stream_chunk(chunk)?,
            "gemini" => self.gemini.to_universal_stream_chunk(chunk)?,
            _ => return Err(TransformerError::UnsupportedProvider(from_provider.to_string())),
        };
        
        // Convert from universal format to target provider
        match to_provider {
            "openai" => self.openai.from_universal_stream_chunk(&universal_chunk),
            "anthropic" => self.anthropic.from_universal_stream_chunk(&universal_chunk),
            "gemini" => self.gemini.from_universal_stream_chunk(&universal_chunk),
            _ => Err(TransformerError::UnsupportedProvider(to_provider.to_string())),
        }
    }

    pub fn list_available_providers(&self) -> Vec<String> {
        vec!["openai".to_string(), "anthropic".to_string(), "gemini".to_string()]
    }

    pub fn is_provider_supported(&self, provider: &str) -> bool {
        matches!(provider, "openai" | "anthropic" | "gemini")
    }

    pub fn to_universal_request(&self, from_provider: &str, request: &Value) -> TransformerResult<ChatRequest> {
        match from_provider {
            "openai" => self.openai.to_universal_request(request),
            "anthropic" => self.anthropic.to_universal_request(request),
            "gemini" => self.gemini.to_universal_request(request),
            _ => Err(TransformerError::UnsupportedProvider(from_provider.to_string())),
        }
    }

    pub fn from_universal_request(&self, to_provider: &str, universal_request: &ChatRequest) -> TransformerResult<Value> {
        match to_provider {
            "openai" => self.openai.from_universal_request(universal_request),
            "anthropic" => self.anthropic.from_universal_request(universal_request),
            "gemini" => self.gemini.from_universal_request(universal_request),
            _ => Err(TransformerError::UnsupportedProvider(to_provider.to_string())),
        }
    }

    pub fn to_universal_response(&self, from_provider: &str, response: &Value) -> TransformerResult<ChatResponse> {
        match from_provider {
            "openai" => self.openai.to_universal_response(response),
            "anthropic" => self.anthropic.to_universal_response(response),
            "gemini" => self.gemini.to_universal_response(response),
            _ => Err(TransformerError::UnsupportedProvider(from_provider.to_string())),
        }
    }

    pub fn from_universal_response(&self, to_provider: &str, universal_response: &ChatResponse) -> TransformerResult<Value> {
        match to_provider {
            "openai" => self.openai.from_universal_response(universal_response),
            "anthropic" => self.anthropic.from_universal_response(universal_response),
            "gemini" => self.gemini.from_universal_response(universal_response),
            _ => Err(TransformerError::UnsupportedProvider(to_provider.to_string())),
        }
    }

    pub fn to_universal_stream_chunk(&self, from_provider: &str, chunk: &Value) -> TransformerResult<ChatStreamChunk> {
        match from_provider {
            "openai" => self.openai.to_universal_stream_chunk(chunk),
            "anthropic" => self.anthropic.to_universal_stream_chunk(chunk),
            "gemini" => self.gemini.to_universal_stream_chunk(chunk),
            _ => Err(TransformerError::UnsupportedProvider(from_provider.to_string())),
        }
    }

    pub fn from_universal_stream_chunk(&self, to_provider: &str, universal_chunk: &ChatStreamChunk) -> TransformerResult<Value> {
        match to_provider {
            "openai" => self.openai.from_universal_stream_chunk(universal_chunk),
            "anthropic" => self.anthropic.from_universal_stream_chunk(universal_chunk),
            "gemini" => self.gemini.from_universal_stream_chunk(universal_chunk),
            _ => Err(TransformerError::UnsupportedProvider(to_provider.to_string())),
        }
    }
}

impl Default for TransformerManager {
    fn default() -> Self {
        Self::new()
    }
}