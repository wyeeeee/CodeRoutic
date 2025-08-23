use crate::transformers::types::{Transformer, TransformerConfig};
use crate::transformers::providers::TransformerRegistry;
use crate::transformers::error::{TransformerError, TransformerResult};
use std::collections::HashMap;
use serde_json::Value;

pub struct TransformerManager {
    registry: TransformerRegistry,
}

impl TransformerManager {
    pub fn new() -> Self {
        Self {
            registry: TransformerRegistry::new(),
        }
    }

    pub fn with_custom_registry(registry: TransformerRegistry) -> Self {
        Self { registry }
    }
    
    pub fn get_all_transformers(&self) -> HashMap<String, TransformerConfig> {
        // Return available transformer configurations
        let mut configs = HashMap::new();
        
        for provider in self.registry.list_available_providers() {
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
            if let Ok(transformer_impl) = self.registry.get_transformer(provider_name) {
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
        let from_transformer = self.registry.get_transformer(from_provider)?;
        let to_transformer = self.registry.get_transformer(to_provider)?;
        
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
        let from_transformer = self.registry.get_transformer(from_provider)?;
        let to_transformer = self.registry.get_transformer(to_provider)?;
        
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
        let from_transformer = self.registry.get_transformer(from_provider)?;
        let to_transformer = self.registry.get_transformer(to_provider)?;
        
        // Convert from source provider to universal format
        let universal_chunk = from_transformer.to_universal_stream_chunk(chunk)?;
        
        // Convert from universal format to target provider
        let target_chunk = to_transformer.from_universal_stream_chunk(&universal_chunk)?;
        
        Ok(target_chunk)
    }

    pub fn list_available_providers(&self) -> Vec<String> {
        self.registry.list_available_providers()
    }

    pub fn is_provider_supported(&self, provider: &str) -> bool {
        self.registry.is_provider_supported(provider)
    }

    pub fn register_custom_transformer<T: crate::transformers::provider_trait::ProviderTransformer + 'static>(
        &mut self,
        name: String,
        transformer: T,
    ) {
        self.registry.register_custom_transformer(name, transformer);
    }
}

impl Default for TransformerManager {
    fn default() -> Self {
        Self::new()
    }
}