mod ggml_chat;
mod openai_chat;

use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, Default)]
pub struct ChatSubmission {
    messages: Vec<ChatMessage>,
    temperature: Option<f32>,
}
