mod errors;
mod models;
mod tracing_config;

use std::{path::PathBuf, sync::Arc};

use axum::{routing::get, Router};
use error_stack::{IntoReport, Report, ResultExt};
use maiven_search_store::{models::download::ModelCache, SearchStore};
use models::list_models;
use sqlx::postgres::PgPoolOptions;
use thiserror::Error;

pub struct AppStateInner {
    pub pool: sqlx::PgPool,
    pub search_store: SearchStore,
}

pub type AppState = Arc<AppStateInner>;

#[derive(Error, Debug)]
#[error("app error")]
struct MainError;

#[tokio::main]
async fn main() -> Result<(), Report<MainError>> {
    dotenvy::dotenv().ok();
    tracing_config::configure(std::io::stdout);
    let supports_color = supports_color::on_cached(supports_color::Stream::Stdout)
        .map_or(false, |level| level.has_basic);

    if supports_color {
        error_stack::Report::set_color_mode(error_stack::fmt::ColorMode::Color);
    }

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
        .with_state(Arc::new(app_state));

    axum::Server::bind(&"127.0.0.1:9824".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .into_report()
        .change_context(MainError {})?;

    Ok(())
}
