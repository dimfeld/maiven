use serde::{Deserialize, Serialize};

use sqlx_transparent_json_decode::sqlx_json_decode;

pub use self::error::ModelError;

pub mod bi_encoder;
pub mod chat;
pub mod completion;
pub mod download;
pub mod error;
mod ggml;
mod rust_bert_sentence_embeddings;
pub mod transformers;

#[derive(Serialize, Deserialize, Debug)]
pub struct ModelDefinition {
    pub id: i32,
    pub name: String,
    pub category: ModelCategory,
    pub params: ModelParams,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, sqlx::Type)]
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
    Ggml(GgmlModelParams),
    RustBert(ModelLocation),
}

sqlx_json_decode!(ModelParams);

pub struct LocationAndPattern {
    location: String,
    pattern: String,
}

impl ModelParams {
    pub fn location(&self) -> Option<&str> {
        match self {
            ModelParams::OpenaiChat => None,
            ModelParams::OpenaiCompletions => None,
            ModelParams::Ggml(GgmlModelParams { location, .. }) => Some(location),
            ModelParams::RustBert(ModelLocation { location }) => Some(location),
        }
    }

    pub fn additional_files(&self) -> Vec<LocationAndPattern> {
        match self {
            ModelParams::OpenaiChat => Vec::new(),
            ModelParams::OpenaiCompletions => Vec::new(),
            ModelParams::Ggml(GgmlModelParams { tokenizer, .. }) => match tokenizer {
                Some(tokenizer) => vec![LocationAndPattern {
                    location: tokenizer.clone(),
                    pattern: r##".*\.json$"##.to_string(),
                }],
                None => Vec::new(),
            },
            ModelParams::RustBert(_) => Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ModelLocation {
    pub location: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GgmlModelParams {
    pub model: String,
    pub location: String,
    pub tokenizer: Option<String>,
}

pub struct CrossEncoderModel {}
