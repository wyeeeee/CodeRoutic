use std::collections::HashMap;
use std::sync::Arc;
use crate::transformers::error::{TransformerError, TransformerResult};
use crate::transformers::provider_trait::ProviderTransformer;

use super::openai::OpenAITransformer;
use super::anthropic::AnthropicTransformer;
use super::gemini::GeminiTransformer;

pub type TransformerRef = Arc<dyn ProviderTransformer>;

pub struct TransformerFactory {
    transformers: HashMap<String, TransformerRef>,
}

impl TransformerFactory {
    pub fn new() -> Self {
        let mut factory = Self {
            transformers: HashMap::new(),
        };
        
        factory.register_default_transformers();
        factory
    }

    pub fn register_transformer<T: ProviderTransformer + 'static>(
        &mut self,
        name: String,
        transformer: T,
    ) {
        self.transformers.insert(name, Arc::new(transformer));
    }

    pub fn get_transformer(&self, name: &str) -> TransformerResult<TransformerRef> {
        self.transformers
            .get(name)
            .cloned()
            .ok_or_else(|| TransformerError::UnsupportedProvider(name.to_string()))
    }

    pub fn list_providers(&self) -> Vec<String> {
        self.transformers.keys().cloned().collect()
    }

    pub fn has_provider(&self, name: &str) -> bool {
        self.transformers.contains_key(name)
    }

    fn register_default_transformers(&mut self) {
        self.register_transformer("openai".to_string(), OpenAITransformer::new());
        self.register_transformer("anthropic".to_string(), AnthropicTransformer::new());
        self.register_transformer("gemini".to_string(), GeminiTransformer::new());
    }
}

impl Default for TransformerFactory {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TransformerRegistry {
    factory: TransformerFactory,
}

impl TransformerRegistry {
    pub fn new() -> Self {
        Self {
            factory: TransformerFactory::new(),
        }
    }

    pub fn get_transformer(&self, provider: &str) -> TransformerResult<TransformerRef> {
        self.factory.get_transformer(provider)
    }

    pub fn register_custom_transformer<T: ProviderTransformer + 'static>(
        &mut self,
        name: String,
        transformer: T,
    ) {
        self.factory.register_transformer(name, transformer);
    }

    pub fn list_available_providers(&self) -> Vec<String> {
        self.factory.list_providers()
    }

    pub fn is_provider_supported(&self, provider: &str) -> bool {
        self.factory.has_provider(provider)
    }
}

impl Default for TransformerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factory_creation() {
        let factory = TransformerFactory::new();
        assert!(factory.has_provider("openai"));
        assert!(factory.has_provider("anthropic"));
        assert!(factory.has_provider("gemini"));
        assert!(!factory.has_provider("nonexistent"));
    }

    #[test]
    fn test_get_transformer() {
        let factory = TransformerFactory::new();
        let openai_transformer = factory.get_transformer("openai");
        assert!(openai_transformer.is_ok());
        
        let nonexistent = factory.get_transformer("nonexistent");
        assert!(nonexistent.is_err());
    }

    #[test]
    fn test_list_providers() {
        let factory = TransformerFactory::new();
        let providers = factory.list_providers();
        assert!(providers.contains(&"openai".to_string()));
        assert!(providers.contains(&"anthropic".to_string()));
        assert!(providers.contains(&"gemini".to_string()));
    }

    #[test]
    fn test_registry() {
        let registry = TransformerRegistry::new();
        assert!(registry.is_provider_supported("openai"));
        assert!(!registry.is_provider_supported("nonexistent"));
        
        let providers = registry.list_available_providers();
        assert!(providers.len() >= 3);
    }
}