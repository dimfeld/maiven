use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
    Router,
};

use crate::{errors::ApiReport, AppState};

async fn list_chat_sessions(State(state): State<AppState>) -> Result<impl IntoResponse, ApiReport> {
    todo!();
    Ok(())
}

async fn get_chat_session(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiReport> {
    todo!();
    Ok(())
}

async fn add_chat_message(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiReport> {
    todo!();
    Ok(())
}

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_chat_sessions))
        .route("/:id", get(get_chat_session))
        .route("/:id/add_chat_message", post(add_chat_message))
}
