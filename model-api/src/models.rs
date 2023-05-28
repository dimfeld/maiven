use axum::Json;
use maiven_search_store::{db::models, models::ModelDefinition};

pub struct ModelsResult {
    models: Vec<ModelDefinition>,
}

pub async fn list_models() -> Result<Json<Vec<ModelDefinition>>, axum::http::StatusCode> {
    todo!()
    // let models = models::list_models().await?;

    // Ok(Json(ModelsResult { models }))
}
