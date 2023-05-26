use std::convert::Infallible;

pub use self::error::ModelError;

pub mod bi_encoder;
mod download;
pub mod error;
mod rust_bert_sentence_embeddings;
pub mod transformers;

pub struct ChatModel {}

pub struct CompletionModel {}

/// Use a chat or completion model in a generic way.
pub trait TextGenerationModel {
    // TODO not infallible
    /// Generate text from a prompt.
    fn generate(&self, prompt: &str) -> Result<String, ModelError>;
}

pub struct CrossEncoderModel {}
