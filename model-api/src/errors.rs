use axum::{http::StatusCode, response::IntoResponse, Json};
use error_stack::Report;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Invalid input: {0}")]
    ArgError(String),
}

impl ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::ArgError(_) => StatusCode::BAD_REQUEST,
        }
    }
}

#[derive(Serialize)]
pub struct ErrorMessage {
    code: u16,
    message: String,
    detail: String,
}

pub struct ReportError<T>(Report<T>);

impl<T> IntoResponse for ReportError<T> {
    fn into_response(self) -> axum::response::Response {
        let report = self.0;

        let code = report
            .frames()
            .find_map(|f| {
                f.downcast_ref::<ApiError>()
                    .map(|e| e.status_code())
                    .or_else(|| f.downcast_ref::<StatusCode>().copied())
            })
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        (
            code,
            Json(ErrorMessage {
                code: code.as_u16(),
                message: format!("{report}"),
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
