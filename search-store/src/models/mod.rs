use serde::{Deserialize, Serialize};

use sqlx_transparent_json_decode::sqlx_json_decode;

pub use self::error::ModelError;

pub mod bi_encoder;
pub mod download;
pub mod error;
mod rust_bert_sentence_embeddings;
pub mod transformers;

#[derive(Serialize, Deserialize, Debug)]
pub struct ModelDefinition {
    pub id: i32,
    pub name: String,
    pub category: ModelCategory,
    pub params: ModelParams,
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type)]
#[serde(rename_all = "kebab-case")]
#[sqlx(rename_all = "kebab-case")]
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

sqlx_json_decode!(ModelParams);

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

/// Both instruct and complete models fall under `CompletionModel`. The difference is only to inform the
/// caller in how to prompt the model.
pub struct CompletionModel {}

/// Use a chat or completion model in a generic way.
pub trait TextGenerationModel {
    // TODO not infallible
    /// Generate text from a prompt.
    fn generate(&self, prompt: &str) -> Result<String, ModelError>;
}

pub struct CrossEncoderModel {}