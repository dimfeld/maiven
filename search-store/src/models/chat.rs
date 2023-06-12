pub mod ggml_chat;
pub mod openai_chat;

use error_stack::Report;
use serde::{Deserialize, Serialize};

use super::{completion::CompletionModel, ModelError};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    System,
    User,
    Assistant,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
    pub name: Option<String>,
}

impl ChatMessage {
    pub fn format_chatml(&self) -> String {
        let role = match self.role {
            ChatRole::System => "system",
            ChatRole::User => "user",
            ChatRole::Assistant => "assistant",
        };

        match &self.name {
            Some(name) => format!("{} name={}\n{}", role, name, self.content),
            None => format!("{}\n{}", role, self.content),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ChatSubmission {
    pub messages: Vec<ChatMessage>,
    pub temperature: Option<f32>,
}

pub trait ChatModel: CompletionModel + Send + Sync {
    fn chat(&self, submission: ChatSubmission) -> Result<ChatMessage, Report<ModelError>>;
}
