pub mod db;
pub mod models;

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use error_stack::{IntoReport, Report, ResultExt};
use models::{
    bi_encoder::BiEncoderModel,
    chat::ChatModel,
    download::{DownloadError, ModelCache},
    CompletionModel, CrossEncoderModel, ModelDefinition, ModelError,
};
use parking_lot::Mutex;
use sqlx::PgPool;

use crate::models::{chat::ggml_chat::GgmlChatModel, ModelParams, ModelTypeAndLocation};

pub struct LoadedModel<T: ?Sized> {
    pub id: i32,
    pub model: Arc<T>,
}

pub struct SearchStore {
    pg: PgPool,
    model_cache: ModelCache,

    pub loaded_chat_models: Mutex<Vec<LoadedModel<dyn ChatModel>>>,
    pub loaded_completion_models: Mutex<Vec<LoadedModel<CompletionModel>>>,

    loaded_bi_encoders: Mutex<Vec<LoadedModel<BiEncoderModel>>>,
    loaded_cross_encoders: Mutex<Vec<LoadedModel<CrossEncoderModel>>>,
}

impl SearchStore {
    pub fn new(pg: PgPool, model_cache: ModelCache) -> Self {
        Self {
            pg,
            model_cache,
            loaded_chat_models: Mutex::new(Vec::new()),
            loaded_completion_models: Mutex::new(Vec::new()),
            loaded_bi_encoders: Mutex::new(Vec::new()),
            loaded_cross_encoders: Mutex::new(Vec::new()),
        }
    }

    pub fn load_model(&self, model: &ModelDefinition) -> Result<(), Report<ModelError>> {
        let weights_path = model
            .params
            .location()
            .map(|location| self.model_cache.download_if_needed(location))
            .transpose()
            .change_context(ModelError::LoadingError)?;

        match model.category {
            models::ModelCategory::Chat => self.load_chat_model(model, weights_path),
            models::ModelCategory::Complete | models::ModelCategory::Instruct => {
                self.load_completion_model(model, weights_path)
            }
            models::ModelCategory::BiEncoder => self.load_bi_encoder_model(model, weights_path),
            models::ModelCategory::CrossEncoder => {
                self.load_cross_encoder_model(model, weights_path)
            }
        }
    }

    fn load_chat_model(
        &self,
        model: &ModelDefinition,
        weights_path: Option<PathBuf>,
    ) -> Result<(), Report<ModelError>> {
        assert_eq!(model.category, models::ModelCategory::Chat);

        let loaded = match &model.params {
            ModelParams::OpenaiChat | ModelParams::OpenaiCompletions => {
                todo!()
            }
            ModelParams::Ggml(ModelTypeAndLocation {
                model: model_name, ..
            }) => {
                let weights_path = weights_path
                    .ok_or(ModelError::LoadingError)
                    .into_report()
                    .attach_printable("No path provided for GGML model")?;

                Arc::new(GgmlChatModel::new(model_name, &weights_path)?)
            }
            ModelParams::RustBert(location) => todo!(),
        };

        self.loaded_chat_models.lock().push(LoadedModel {
            id: model.id,
            model: loaded,
        });

        Ok(())
    }

    fn load_completion_model(
        &self,
        model: &ModelDefinition,
        weights_path: Option<PathBuf>,
    ) -> Result<(), Report<ModelError>> {
        assert_eq!(model.category, models::ModelCategory::Complete);
        todo!()
    }

    fn load_bi_encoder_model(
        &self,
        model: &ModelDefinition,
        weights_path: Option<PathBuf>,
    ) -> Result<(), Report<ModelError>> {
        assert_eq!(model.category, models::ModelCategory::BiEncoder);
        todo!()
    }

    fn load_cross_encoder_model(
        &self,
        model: &ModelDefinition,
        weights_path: Option<PathBuf>,
    ) -> Result<(), Report<ModelError>> {
        assert_eq!(model.category, models::ModelCategory::CrossEncoder);
        todo!()
    }

    /// A quick lookup for if a particular model is loaded.
    pub fn is_loaded(&self, model_id: i32) -> bool {
        self.loaded_chat_models
            .lock()
            .iter()
            .any(|m| m.id == model_id)
            || self
                .loaded_completion_models
                .lock()
                .iter()
                .any(|m| m.id == model_id)
            || self
                .loaded_bi_encoders
                .lock()
                .iter()
                .any(|m| m.id == model_id)
            || self
                .loaded_cross_encoders
                .lock()
                .iter()
                .any(|m| m.id == model_id)
    }
}

// load models
