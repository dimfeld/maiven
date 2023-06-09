use thiserror::Error;

#[derive(Debug, Error)]
pub enum ModelError {
    #[error("Model worker has closed")]
    WorkerClosed,
    #[error("Model error")]
    ModelFailure,
    #[error("Failed to load model")]
    LoadingError,
    #[error("Unsupported model type {0}")]
    UnknownModelType(String),
}
