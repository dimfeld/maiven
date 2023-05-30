use axum::{extract::State, Json};
use maiven_search_store::{
    db::{models, DbError},
    models::ModelDefinition,
};
use serde::Serialize;

use crate::{errors::ReportError, AppState};

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
