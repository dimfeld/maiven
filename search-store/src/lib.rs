mod models;

use std::path::PathBuf;

use sqlx::PgPool;

pub struct SearchStore {
    pool: PgPool,
    model_cache_dir: PathBuf,
    content_dir: PathBuf,
}

// load models
