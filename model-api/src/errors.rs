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
    #[error("Model not loaded or is not a {0} model")]
    ModelNotLoaded(&'static str),
    #[error("Not implemented")]
    NotImplmented,
}

impl ApiError {
    fn status_code(&self) -> Option<StatusCode> {
        let code = match self {
            Self::ArgError(_) => StatusCode::BAD_REQUEST,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ModelNotLoaded(_) => StatusCode::BAD_REQUEST,
            Self::NotImplmented => StatusCode::NOT_IMPLEMENTED,
            Self::Passthrough => return None,
        };

        Some(code)
    }
}

impl From<ApiError> for ApiReport {
    fn from(value: ApiError) -> Self {
        ReportError(Report::new(value))
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
pub type ApiResult<T> = Result<Json<T>, ApiReport>;

impl<T> IntoResponse for ReportError<T> {
    fn into_response(self) -> axum::response::Response {
        let report = self.0;

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

pub trait IntoPassthrough {
    type Ok;
    fn passthrough_error(self) -> Result<Self::Ok, Report<ApiError>>;
}

impl<T, E> IntoPassthrough for Result<T, E>
where
    Result<T, E>: IntoReport,
{
    type Ok = <Self as IntoReport>::Ok;

    fn passthrough_error(self) -> Result<<Self as IntoReport>::Ok, Report<ApiError>> {
        self.into_report().change_context(ApiError::Passthrough)
    }
}

/// Once specialization is implemented we can combine this with into_passthrough
pub trait PassthroughReport {
    type Ok;
    fn passthrough_error(self) -> Result<Self::Ok, Report<ApiError>>;
}

impl<T, E> PassthroughReport for Result<T, Report<E>> {
    type Ok = T;

    fn passthrough_error(
        self,
    ) -> Result<
        <std::result::Result<T, error_stack::Report<E>> as PassthroughReport>::Ok,
        Report<ApiError>,
    > {
        self.change_context(ApiError::Passthrough)
    }
}
