use axum::{extract::State, Json};
use maiven_search_store::{
    db::{models, DbError},
    models::ModelDefinition,
};
use serde::Serialize;

use crate::{errors::ReportError, AppState};

#[derive(Serialize)]
pub struct ModelsResult {
    models: Vec<ModelDefinition>,
}

pub async fn list_models(
    State(state): State<AppState>,
) -> Result<Json<ModelsResult>, ReportError<DbError>> {
    let models = models::list_models(&state.pool).await?;

    Ok(Json(ModelsResult { models }))
}
