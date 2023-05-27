use thiserror::Error;

pub mod models;

#[derive(Debug, Error)]
#[error("Database error")]
pub struct DbError {}
