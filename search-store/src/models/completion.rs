pub mod ggml_completion;

use error_stack::Report;
use serde::{Deserialize, Serialize};

use super::ModelError;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct CompletionSubmission {
    pub prompt: String,
    pub temperature: Option<f32>,
}

pub trait CompletionModel: Send + Sync {
    fn complete(&self, submission: CompletionSubmission) -> Result<String, Report<ModelError>>;
}
