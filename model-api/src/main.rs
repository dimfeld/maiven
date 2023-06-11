mod chat;
mod errors;
mod items;
mod models;
mod sources;
mod tracing_config;

use std::{path::PathBuf, sync::Arc};

use axum::{extract::State, routing::get, Router};
use error_stack::{IntoReport, Report, ResultExt};
use maiven_search_store::{models::download::ModelCache, SearchStore};
use sqlx::postgres::PgPoolOptions;
use thiserror::Error;

pub struct AppStateInner {
    pub pool: sqlx::PgPool,
    pub search_store: SearchStore,
}

pub type AppStateContents = Arc<AppStateInner>;
pub type AppState = State<AppStateContents>;

#[derive(Error, Debug)]
#[error("app error")]
struct MainError;

#[tokio::main]
async fn main() -> Result<(), Report<MainError>> {
    dotenvy::dotenv().ok();
    tracing_config::configure(std::io::stdout);

    error_stack::Report::set_color_mode(error_stack::fmt::ColorMode::None);

    let file_storage_dir = std::env::var("FILE_STORAGE_LOCATION")
        .into_report()
        .attach_printable("FILE_STORAGE_LOCATION")
        .change_context(MainError {})?;

    let model_cache_dir = std::env::var("MODEL_DIR")
        .into_report()
        .attach_printable("MODEL_DIR")
        .change_context(MainError {})?;
    let model_cache = ModelCache::new(PathBuf::from(model_cache_dir));

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .into_report()
        .attach_printable("DATABASE_URL")
        .change_context(MainError {})?;

    let app_state = AppStateInner {
        pool: pool.clone(),
        search_store: SearchStore::new(pool, model_cache),
    };

    let app = Router::new()
        .nest("/models", models::create_router())
        .nest("/chats", chat::create_router())
        .nest("/items", items::create_router())
        .nest("/sources", sources::create_router())
        .with_state(Arc::new(app_state));

    axum::Server::bind(&"127.0.0.1:9824".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .into_report()
        .change_context(MainError {})?;

    Ok(())
}
