pub mod form;
pub mod index;
pub mod model_table;
pub mod model_type;
use crate::types::ModelType;

fn model_type(model_type: ModelType) -> String {
    match model_type {
        ModelType::OpenAI => "OpenAI".to_string(),
        ModelType::Anthropic => "Anthropic".to_string(),
        ModelType::Local => "Local".to_string(),
        ModelType::Custom => "Custom".to_string(),
        ModelType::LLM => "LLM".to_string(),
        ModelType::Image => "Image".to_string(),
        ModelType::Embeddings => "Embeddings".to_string(),
        ModelType::TextToSpeech => "TextToSpeech".to_string(),
    }
}
