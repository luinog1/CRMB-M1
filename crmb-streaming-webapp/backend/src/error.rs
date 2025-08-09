//! Centralized error handling for the CRMB Streaming WebApp
//!
//! This module provides comprehensive error types and handling for:
//! - API errors with proper HTTP status codes
//! - Database errors
//! - Authentication and authorization errors
//! - External service errors (TMDB, Stremio)
//! - Validation errors
//! - Rate limiting errors
//! - Caching errors

use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Main application error type
#[derive(Error, Debug)]
pub enum AppError {
    /// Database-related errors
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// Authentication errors
    #[error("Authentication error: {0}")]
    Auth(#[from] crate::models::auth::AuthError),

    /// Validation errors
    #[error("Validation error: {0}")]
    Validation(String),

    /// External API errors
    #[error("External API error: {0}")]
    ExternalApi(#[from] ExternalApiError),

    /// Rate limiting errors
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    /// Caching errors
    #[error("Cache error: {0}")]
    Cache(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Serialization/Deserialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// HTTP client errors
    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Not found errors
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Unauthorized access
    #[error("Unauthorized access")]
    Unauthorized,

    /// Forbidden access
    #[error("Forbidden access: {0}")]
    Forbidden(String),

    /// Bad request
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// Internal server error
    #[error("Internal server error: {0}")]
    Internal(String),

    /// Service unavailable
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    /// Timeout error
    #[error("Request timeout: {0}")]
    Timeout(String),

    /// Conflict error
    #[error("Conflict: {0}")]
    Conflict(String),

    /// Too many requests
    #[error("Too many requests: {0}")]
    TooManyRequests(String),
}

/// External API error types
#[derive(Error, Debug)]
pub enum ExternalApiError {
    /// TMDB API errors
    #[error("TMDB API error: {0}")]
    Tmdb(TmdbError),

    /// Stremio addon errors
    #[error("Stremio error: {0}")]
    Stremio(StremioError),

    /// MDBList API errors
    #[error("MDBList API error: {0}")]
    MdbList(String),

    /// Generic external service error
    #[error("External service error: {service} - {message}")]
    Generic { service: String, message: String },
}

/// TMDB API specific errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum TmdbError {
    #[error("Invalid API key")]
    InvalidApiKey,

    #[error("Resource not found")]
    NotFound,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Service unavailable")]
    ServiceUnavailable,

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Stremio addon specific errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum StremioError {
    #[error("Invalid addon manifest")]
    InvalidManifest,

    #[error("Catalog not found: {0}")]
    CatalogNotFound(String),

    #[error("Meta not found: {0}")]
    MetaNotFound(String),

    #[error("Stream not found: {0}")]
    StreamNotFound(String),

    #[error("Invalid ID format: {0}")]
    InvalidIdFormat(String),

    #[error("Unsupported type: {0}")]
    UnsupportedType(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("External service error: {0}")]
    ExternalServiceError(String),
}

/// Error response structure for API responses
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub code: String,
    pub details: Option<serde_json::Value>,
    pub timestamp: String,
    pub path: Option<String>,
}

/// Validation error details
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub code: String,
    pub value: Option<serde_json::Value>,
}

/// Multiple validation errors
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationErrors {
    pub errors: Vec<ValidationError>,
    pub message: String,
}

impl AppError {
    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::Auth(auth_error) => auth_error.status_code(),
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::TooManyRequests(_) | AppError::RateLimit(_) => StatusCode::TOO_MANY_REQUESTS,
            AppError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            AppError::Timeout(_) => StatusCode::REQUEST_TIMEOUT,
            AppError::ExternalApi(external_error) => external_error.status_code(),
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Cache(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Serialization(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::HttpClient(_) => StatusCode::BAD_GATEWAY,
            AppError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Get the error code for this error
    pub fn error_code(&self) -> &'static str {
        match self {
            AppError::Auth(_) => "AUTH_ERROR",
            AppError::Validation(_) => "VALIDATION_ERROR",
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::Unauthorized => "UNAUTHORIZED",
            AppError::Forbidden(_) => "FORBIDDEN",
            AppError::BadRequest(_) => "BAD_REQUEST",
            AppError::Conflict(_) => "CONFLICT",
            AppError::TooManyRequests(_) => "TOO_MANY_REQUESTS",
            AppError::RateLimit(_) => "RATE_LIMIT_EXCEEDED",
            AppError::ServiceUnavailable(_) => "SERVICE_UNAVAILABLE",
            AppError::Timeout(_) => "TIMEOUT",
            AppError::ExternalApi(_) => "EXTERNAL_API_ERROR",
            AppError::Database(_) => "DATABASE_ERROR",
            AppError::Cache(_) => "CACHE_ERROR",
            AppError::Config(_) => "CONFIG_ERROR",
            AppError::Serialization(_) => "SERIALIZATION_ERROR",
            AppError::HttpClient(_) => "HTTP_CLIENT_ERROR",
            AppError::Io(_) => "IO_ERROR",
            AppError::Internal(_) => "INTERNAL_ERROR",
        }
    }

    /// Create a validation error
    pub fn validation<T: Into<String>>(message: T) -> Self {
        AppError::Validation(message.into())
    }

    /// Create a not found error
    pub fn not_found<T: Into<String>>(resource: T) -> Self {
        AppError::NotFound(resource.into())
    }

    /// Create a bad request error
    pub fn bad_request<T: Into<String>>(message: T) -> Self {
        AppError::BadRequest(message.into())
    }

    /// Create an internal error
    pub fn internal<T: Into<String>>(message: T) -> Self {
        AppError::Internal(message.into())
    }

    /// Create a forbidden error
    pub fn forbidden<T: Into<String>>(message: T) -> Self {
        AppError::Forbidden(message.into())
    }

    /// Create a conflict error
    pub fn conflict<T: Into<String>>(message: T) -> Self {
        AppError::Conflict(message.into())
    }

    /// Create a service unavailable error
    pub fn service_unavailable<T: Into<String>>(message: T) -> Self {
        AppError::ServiceUnavailable(message.into())
    }

    /// Create a timeout error
    pub fn timeout<T: Into<String>>(message: T) -> Self {
        AppError::Timeout(message.into())
    }
}

impl ExternalApiError {
    /// Get the HTTP status code for external API errors
    pub fn status_code(&self) -> StatusCode {
        match self {
            ExternalApiError::Tmdb(tmdb_error) => tmdb_error.status_code(),
            ExternalApiError::Stremio(stremio_error) => stremio_error.status_code(),
            ExternalApiError::MdbList(_) => StatusCode::BAD_GATEWAY,
            ExternalApiError::Generic { .. } => StatusCode::BAD_GATEWAY,
        }
    }
}

impl TmdbError {
    /// Get the HTTP status code for TMDB errors
    pub fn status_code(&self) -> StatusCode {
        match self {
            TmdbError::InvalidApiKey => StatusCode::UNAUTHORIZED,
            TmdbError::NotFound => StatusCode::NOT_FOUND,
            TmdbError::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
            TmdbError::InvalidRequest(_) => StatusCode::BAD_REQUEST,
            TmdbError::AuthenticationFailed => StatusCode::UNAUTHORIZED,
            TmdbError::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            TmdbError::NetworkError(_) => StatusCode::BAD_GATEWAY,
            TmdbError::ParseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            TmdbError::Unknown(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Create TMDB error from HTTP status code
    pub fn from_status_code(status: StatusCode, message: Option<String>) -> Self {
        let default_message = message.unwrap_or_else(|| "Unknown error".to_string());
        
        match status {
            StatusCode::UNAUTHORIZED => TmdbError::InvalidApiKey,
            StatusCode::NOT_FOUND => TmdbError::NotFound,
            StatusCode::TOO_MANY_REQUESTS => TmdbError::RateLimitExceeded,
            StatusCode::BAD_REQUEST => TmdbError::InvalidRequest(default_message),
            StatusCode::SERVICE_UNAVAILABLE => TmdbError::ServiceUnavailable,
            _ => TmdbError::Unknown(default_message),
        }
    }
}

impl StremioError {
    /// Get the HTTP status code for Stremio errors
    pub fn status_code(&self) -> StatusCode {
        match self {
            StremioError::InvalidManifest => StatusCode::INTERNAL_SERVER_ERROR,
            StremioError::CatalogNotFound(_) => StatusCode::NOT_FOUND,
            StremioError::MetaNotFound(_) => StatusCode::NOT_FOUND,
            StremioError::StreamNotFound(_) => StatusCode::NOT_FOUND,
            StremioError::InvalidIdFormat(_) => StatusCode::BAD_REQUEST,
            StremioError::UnsupportedType(_) => StatusCode::BAD_REQUEST,
            StremioError::ConfigError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            StremioError::ExternalServiceError(_) => StatusCode::BAD_GATEWAY,
        }
    }
}

impl crate::models::auth::AuthError {
    /// Get the HTTP status code for authentication errors
    pub fn status_code(&self) -> StatusCode {
        match self {
            crate::models::auth::AuthError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            crate::models::auth::AuthError::InvalidToken => StatusCode::UNAUTHORIZED,
            crate::models::auth::AuthError::TokenExpired => StatusCode::UNAUTHORIZED,
            crate::models::auth::AuthError::TokenGenerationFailed => StatusCode::INTERNAL_SERVER_ERROR,
            crate::models::auth::AuthError::UserAlreadyExists => StatusCode::CONFLICT,
            crate::models::auth::AuthError::UserNotFound => StatusCode::NOT_FOUND,
            crate::models::auth::AuthError::InvalidEmail => StatusCode::BAD_REQUEST,
            crate::models::auth::AuthError::InvalidUsername => StatusCode::BAD_REQUEST,
            crate::models::auth::AuthError::WeakPassword(_) => StatusCode::BAD_REQUEST,
            crate::models::auth::AuthError::PasswordHashingFailed => StatusCode::INTERNAL_SERVER_ERROR,
            crate::models::auth::AuthError::PasswordVerificationFailed => StatusCode::INTERNAL_SERVER_ERROR,
            crate::models::auth::AuthError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            crate::models::auth::AuthError::SessionExpired => StatusCode::UNAUTHORIZED,
            crate::models::auth::AuthError::SessionNotFound => StatusCode::UNAUTHORIZED,
            crate::models::auth::AuthError::InsufficientPermissions => StatusCode::FORBIDDEN,
        }
    }
}

/// Convert AppError to HTTP response
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_code = self.error_code();
        
        let error_response = ErrorResponse {
            error: error_code.to_string(),
            message: self.to_string(),
            code: error_code.to_string(),
            details: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
            path: None,
        };

        // Log the error
        match status {
            StatusCode::INTERNAL_SERVER_ERROR => {
                tracing::error!("Internal server error: {}", self);
            }
            StatusCode::BAD_GATEWAY | StatusCode::SERVICE_UNAVAILABLE => {
                tracing::warn!("External service error: {}", self);
            }
            _ => {
                tracing::debug!("Client error: {}", self);
            }
        }

        (status, Json(error_response)).into_response()
    }
}

/// Convert validation errors to HTTP response
impl IntoResponse for ValidationErrors {
    fn into_response(self) -> Response {
        let error_response = ErrorResponse {
            error: "VALIDATION_ERROR".to_string(),
            message: self.message,
            code: "VALIDATION_ERROR".to_string(),
            details: Some(serde_json::to_value(&self.errors).unwrap_or_default()),
            timestamp: chrono::Utc::now().to_rfc3339(),
            path: None,
        };

        (StatusCode::BAD_REQUEST, Json(error_response)).into_response()
    }
}

/// Result type alias for the application
pub type AppResult<T> = Result<T, AppError>;

/// Helper function to create validation errors
pub fn validation_error(field: &str, message: &str, code: &str) -> ValidationError {
    ValidationError {
        field: field.to_string(),
        message: message.to_string(),
        code: code.to_string(),
        value: None,
    }
}

/// Helper function to create multiple validation errors
pub fn validation_errors(errors: Vec<ValidationError>, message: &str) -> ValidationErrors {
    ValidationErrors {
        errors,
        message: message.to_string(),
    }
}

/// Helper function to handle database errors
pub fn handle_database_error(error: sqlx::Error) -> AppError {
    match error {
        sqlx::Error::RowNotFound => AppError::NotFound("Resource not found".to_string()),
        sqlx::Error::Database(db_error) => {
            if let Some(constraint) = db_error.constraint() {
                AppError::Conflict(format!("Constraint violation: {}", constraint))
            } else {
                AppError::Database(sqlx::Error::Database(db_error))
            }
        }
        _ => AppError::Database(error),
    }
}

/// Helper function to handle external API errors
pub fn handle_external_api_error(service: &str, error: reqwest::Error) -> AppError {
    if error.is_timeout() {
        AppError::Timeout(format!("{} request timeout", service))
    } else if error.is_connect() {
        AppError::ServiceUnavailable(format!("{} service unavailable", service))
    } else if let Some(status) = error.status() {
        match status {
            StatusCode::TOO_MANY_REQUESTS => {
                AppError::RateLimit(format!("{} rate limit exceeded", service))
            }
            StatusCode::UNAUTHORIZED => {
                AppError::ExternalApi(ExternalApiError::Generic {
                    service: service.to_string(),
                    message: "Authentication failed".to_string(),
                })
            }
            StatusCode::NOT_FOUND => {
                AppError::NotFound(format!("{} resource not found", service))
            }
            _ => AppError::ExternalApi(ExternalApiError::Generic {
                service: service.to_string(),
                message: format!("HTTP {}: {}", status, error),
            }),
        }
    } else {
        AppError::HttpClient(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_error_status_codes() {
        assert_eq!(AppError::Unauthorized.status_code(), StatusCode::UNAUTHORIZED);
        assert_eq!(AppError::NotFound("test".to_string()).status_code(), StatusCode::NOT_FOUND);
        assert_eq!(AppError::BadRequest("test".to_string()).status_code(), StatusCode::BAD_REQUEST);
        assert_eq!(AppError::Internal("test".to_string()).status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_tmdb_error_from_status_code() {
        let error = TmdbError::from_status_code(StatusCode::UNAUTHORIZED, None);
        assert!(matches!(error, TmdbError::InvalidApiKey));

        let error = TmdbError::from_status_code(StatusCode::NOT_FOUND, None);
        assert!(matches!(error, TmdbError::NotFound));

        let error = TmdbError::from_status_code(StatusCode::TOO_MANY_REQUESTS, None);
        assert!(matches!(error, TmdbError::RateLimitExceeded));
    }

    #[test]
    fn test_validation_error_creation() {
        let error = validation_error("email", "Invalid email format", "INVALID_EMAIL");
        assert_eq!(error.field, "email");
        assert_eq!(error.message, "Invalid email format");
        assert_eq!(error.code, "INVALID_EMAIL");
    }

    #[test]
    fn test_error_response_serialization() {
        let error_response = ErrorResponse {
            error: "TEST_ERROR".to_string(),
            message: "Test error message".to_string(),
            code: "TEST_ERROR".to_string(),
            details: None,
            timestamp: "2023-01-01T00:00:00Z".to_string(),
            path: Some("/test".to_string()),
        };

        let json = serde_json::to_string(&error_response).unwrap();
        assert!(json.contains("TEST_ERROR"));
        assert!(json.contains("Test error message"));
    }
}

// Add From<anyhow::Error> implementation
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

// Add From<JsonRejection> implementation
impl From<JsonRejection> for AppError {
    fn from(rejection: JsonRejection) -> Self {
        AppError::BadRequest(format!("Invalid JSON: {}", rejection))
    }
}