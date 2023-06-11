use axum::{
    extract::{BodyStream, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use base64::Engine;
use error_stack::{IntoReport, ResultExt};
use futures::StreamExt;
use maiven_search_store::db::{self, items::ItemStatus};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;

use crate::{
    errors::{ApiError, ApiReport, ApiResult, IntoPassthrough},
    AppState, AppStateContents,
};

const BASE64_CONFIG: base64::engine::GeneralPurposeConfig =
    base64::engine::GeneralPurposeConfig::new()
        .with_encode_padding(true)
        .with_decode_padding_mode(base64::engine::DecodePaddingMode::Indifferent);
const BASE64_ENGINE: base64::engine::GeneralPurpose =
    base64::engine::GeneralPurpose::new(&base64::alphabet::URL_SAFE, BASE64_CONFIG);

#[derive(Deserialize, Debug)]
struct ItemPayload {
    pub source_id: i32,
    pub status: ItemStatus,
    pub content_type: String,
    pub external_id: String,
    pub original_location: Option<String>,
    pub tags: Vec<i32>,
    pub name: Option<String>,
    pub title: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub hidden: bool,
}

#[derive(Serialize, Debug)]
struct ItemResponse {
    pub id: i64,
    pub source_id: i32,
    pub status: ItemStatus,
    pub content_type: String,
    pub external_id: String,
    pub version: i32,
    pub hash: Option<String>,

    pub saved_original_path: Option<String>,
    pub original_location: Option<String>,

    pub tags: Vec<i32>,

    pub name: Option<String>,
    pub title: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub generated_summary: Option<String>,

    pub updated_at: time::OffsetDateTime,
    pub hidden: bool,
}

impl From<maiven_search_store::db::items::ItemMetadata> for ItemResponse {
    fn from(item: maiven_search_store::db::items::ItemMetadata) -> Self {
        Self {
            id: item.id,
            source_id: item.source_id,
            status: item.status,
            content_type: item.content_type,
            external_id: item.external_id,
            version: item.version,
            hash: item.hash.map(|hash| BASE64_ENGINE.encode(hash)),
            tags: item.tags,
            saved_original_path: item.saved_original_path,
            original_location: item.original_location,
            name: item.name,
            title: item.title,
            author: item.author,
            description: item.description,
            generated_summary: item.generated_summary,
            updated_at: item.updated_at,
            hidden: item.hidden,
        }
    }
}

async fn lookup_by_hash(
    State(state): AppState,
    Path(hash): Path<String>,
) -> ApiResult<ItemResponse> {
    let decoded = BASE64_ENGINE
        .decode(&hash)
        .into_report()
        .change_context_lazy(|| ApiError::ArgError("invalid hash format".to_string()))?;

    let item = maiven_search_store::db::items::lookup_by_hash(&state.pool, &decoded)
        .await?
        .ok_or(ApiError::NotFound)
        .map(ItemResponse::from)?;

    Ok(Json(item))
}

async fn lookup_by_external_id(
    State(state): AppState,
    Path(id): Path<String>,
) -> ApiResult<ItemResponse> {
    let item = maiven_search_store::db::items::lookup_by_external_id(&state.pool, &id)
        .await?
        .ok_or(ApiError::NotFound)
        .map(ItemResponse::from)?;

    Ok(Json(item))
}

async fn upsert_by_hash() -> Result<impl IntoResponse, ApiReport> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}

async fn upsert_by_external_id() -> Result<impl IntoResponse, ApiReport> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}

async fn get_file_metadata() -> Result<impl IntoResponse, ApiReport> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}

async fn update_file_metadata() -> Result<impl IntoResponse, ApiReport> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}

async fn new_file(
    State(state): AppState,
    Json(payload): Json<ItemPayload>,
) -> ApiResult<ItemResponse> {
    let new_item = db::items::ItemPayload {
        source_id: payload.source_id,
        version: 0,
        hash: None,
        saved_original_path: None,
        original_content: None,
        processed_content: None,
        status: payload.status,
        content_type: payload.content_type,
        external_id: payload.external_id,
        original_location: payload.original_location,
        tags: payload.tags,
        name: payload.name,
        title: payload.title,
        author: payload.author,
        description: payload.description,
        generated_summary: None,
        hidden: false,
    };

    let result = db::items::add_new_item(&state.pool, &new_item)
        .await
        .map(ItemResponse::from)?;

    Ok(Json(result))
}

async fn upload_file(
    State(state): AppState,
    Path(id): Path<i64>,
    mut body: BodyStream,
) -> Result<impl IntoResponse, ApiReport> {
    let item_data = db::items::lookup_by_id(&state.pool, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let new_version = match item_data.status {
        ItemStatus::WaitingForUpload | ItemStatus::Error => item_data.version,
        _ => item_data.version + 1,
    };

    let storage_filename = format!(
        "{}-{}",
        item_data.external_id.replace('/', "-"),
        time::OffsetDateTime::now_utc().unix_timestamp(),
    );

    let storage_dir = std::path::Path::new(&state.search_store.file_storage_location);
    let full_filename = storage_dir.join(&storage_filename);

    let mut hasher = blake3::Hasher::new();
    let file = tokio::fs::File::create(full_filename)
        .await
        .passthrough_error()?;
    let mut buffile = tokio::io::BufWriter::new(file);

    while let Some(chunk) = body.next().await {
        let chunk = chunk.passthrough_error()?;
        buffile.write_all(&chunk).await.passthrough_error()?;
        hasher.update(&chunk);
    }

    buffile.flush().await.passthrough_error()?;

    let output_hash = hasher.finalize();

    db::items::update_item_after_upload(
        &state.pool,
        id,
        new_version,
        output_hash.as_bytes(),
        &storage_filename,
    )
    .await?;

    // TODO Enqueue for processing

    if let Some(old_path) = item_data.saved_original_path.as_ref() {
        let old_filename = storage_dir.join(old_path);
        // TODO report error some other way
        tokio::fs::remove_file(old_filename).await.ok();
    }

    Ok(StatusCode::OK)
}

async fn delete_file() -> Result<impl IntoResponse, ApiReport> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}

pub fn create_router() -> Router<AppStateContents> {
    Router::new()
        .route("/", post(new_file))
        .route("/hash/:hash", get(lookup_by_hash).put(upsert_by_hash))
        .route(
            "/external_id/:external_id",
            get(lookup_by_external_id).put(upsert_by_external_id),
        )
        .route(
            "/id/:id",
            get(get_file_metadata)
                .patch(update_file_metadata)
                .delete(delete_file),
        )
        .route("/id/:id/upload", post(upload_file))
}
