use serde::{Deserialize, Serialize};

pub use self::error::ModelError;

pub mod bi_encoder;
mod download;
pub mod error;
mod rust_bert_sentence_embeddings;
pub mod transformers;

#[derive(Serialize, Deserialize, Debug)]
pub struct ModelDefinition {
    pub name: String,
    pub category: ModelCategory,
    pub params: ModelParams,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum ModelCategory {
    Chat,
    Instruct,
    Complete,
    CrossEncoder,
    BiEncoder,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "code", rename_all = "kebab-case")]
pub enum ModelParams {
    OpenaiChat,
    OpenaiCompletions,
    Ggml(ModelTypeAndLocation),
    RustBert(ModelLocation),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ModelLocation {
    location: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ModelTypeAndLocation {
    model: String,
    location: String,
}

pub struct ChatModel {}

pub struct CompletionModel {}

/// Use a chat or completion model in a generic way.
pub trait TextGenerationModel {
    // TODO not infallible
    /// Generate text from a prompt.
    fn generate(&self, prompt: &str) -> Result<String, ModelError>;
}

pub struct CrossEncoderModel {}
