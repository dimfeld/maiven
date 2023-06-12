use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use error_stack::{ensure, IntoReport, ResultExt};
use maiven_search_store::{
    check_temperature,
    db::models,
    models::{
        chat::{ChatMessage, ChatRole, ChatSubmission},
        completion::CompletionSubmission,
        ModelDefinition,
    },
};
use serde::{Deserialize, Serialize};

use crate::{
    errors::{ApiError, ApiReport, ApiResult, IntoPassthrough, PassthroughReport, ReportError},
    AppState, AppStateContents,
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

async fn list_models(State(state): AppState) -> ApiResult<ModelsResult> {
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

async fn load_model(State(state): AppState, Path(id): Path<i32>) -> Result<(), ApiReport> {
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

#[derive(Serialize)]
struct CompletionResult {
    response: String,
}

async fn run_chat_model(
    State(state): AppState,
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

    check_temperature(&body.temperature)
        .change_context(ApiError::ArgError("temperature".to_string()))?;

    let mut messages = Vec::with_capacity(2);
    if let Some(system) = body.system {
        messages.push(ChatMessage {
            role: ChatRole::System,
            content: system,
            name: None,
        });
    }

    messages.push(ChatMessage {
        role: ChatRole::User,
        content: body.prompt,
        name: None,
    });

    let answer = model
        .chat(ChatSubmission {
            temperature: body.temperature,
            messages,
        })
        .passthrough_error()?;

    Ok(Json(ChatResult {
        response: answer.content,
    }))
}

async fn run_completion_model(
    State(state): AppState,
    Path(id): Path<i32>,
    Json(body): Json<CompletionSubmission>,
) -> ApiResult<CompletionResult> {
    let model = state
        .search_store
        .loaded_completion_models
        .read()
        .iter()
        .find(|model| model.id == id)
        .ok_or(ApiError::ModelNotLoaded("completion"))?
        .model
        .clone();

    check_temperature(&body.temperature)
        .change_context(ApiError::ArgError("temperature".to_string()))?;

    let answer = model.complete(body).passthrough_error()?;

    Ok(Json(CompletionResult { response: answer }))
}

pub fn create_router() -> Router<AppStateContents> {
    Router::new()
        .route("/", get(list_models))
        .route("/:id/load", post(load_model))
        .route("/:id/chat", post(run_chat_model))
        .route("/:id/complete", post(run_completion_model))
}
