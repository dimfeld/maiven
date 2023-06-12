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
    completion::CompletionModel,
    download::{DownloadError, ModelCache},
    CrossEncoderModel, ModelDefinition, ModelError,
};
use parking_lot::RwLock;
use sqlx::PgPool;

use crate::models::{
    chat::ggml_chat::GgmlChatModel, completion::ggml_completion::GgmlCompletionModel, ModelParams,
    ModelTypeAndLocation,
};

pub struct LoadedModel<T: ?Sized> {
    pub id: i32,
    pub model: Arc<T>,
}

pub struct SearchStore {
    pg: PgPool,
    model_cache: ModelCache,

    pub file_storage_location: String,

    pub loaded_chat_models: RwLock<Vec<LoadedModel<dyn ChatModel>>>,
    pub loaded_completion_models: RwLock<Vec<LoadedModel<dyn CompletionModel>>>,

    loaded_bi_encoders: RwLock<Vec<LoadedModel<BiEncoderModel>>>,
    loaded_cross_encoders: RwLock<Vec<LoadedModel<CrossEncoderModel>>>,
}

impl SearchStore {
    pub fn new(pg: PgPool, file_storage_location: String, model_cache: ModelCache) -> Self {
        Self {
            pg,
            model_cache,
            file_storage_location,
            loaded_chat_models: RwLock::new(Vec::new()),
            loaded_completion_models: RwLock::new(Vec::new()),
            loaded_bi_encoders: RwLock::new(Vec::new()),
            loaded_cross_encoders: RwLock::new(Vec::new()),
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

                Arc::new(GgmlChatModel::new(
                    model.name.clone(),
                    model_name,
                    &weights_path,
                )?)
            }
            ModelParams::RustBert(location) => todo!(),
        };

        self.loaded_completion_models.write().push(LoadedModel {
            id: model.id,
            model: loaded.clone(),
        });

        self.loaded_chat_models.write().push(LoadedModel {
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
        assert!(
            model.category == models::ModelCategory::Chat
                || model.category == models::ModelCategory::Complete
                || model.category == models::ModelCategory::Instruct
        );

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

                Arc::new(GgmlCompletionModel::new(
                    model.name.clone(),
                    model_name,
                    &weights_path,
                )?)
            }
            ModelParams::RustBert(location) => todo!(),
        };

        self.loaded_completion_models.write().push(LoadedModel {
            id: model.id,
            model: loaded,
        });

        Ok(())
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
            .read()
            .iter()
            .any(|m| m.id == model_id)
            || self
                .loaded_completion_models
                .read()
                .iter()
                .any(|m| m.id == model_id)
            || self
                .loaded_bi_encoders
                .read()
                .iter()
                .any(|m| m.id == model_id)
            || self
                .loaded_cross_encoders
                .read()
                .iter()
                .any(|m| m.id == model_id)
    }
}

pub fn check_temperature(temperature: &Option<f32>) -> Result<(), Report<ModelError>> {
    if let Some(temp) = temperature {
        if *temp <= 0.0 {
            return Err(ModelError::ParameterError)
                .into_report()
                .attach_printable("Temperature must be greater than 0");
        } else if *temp > 2.0 {
            return Err(ModelError::ParameterError)
                .into_report()
                .attach_printable("Temperature must be less than 2");
        }
    }

    Ok(())
}
