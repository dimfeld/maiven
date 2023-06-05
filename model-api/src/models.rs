use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use error_stack::{IntoReport, ResultExt};
use maiven_search_store::{
    db::models,
    models::{
        chat::{ChatMessage, ChatRole, ChatSubmission},
        ModelDefinition,
    },
};
use serde::{Deserialize, Serialize};

use crate::{
    errors::{ApiError, ApiReport, ApiResult, IntoPassthrough, PassthroughReport, ReportError},
    AppState,
};

#[derive(Serialize)]
struct ModelInfo {
    #[serde(flatten)]
    model: ModelDefinition,
    loaded: bool,
}

#[derive(Serialize)]
struct ModelsResult {
    models: Vec<ModelInfo>,
}

async fn list_models(State(state): State<AppState>) -> ApiResult<ModelsResult> {
    let models = models::list_models(&state.pool)
        .await
        .change_context(ApiError::Passthrough)?
        .into_iter()
        .map(|model| ModelInfo {
            loaded: state.search_store.is_loaded(model.id),
            model,
        })
        .collect();

    Ok(Json(ModelsResult { models }))
}

async fn load_model(State(state): State<AppState>, Path(id): Path<i32>) -> Result<(), ApiReport> {
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

#[derive(Serialize)]
struct ChatResult {
    response: String,
}

#[derive(Deserialize)]
struct ChatBody {
    system: Option<String>,
    prompt: String,
    temperature: Option<f32>,
}

async fn run_chat_model(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(body): Json<ChatBody>,
) -> Result<Json<ChatResult>, ApiReport> {
    let model = state
        .search_store
        .loaded_chat_models
        .read()
        .iter()
        .find(|model| model.id == id)
        .ok_or(ApiError::ModelNotLoaded("chat"))?
        .model
        .clone();

    let answer = model
        .chat(ChatSubmission {
            temperature: body.temperature,
            messages: vec![ChatMessage {
                role: ChatRole::User,
                content: body.prompt,
                name: None,
            }],
        })
        .passthrough_error()?;

    Ok(Json(ChatResult {
        response: answer.content,
    }))
}

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_models))
        .route("/:id/load", post(load_model))
        .route("/:id/chat", post(run_chat_model))
}
