//! Error handling utilities

use axum::{http::StatusCode, response::IntoResponse};
use crate::utils::response::error_response;

/// Convert common error types to HTTP responses
pub trait IntoHttpError {
    fn into_http_error(self) -> impl IntoResponse;
}

impl IntoHttpError for sqlx::Error {
    fn into_http_error(self) -> impl IntoResponse {
        tracing::error!("Database error: {:?}", self);
        error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
    }
}

impl IntoHttpError for reqwest::Error {
    fn into_http_error(self) -> impl IntoResponse {
        tracing::error!("HTTP client error: {:?}", self);
        error_response(StatusCode::BAD_GATEWAY, "External service error")
    }
}

impl IntoHttpError for serde_json::Error {
    fn into_http_error(self) -> impl IntoResponse {
        tracing::error!("JSON parsing error: {:?}", self);
        error_response(StatusCode::BAD_REQUEST, "Invalid JSON format")
    }
}

/// Log and convert any error to a generic internal server error
pub fn log_and_convert_error<E: std::fmt::Debug>(error: E) -> impl IntoResponse {
    tracing::error!("Unexpected error: {:?}", error);
    error_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
}