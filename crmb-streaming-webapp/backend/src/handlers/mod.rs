pub mod auth;
pub mod enhanced_metadata;
pub mod health;
pub mod movies;
pub mod stremio;
pub mod stremio_mdblist;
pub mod tmdb;
pub mod tv;
pub mod user;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde_json::json;

// Common error response handler
pub async fn handle_error(err: Box<dyn std::error::Error + Send + Sync>) -> impl IntoResponse {
    let error_message = err.to_string();
    tracing::error!("Handler error: {}", error_message);
    
    let response = json!({
        "success": false,
        "error": {
            "message": "Internal server error",
            "code": "INTERNAL_ERROR"
        }
    });
    
    (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
}

// Common not found handler
pub async fn not_found() -> impl IntoResponse {
    let response = json!({
        "success": false,
        "error": {
            "message": "Resource not found",
            "code": "NOT_FOUND"
        }
    });
    
    (StatusCode::NOT_FOUND, Json(response))
}

// Common method not allowed handler
pub async fn method_not_allowed() -> impl IntoResponse {
    let response = json!({
        "success": false,
        "error": {
            "message": "Method not allowed",
            "code": "METHOD_NOT_ALLOWED"
        }
    });
    
    (StatusCode::METHOD_NOT_ALLOWED, Json(response))
}