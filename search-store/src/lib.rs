pub mod db;
pub mod models;

use std::path::{Path, PathBuf};

use error_stack::{IntoReport, Report, ResultExt};
use models::{
    bi_encoder::BiEncoderModel,
    chat::ChatModel,
    download::{DownloadError, ModelCache},
    CompletionModel, CrossEncoderModel, ModelDefinition, ModelError,
};
use sqlx::PgPool;

use crate::models::{chat::ggml_chat::GgmlChatModel, ModelParams, ModelTypeAndLocation};

pub struct LoadedModel<T> {
    pub id: i32,
    pub model: T,
}

pub struct SearchStore {
    pg: PgPool,
    model_cache: ModelCache,

    pub loaded_chat_models: Vec<LoadedModel<Box<dyn ChatModel>>>,
    pub loaded_completion_models: Vec<LoadedModel<CompletionModel>>,

    loaded_bi_encoders: Vec<LoadedModel<BiEncoderModel>>,
    loaded_cross_encoders: Vec<LoadedModel<CrossEncoderModel>>,
}

impl SearchStore {
    pub fn new(pg: PgPool, model_cache: ModelCache) -> Self {
        Self {
            pg,
            model_cache,
            loaded_chat_models: Vec::new(),
            loaded_completion_models: Vec::new(),
            loaded_bi_encoders: Vec::new(),
            loaded_cross_encoders: Vec::new(),
        }
    }

    pub fn load_model(&mut self, model: &ModelDefinition) -> Result<(), Report<ModelError>> {
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
        &mut self,
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

                Box::new(GgmlChatModel::new(model_name, &weights_path)?)
            }
            ModelParams::RustBert(location) => todo!(),
        };

        self.loaded_chat_models.push(LoadedModel {
            id: model.id,
            model: loaded,
        });

        Ok(())
    }

    fn load_completion_model(
        &mut self,
        model: &ModelDefinition,
        weights_path: Option<PathBuf>,
    ) -> Result<(), Report<ModelError>> {
        assert_eq!(model.category, models::ModelCategory::Complete);
        todo!()
    }

    fn load_bi_encoder_model(
        &mut self,
        model: &ModelDefinition,
        weights_path: Option<PathBuf>,
    ) -> Result<(), Report<ModelError>> {
        assert_eq!(model.category, models::ModelCategory::BiEncoder);
        todo!()
    }

    fn load_cross_encoder_model(
        &mut self,
        model: &ModelDefinition,
        weights_path: Option<PathBuf>,
    ) -> Result<(), Report<ModelError>> {
        assert_eq!(model.category, models::ModelCategory::CrossEncoder);
        todo!()
    }

    /// A quick lookup for if a particular model is loaded.
    pub fn is_loaded(&self, model_id: i32) -> bool {
        self.loaded_chat_models.iter().any(|m| m.id == model_id)
            || self
                .loaded_completion_models
                .iter()
                .any(|m| m.id == model_id)
            || self.loaded_bi_encoders.iter().any(|m| m.id == model_id)
            || self.loaded_cross_encoders.iter().any(|m| m.id == model_id)
    }
}

// load models
