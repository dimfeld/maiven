mod errors;
mod models;
mod tracing_config;

use axum::{routing::get, Router};
use models::list_models;
use sqlx::postgres::PgPoolOptions;

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_config::configure(std::io::stdout);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let app_state = AppState { pool };

    let app = Router::new()
        .route("/models", get(list_models))
        .with_state(app_state);

    axum::Server::bind(&"127.0.0.1:9824".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
