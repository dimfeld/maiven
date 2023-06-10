use error_stack::{IntoReport, Report, ResultExt};
use serde::{Deserialize, Serialize};
use sqlx::{query_as, FromRow, PgPool};

use super::DbError;

#[derive(Serialize, Deserialize, Debug, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "item_status", rename_all = "snake_case")]
pub enum ItemStatus {
    WaitingForUpload,
    Pending,
    Processing,
    Ready,
    Error,
}

#[derive(Serialize, Deserialize, Debug, FromRow)]
pub struct ItemPayload {
    pub source_id: i32,
    pub status: ItemStatus,
    pub content_type: String,
    pub external_id: String,
    pub version: i32,
    pub hash: Option<Vec<u8>>,

    pub saved_original_path: Option<String>,
    pub original_location: Option<String>,
    pub original_content: Option<String>,
    pub processed_content: Option<String>,
    pub tags: Vec<i32>,
    pub name: Option<String>,
    pub title: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub generated_summary: Option<String>,
    pub hidden: bool,
}

#[derive(Debug, FromRow)]
pub struct ItemMetadata {
    pub id: i64,
    pub source_id: i32,
    pub status: ItemStatus,
    pub content_type: String,
    pub external_id: String,
    pub version: i32,
    pub hash: Option<Vec<u8>>,

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

pub async fn add_new_item(
    pool: &PgPool,
    item: &ItemPayload,
) -> Result<ItemMetadata, Report<DbError>> {
    query_as!(
        ItemMetadata,
        r#"
        INSERT INTO items (
            source_id,
            status,
            content_type,
            external_id,
            version,
            hash,
            saved_original_path,
            original_location,
            original_content,
            tags,
            name,
            title,
            author,
            description,
            generated_summary,
            updated_at,
            hidden
        )
        VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15,
            now(),
            false
        )
        RETURNING
            id, source_id, status as "status: ItemStatus", content_type, external_id, version, hash,
            saved_original_path, original_location, tags, name, title, author,
            description, generated_summary, updated_at, hidden"#,
        item.source_id,
        item.status as _,
        item.content_type,
        item.external_id,
        item.version,
        item.hash,
        item.saved_original_path,
        item.original_location,
        item.original_content,
        item.tags.as_slice(),
        item.name,
        item.title,
        item.author,
        item.description,
        item.generated_summary,
    )
    .fetch_one(pool)
    .await
    .into_report()
    .change_context(DbError {})
}

pub async fn lookup_by_hash(
    pool: &PgPool,
    hash: &[u8],
) -> Result<Option<ItemMetadata>, Report<DbError>> {
    query_as!(
        ItemMetadata,
        r#"
        SELECT
            id, source_id, status as "status: ItemStatus", content_type, external_id, version, hash,
            saved_original_path, original_location, tags, name, title, author,
            description, generated_summary, updated_at, hidden
        FROM items
        WHERE hash = $1
        LIMIT 1"#,
        hash
    )
    .fetch_optional(pool)
    .await
    .into_report()
    .change_context(DbError {})
}

pub async fn lookup_by_external_id(
    pool: &PgPool,
    id: &str,
) -> Result<Option<ItemMetadata>, Report<DbError>> {
    query_as!(
        ItemMetadata,
        r#"
        SELECT
            id, source_id, status as "status: ItemStatus", content_type, external_id, version, hash,
            saved_original_path, original_location, tags, name, title, author,
            description, generated_summary, updated_at, hidden
        FROM items
        WHERE external_id = $1
        LIMIT 1"#,
        id
    )
    .fetch_optional(pool)
    .await
    .into_report()
    .change_context(DbError {})
}
