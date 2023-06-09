use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};

use crate::{errors::ApiReport, AppState};

async fn lookup_by_hash() -> Result<impl IntoResponse, ApiReport> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}

async fn get_file_metadata() -> Result<impl IntoResponse, ApiReport> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}

async fn update_file_metadata() -> Result<impl IntoResponse, ApiReport> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}

async fn new_file() -> Result<impl IntoResponse, ApiReport> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}

async fn upload_file() -> Result<impl IntoResponse, ApiReport> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}

async fn delete_file() -> Result<impl IntoResponse, ApiReport> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/", post(new_file))
        .route("/by_hash", get(lookup_by_hash))
        .route(
            "/:id",
            get(get_file_metadata)
                .patch(update_file_metadata)
                .delete(delete_file),
        )
        .route("/:id/upload", post(upload_file))
}
