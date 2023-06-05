use error_stack::{IntoReport, Report, ResultExt};
use serde::{Deserialize, Serialize};
use sqlx::{query_as, FromRow, PgPool};
use sqlx_transparent_json_decode::sqlx_json_decode;

use super::DbError;

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatSystemMessage {
    pub id: i64,
    pub name: Option<String>,
    pub message: String,
    pub hidden: bool,
    pub created_at: time::OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: i64,
    pub name: String,
    pub system_message_id: Option<i64>,
    pub parent_session: Option<i64>,
    pub hidden: bool,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: i64,
    pub session_id: i64,
    pub parent_id: Option<i64>,
    pub important: bool,
    pub user_message: String,
    pub ai_message: Option<String>,
    pub created_at: time::OffsetDateTime,
}

sqlx_json_decode!(ChatMessage);

pub async fn list_chat_sessions(pool: &PgPool) -> Result<Vec<ChatSession>, Report<DbError>> {
    todo!()
}

pub async fn get_chat_session_messages(
    pool: &PgPool,
    session_id: i64,
) -> Result<Vec<ChatMessage>, Report<DbError>> {
    query_as!(
        ChatMessage,
        r##"SELECT
            id,
            session_id,
            parent_id,
            important,
            user_message,
            ai_message,
            created_at
        FROM chat_messages
        WHERE session_id = $1
        ORDER BY created_at"##,
        session_id
    )
    .fetch_all(pool)
    .await
    .into_report()
    .change_context(DbError {})
}

pub async fn add_chat_message(
    pool: &PgPool,
    session_id: i64,
    parent_id: Option<i64>,
    important: bool,
    user_message: String,
    ai_message: Option<String>,
) -> Result<ChatMessage, Report<DbError>> {
    query_as!(
        ChatMessage,
        r##"INSERT INTO chat_messages
            (session_id, parent_id, important, user_message, ai_message)
        VALUES
            ($1, $2, $3, $4, $5)
        RETURNING
            id,
            session_id,
            parent_id,
            important,
            user_message,
            ai_message,
            created_at
        "##,
        session_id,
        parent_id,
        important,
        user_message,
        ai_message
    )
    .fetch_one(pool)
    .await
    .into_report()
    .change_context(DbError {})
}

pub async fn add_ai_response_to_chat_message(
    pool: &PgPool,
    message_id: i64,
    ai_message: String,
) -> Result<ChatMessage, Report<DbError>> {
    query_as!(
        ChatMessage,
        r##"UPDATE chat_messages
        SET ai_message = $2
        WHERE id = $1
        RETURNING
            id,
            session_id,
            parent_id,
            important,
            user_message,
            ai_message,
            created_at
        "##,
        message_id,
        ai_message
    )
    .fetch_one(pool)
    .await
    .into_report()
    .change_context(DbError {})
}
