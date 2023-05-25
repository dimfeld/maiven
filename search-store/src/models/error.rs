use thiserror::Error;

#[derive(Debug, Error)]
pub enum ModelError {
    #[error("Model worker has closed")]
    WorkerClosed,
    // TODO make this better
    #[error("Generic model failure")]
    ModelFailure,
    #[error("Failed to load model")]
    LoadingError,
    #[error("Unsupported model type {0}")]
    UnknownModelType(String),
}
