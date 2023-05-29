use error_stack::Report;

use axum::{http::StatusCode, response::IntoResponse};

pub struct ReportError<T>(Report<T>);

impl<T> IntoResponse for ReportError<T> {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()).into_response()
    }
}

impl<T> From<Report<T>> for ReportError<T> {
    fn from(report: Report<T>) -> Self {
        ReportError(report)
    }
}
