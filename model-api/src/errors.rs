use axum::{http::StatusCode, response::IntoResponse, Json};
use error_stack::{FrameKind, IntoReport, Report, ResultExt};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Invalid input: {0}")]
    ArgError(String),
    #[error("Not found")]
    NotFound,
    #[error("Internal server error")]
    InternalError,
    #[error("")]
    Passthrough,
}

impl ApiError {
    fn status_code(&self) -> Option<StatusCode> {
        let code = match self {
            Self::ArgError(_) => StatusCode::BAD_REQUEST,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Passthrough => return None,
        };

        Some(code)
    }
}

#[derive(Serialize)]
pub struct ErrorMessage {
    code: u16,
    message: String,
    detail: String,
}

pub struct ReportError<T>(Report<T>);

pub type ApiReport = ReportError<ApiError>;

impl<T> IntoResponse for ReportError<T> {
    fn into_response(self) -> axum::response::Response {
        let report = self.0;

        let code: StatusCode;

        let message = report
            .frames()
            .find_map(|frame| {
                if let FrameKind::Context(c) = frame.kind() {
                    if let Some(ApiError::Passthrough) = frame.downcast_ref::<ApiError>() {
                        None
                    } else {
                        Some(c.to_string())
                    }
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "Internal server error".to_string());

        let code = report
            .frames()
            .find_map(|f| {
                f.downcast_ref::<ApiError>()
                    .and_then(|e| e.status_code())
                    .or_else(|| f.downcast_ref::<StatusCode>().copied())
            })
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        (
            code,
            Json(ErrorMessage {
                code: code.as_u16(),
                message,
                detail: format!("{report:?}"),
            }),
        )
            .into_response()
    }
}

impl<T> From<Report<T>> for ReportError<T> {
    fn from(report: Report<T>) -> Self {
        ReportError(report)
    }
}

pub trait PassthroughResult {
    type Ok;
    fn passthrough_error(self) -> Result<Self::Ok, Report<ApiError>>;
}

impl<T, E> PassthroughResult for Result<T, E>
where
    Result<T, E>: IntoReport,
{
    type Ok = <Self as IntoReport>::Ok;

    fn passthrough_error(self) -> Result<<Self as IntoReport>::Ok, Report<ApiError>> {
        self.into_report().change_context(ApiError::Passthrough)
    }
}
