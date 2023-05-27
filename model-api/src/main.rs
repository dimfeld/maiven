mod models;
mod tracing_config;

use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    tracing_config::configure(std::io::stdout);

    let app = Router::new().route("/models", get(|| async { "Hello, World!" }));

    axum::Server::bind(&"127.0.0.1:9824".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
