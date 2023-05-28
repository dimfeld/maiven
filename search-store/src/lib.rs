pub mod db;
pub mod models;

use models::{
    bi_encoder::BiEncoderModel,
    download::{DownloadError, ModelCache},
    ChatModel, CompletionModel, CrossEncoderModel, ModelDefinition,
};
use sqlx::PgPool;

pub struct SearchStore {
    pg: PgPool,
    model_cache: ModelCache,

    active_cross_encoder: Option<CrossEncoderModel>,
    active_bi_encoder: Option<BiEncoderModel>,
    active_completion: Option<CompletionModel>,
    active_chat: Option<ChatModel>,
}

impl SearchStore {
    pub fn new(pg: PgPool, model_cache: ModelCache) -> Self {
        Self {
            pg,
            model_cache,
            active_cross_encoder: None,
            active_bi_encoder: None,
            active_completion: None,
            active_chat: None,
        }
    }

    pub fn load_chat_model(&mut self, model: ModelDefinition) -> Result<(), DownloadError> {
        self.active_completion = None;
        todo!()
    }

    pub fn load_completion_model(&mut self, model: ModelDefinition) -> Result<(), DownloadError> {
        self.active_chat = None;
        todo!()
    }
}

// load models
