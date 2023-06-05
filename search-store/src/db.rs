use thiserror::Error;

pub mod chat_sessions;
pub mod models;

#[derive(Debug, Error)]
#[error("Database error")]
pub struct DbError {}
