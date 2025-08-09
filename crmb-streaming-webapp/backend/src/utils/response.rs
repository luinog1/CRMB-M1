//! Response utilities for consistent API responses

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::{json, Value};

/// Create a success response with data
pub fn success_response<T: serde::Serialize>(data: T) -> impl IntoResponse {
    (StatusCode::OK, Json(json!({
        "success": true,
        "data": data
    })))
}

/// Create a success response with message
pub fn success_message(message: &str) -> impl IntoResponse {
    (StatusCode::OK, Json(json!({
        "success": true,
        "message": message
    })))
}

/// Create an error response
pub fn error_response(status: StatusCode, message: &str) -> impl IntoResponse {
    (status, Json(json!({
        "success": false,
        "error": message
    })))
}

/// Create a validation error response
pub fn validation_error(errors: Vec<String>) -> impl IntoResponse {
    (StatusCode::BAD_REQUEST, Json(json!({
        "success": false,
        "error": "Validation failed",
        "details": errors
    })))
}

/// Create a not found response
pub fn not_found(resource: &str) -> impl IntoResponse {
    error_response(StatusCode::NOT_FOUND, &format!("{} not found", resource))
}

/// Create an unauthorized response
pub fn unauthorized() -> impl IntoResponse {
    error_response(StatusCode::UNAUTHORIZED, "Unauthorized")
}

/// Create a forbidden response
pub fn forbidden() -> impl IntoResponse {
    error_response(StatusCode::FORBIDDEN, "Forbidden")
}