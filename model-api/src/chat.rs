use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};

use crate::{errors::ApiReport, AppState, AppStateContents};

async fn list_chat_sessions(State(state): AppState) -> Result<impl IntoResponse, ApiReport> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}

async fn get_chat_session(
    State(state): AppState,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiReport> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}

async fn update_chat_session(
    State(state): AppState,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiReport> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}

async fn delete_chat_session(
    State(state): AppState,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiReport> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}

async fn add_chat_message(
    State(state): AppState,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiReport> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}

async fn new_chat_session(State(state): AppState) -> Result<impl IntoResponse, ApiReport> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}

pub fn create_router() -> Router<AppStateContents> {
    Router::new()
        .route("/", get(list_chat_sessions).post(new_chat_session))
        .route(
            "/:id",
            get(get_chat_session)
                .put(update_chat_session)
                .delete(delete_chat_session),
        )
        .route("/:id/add_message", post(add_chat_message))
}
