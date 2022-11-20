use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

#[derive(Debug)]
struct AnyError(anyhow::Error);

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Other(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Something went wrong, {}", err),
            )
                .into_response(),
        }
    }
}

impl<E> From<E> for AnyError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
