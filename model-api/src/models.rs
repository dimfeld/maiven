use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use error_stack::{IntoReport, ResultExt};
use maiven_search_store::{
    db::{models, DbError},
    models::ModelDefinition,
};
use serde::Serialize;

use crate::{
    errors::{ApiError, ApiReport, PassthroughResult, ReportError},
    AppState,
};

#[derive(Serialize)]
pub struct ModelInfo {
    #[serde(flatten)]
    model: ModelDefinition,
    loaded: bool,
}

#[derive(Serialize)]
pub struct ModelsResult {
    models: Vec<ModelInfo>,
}

pub async fn list_models(
    State(state): State<AppState>,
) -> Result<Json<ModelsResult>, ReportError<DbError>> {
    let models = models::list_models(&state.pool)
        .await?
        .into_iter()
        .map(|model| ModelInfo {
            loaded: state.search_store.is_loaded(model.id),
            model,
        })
        .collect();

    Ok(Json(ModelsResult { models }))
}

pub async fn load_model(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<(), ApiReport> {
    let model = models::list_models(&state.pool)
        .await
        .change_context(ApiError::Passthrough)?
        .into_iter()
        .find(|m| m.id == id)
        .ok_or(ApiError::NotFound)
        .into_report()?;

    if !state.search_store.is_loaded(model.id) {
        tokio::task::spawn_blocking(move || {
            state
                .search_store
                .load_model(&model)
                .change_context(ApiError::Passthrough)
        })
        .await
        .passthrough_error()??;
    }

    Ok(())
}

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_models))
        .route("/:id/load", post(load_model))
}
