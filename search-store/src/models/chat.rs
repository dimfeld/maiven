pub mod ggml_chat;
pub mod openai_chat;

use error_stack::Report;
use serde::{Deserialize, Serialize};

use super::ModelError;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    System,
    User,
    Assistant,
}

#[derive(Serialize, Deserialize)]
pub struct ChatMessage {
    role: ChatRole,
    content: String,
    name: Option<String>,
}

impl ChatMessage {
    pub fn format_chatml(&self) -> String {
        let role = match self.role {
            ChatRole::System => "system",
            ChatRole::User => "user",
            ChatRole::Assistant => "assistant",
        };

        match &self.name {
            Some(name) => format!("{} name={}\n{}\n", role, name, self.content),
            None => format!("{}\n{}\n", role, self.content),
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct ChatSubmission {
    messages: Vec<ChatMessage>,
    temperature: Option<f32>,
}

pub trait ChatModel {
    fn chat(&self, submission: ChatSubmission) -> Result<ChatMessage, Report<ModelError>>;
}
