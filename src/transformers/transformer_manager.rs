use crate::transformers::types::{Transformer, TransformerConfig};
use crate::transformers::error::{TransformerError, TransformerResult};
use crate::transformers::providers::{ProviderTransformer, OpenAITransformer, AnthropicTransformer, GeminiTransformer};
use crate::transformers::providers::provider_trait::{ChatRequest, ChatResponse, ChatStreamChunk};
use std::collections::HashMap;
use std::sync::Arc;
use serde_json::Value;

pub struct TransformerManager {
    transformers: HashMap<String, Arc<dyn ProviderTransformer>>,
}

impl TransformerManager {
    pub fn new() -> Self {
        let mut manager = Self {
            transformers: HashMap::new(),
        };
        
        manager.register_default_transformers();
        manager
    }

    fn register_default_transformers(&mut self) {
        self.transformers.insert("openai".to_string(), Arc::new(OpenAITransformer::new()));
        self.transformers.insert("anthropic".to_string(), Arc::new(AnthropicTransformer::new()));
        self.transformers.insert("gemini".to_string(), Arc::new(GeminiTransformer::new()));
    }
    
    pub fn get_all_transformers(&self) -> HashMap<String, TransformerConfig> {
        // Return available transformer configurations
        let mut configs = HashMap::new();
        
        for provider in self.transformers.keys() {
            configs.insert(provider.clone(), TransformerConfig {
                path: format!("transformers/providers/{}", provider),
                options: None,
            });
        }
        
        configs
    }

    pub fn apply_transformer(&self, transformer: &Transformer, data: &str) -> TransformerResult<String> {
        // Find the first available transformer for the requested providers
        for provider_name in &transformer.use_transformers {
            if let Some(transformer_impl) = self.transformers.get(provider_name) {
                // Parse the input data
                let input_value: Value = serde_json::from_str(data)
                    .map_err(|e| TransformerError::Deserialization(e.to_string()))?;
                
                // Convert to universal format and back to provider format
                let universal_request = transformer_impl.to_universal_request(&input_value)?;
                let provider_request = transformer_impl.from_universal_request(&universal_request)?;
                
                return serde_json::to_string(&provider_request)
                    .map_err(|e| TransformerError::Serialization(e.to_string()));
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
        let from_transformer = self.get_transformer(from_provider)?;
        let to_transformer = self.get_transformer(to_provider)?;
        
        // Convert from source provider to universal format
        let universal_request = from_transformer.to_universal_request(request)?;
        
        // Convert from universal format to target provider
        let target_request = to_transformer.from_universal_request(&universal_request)?;
        
        Ok(target_request)
    }

    pub fn transform_response(
        &self, 
        from_provider: &str, 
        to_provider: &str, 
        response: &Value
    ) -> TransformerResult<Value> {
        let from_transformer = self.get_transformer(from_provider)?;
        let to_transformer = self.get_transformer(to_provider)?;
        
        // Convert from source provider to universal format
        let universal_response = from_transformer.to_universal_response(response)?;
        
        // Convert from universal format to target provider
        let target_response = to_transformer.from_universal_response(&universal_response)?;
        
        Ok(target_response)
    }

    pub fn transform_stream_chunk(
        &self, 
        from_provider: &str, 
        to_provider: &str, 
        chunk: &Value
    ) -> TransformerResult<Value> {
        let from_transformer = self.get_transformer(from_provider)?;
        let to_transformer = self.get_transformer(to_provider)?;
        
        // Convert from source provider to universal format
        let universal_chunk = from_transformer.to_universal_stream_chunk(chunk)?;
        
        // Convert from universal format to target provider
        let target_chunk = to_transformer.from_universal_stream_chunk(&universal_chunk)?;
        
        Ok(target_chunk)
    }

    pub fn list_available_providers(&self) -> Vec<String> {
        self.transformers.keys().cloned().collect()
    }

    pub fn is_provider_supported(&self, provider: &str) -> bool {
        self.transformers.contains_key(provider)
    }

    pub fn register_custom_transformer<T: ProviderTransformer + 'static>(
        &mut self,
        name: String,
        transformer: T,
    ) {
        self.transformers.insert(name, Arc::new(transformer));
    }

    fn get_transformer(&self, provider: &str) -> TransformerResult<Arc<dyn ProviderTransformer>> {
        self.transformers
            .get(provider)
            .cloned()
            .ok_or_else(|| TransformerError::UnsupportedProvider(provider.to_string()))
    }

    pub fn to_universal_request(&self, from_provider: &str, request: &Value) -> TransformerResult<ChatRequest> {
        let transformer = self.get_transformer(from_provider)?;
        transformer.to_universal_request(request)
    }

    pub fn from_universal_request(&self, to_provider: &str, universal_request: &ChatRequest) -> TransformerResult<Value> {
        let transformer = self.get_transformer(to_provider)?;
        transformer.from_universal_request(universal_request)
    }

    pub fn to_universal_response(&self, from_provider: &str, response: &Value) -> TransformerResult<ChatResponse> {
        let transformer = self.get_transformer(from_provider)?;
        transformer.to_universal_response(response)
    }

    pub fn from_universal_response(&self, to_provider: &str, universal_response: &ChatResponse) -> TransformerResult<Value> {
        let transformer = self.get_transformer(to_provider)?;
        transformer.from_universal_response(universal_response)
    }

    pub fn to_universal_stream_chunk(&self, from_provider: &str, chunk: &Value) -> TransformerResult<ChatStreamChunk> {
        let transformer = self.get_transformer(from_provider)?;
        transformer.to_universal_stream_chunk(chunk)
    }

    pub fn from_universal_stream_chunk(&self, to_provider: &str, universal_chunk: &ChatStreamChunk) -> TransformerResult<Value> {
        let transformer = self.get_transformer(to_provider)?;
        transformer.from_universal_stream_chunk(universal_chunk)
    }
}

impl Default for TransformerManager {
    fn default() -> Self {
        Self::new()
    }
}