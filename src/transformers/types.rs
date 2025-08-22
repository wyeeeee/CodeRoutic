use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformerConfig {
    pub path: String,
    pub options: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transformer {
    #[serde(rename = "use")]
    pub use_transformers: Vec<String>,
    #[serde(flatten)]
    pub model_specific: HashMap<String, ModelTransformer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelTransformer {
    #[serde(rename = "use")]
    pub use_transformers: Vec<String>,
}