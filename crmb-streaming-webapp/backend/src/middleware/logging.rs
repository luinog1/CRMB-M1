//! Logging middleware for request/response tracking
//!
//! This module provides comprehensive logging middleware for:
//! - Request/response logging with timing
//! - Structured logging with tracing
//! - Error logging and tracking
//! - Performance metrics collection
//! - Request correlation IDs
//! - User activity tracking

use axum::{
    extract::{MatchedPath, Request},
    http::{HeaderMap, Method, StatusCode, Uri},
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tracing::{info, warn, error, debug, Span};
use uuid::Uuid;

use crate::middleware::{extract_real_ip, extract_user_agent, RequestContext};
use crate::middleware::auth::get_user_context;

/// Request log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestLog {
    pub request_id: String,
    pub method: String,
    pub path: String,
    pub query: Option<String>,
    pub status_code: u16,
    pub duration_ms: u64,
    pub user_id: Option<u32>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub timestamp: String,
    pub error: Option<String>,
    pub bytes_sent: Option<u64>,
    pub bytes_received: Option<u64>,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub endpoint: String,
    pub method: String,
    pub avg_duration_ms: f64,
    pub min_duration_ms: u64,
    pub max_duration_ms: u64,
    pub request_count: u64,
    pub error_count: u64,
    pub success_rate: f64,
}

/// Request logging middleware with comprehensive tracking
pub async fn request_logging(request: Request, next: Next) -> Response {
    let start_time = Instant::now();
    let request_id = Uuid::new_v4().to_string();
    
    // Extract request information
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path().to_string();
    let query = uri.query().map(|q| q.to_string());
    let headers = request.headers().clone();
    
    // Extract client information
    let ip_address = extract_real_ip(&headers);
    let user_agent = extract_user_agent(&headers);
    
    // Create request context
    let mut request_context = RequestContext::new();
    request_context.request_id = request_id.clone();
    request_context.ip_address = ip_address.clone();
    request_context.user_agent = user_agent.clone();
    
    // Add request context to extensions
    let mut request = request;
    request.extensions_mut().insert(request_context);
    
    // Create tracing span for this request
    let span = tracing::info_span!(
        "http_request",
        request_id = %request_id,
        method = %method,
        path = %path,
        ip = ip_address.as_deref().unwrap_or("unknown"),
        user_agent = user_agent.as_deref().unwrap_or("unknown")
    );
    
    let _guard = span.enter();
    
    // Log incoming request
    debug!(
        "Incoming request: {} {} from {}",
        method,
        path,
        ip_address.as_deref().unwrap_or("unknown")
    );
    
    // Process request
    let response = next.run(request).await;
    
    // Calculate duration
    let duration = start_time.elapsed();
    let status_code = response.status();
    
    // Extract user information if available
    let user_id = response.extensions().get::<crate::middleware::auth::UserContext>()
        .map(|ctx| ctx.user.id);
    
    // Create request log entry
    let request_log = RequestLog {
        request_id: request_id.clone(),
        method: method.to_string(),
        path: path.clone(),
        query,
        status_code: status_code.as_u16(),
        duration_ms: duration.as_millis() as u64,
        user_id,
        user_agent,
        ip_address,
        timestamp: chrono::Utc::now().to_rfc3339(),
        error: if status_code.is_server_error() || status_code.is_client_error() {
            Some(status_code.canonical_reason().unwrap_or("Unknown error").to_string())
        } else {
            None
        },
        bytes_sent: None, // TODO: Extract from response body
        bytes_received: None, // TODO: Extract from request body
    };
    
    // Log based on status code
    match status_code {
        status if status.is_success() => {
            info!(
                "Request completed successfully: {} {} {} in {}ms",
                method,
                path,
                status_code.as_u16(),
                duration.as_millis()
            );
        }
        status if status.is_client_error() => {
            warn!(
                "Client error: {} {} {} in {}ms - {}",
                method,
                path,
                status_code.as_u16(),
                duration.as_millis(),
                status.canonical_reason().unwrap_or("Unknown error")
            );
        }
        status if status.is_server_error() => {
            error!(
                "Server error: {} {} {} in {}ms - {}",
                method,
                path,
                status_code.as_u16(),
                duration.as_millis(),
                status.canonical_reason().unwrap_or("Unknown error")
            );
        }
        _ => {
            debug!(
                "Request completed: {} {} {} in {}ms",
                method,
                path,
                status_code.as_u16(),
                duration.as_millis()
            );
        }
    }
    
    // Log slow requests
    if duration > Duration::from_millis(1000) {
        warn!(
            "Slow request detected: {} {} took {}ms",
            method,
            path,
            duration.as_millis()
        );
    }
    
    // TODO: Send metrics to monitoring system
    // send_metrics_to_monitoring_system(&request_log).await;
    
    response
}

/// Middleware for API-specific logging with detailed metrics
pub async fn api_request_logging(request: Request, next: Next) -> Response {
    let start_time = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path().to_string();
    
    // Extract matched path for better grouping
    let matched_path = request.extensions().get::<MatchedPath>()
        .map(|mp| mp.as_str().to_string())
        .unwrap_or_else(|| path.clone());
    
    let response = next.run(request).await;
    
    let duration = start_time.elapsed();
    let status_code = response.status();
    
    // Log API metrics
    info!(
        target: "api_metrics",
        endpoint = %matched_path,
        method = %method,
        status = %status_code.as_u16(),
        duration_ms = %duration.as_millis(),
        "API request completed"
    );
    
    response
}

/// Middleware for error logging with stack traces
pub async fn error_logging(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path().to_string();
    
    let response = next.run(request).await;
    
    // Log errors with additional context
    if response.status().is_server_error() {
        error!(
            "Server error occurred: {} {} returned {}",
            method,
            path,
            response.status().as_u16()
        );
        
        // TODO: Extract error details from response body if available
        // TODO: Send error to error tracking service (e.g., Sentry)
    }
    
    response
}

/// Middleware for user activity logging
pub async fn user_activity_logging(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path().to_string();
    
    let response = next.run(request).await;
    
    // Log user activities
    if let Some(user_context) = get_user_context(&request) {
        info!(
            target: "user_activity",
            user_id = %user_context.user.id,
            username = %user_context.user.username,
            action = %format!("{} {}", method, path),
            status = %response.status().as_u16(),
            "User activity logged"
        );
    }
    
    response
}

/// Middleware for security event logging
pub async fn security_logging(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path().to_string();
    let headers = request.headers().clone();
    let ip_address = extract_real_ip(&headers);
    
    let response = next.run(request).await;
    let status_code = response.status();
    
    // Log security-relevant events
    match status_code {
        StatusCode::UNAUTHORIZED => {
            warn!(
                target: "security",
                event = "unauthorized_access_attempt",
                path = %path,
                method = %method,
                ip = ip_address.as_deref().unwrap_or("unknown"),
                "Unauthorized access attempt"
            );
        }
        StatusCode::FORBIDDEN => {
            warn!(
                target: "security",
                event = "forbidden_access_attempt",
                path = %path,
                method = %method,
                ip = ip_address.as_deref().unwrap_or("unknown"),
                "Forbidden access attempt"
            );
        }
        StatusCode::TOO_MANY_REQUESTS => {
            warn!(
                target: "security",
                event = "rate_limit_exceeded",
                path = %path,
                method = %method,
                ip = ip_address.as_deref().unwrap_or("unknown"),
                "Rate limit exceeded"
            );
        }
        _ => {}
    }
    
    // Log suspicious patterns
    if path.contains("../") || path.contains("..\\") {
        warn!(
            target: "security",
            event = "path_traversal_attempt",
            path = %path,
            ip = ip_address.as_deref().unwrap_or("unknown"),
            "Potential path traversal attempt"
        );
    }
    
    response
}

/// Middleware for performance monitoring
pub async fn performance_monitoring(request: Request, next: Next) -> Response {
    let start_time = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    
    // Extract matched path for better grouping
    let matched_path = request.extensions().get::<MatchedPath>()
        .map(|mp| mp.as_str().to_string())
        .unwrap_or_else(|| uri.path().to_string());
    
    let response = next.run(request).await;
    
    let duration = start_time.elapsed();
    let status_code = response.status();
    
    // Log performance metrics
    info!(
        target: "performance",
        endpoint = %matched_path,
        method = %method,
        duration_ms = %duration.as_millis(),
        status = %status_code.as_u16(),
        "Performance metric recorded"
    );
    
    // Alert on slow requests
    if duration > Duration::from_millis(5000) {
        error!(
            target: "performance",
            endpoint = %matched_path,
            method = %method,
            duration_ms = %duration.as_millis(),
            "Very slow request detected"
        );
    } else if duration > Duration::from_millis(1000) {
        warn!(
            target: "performance",
            endpoint = %matched_path,
            method = %method,
            duration_ms = %duration.as_millis(),
            "Slow request detected"
        );
    }
    
    response
}

/// Middleware for request/response body logging (for debugging)
pub async fn debug_logging(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let headers = request.headers().clone();
    
    // Log request details in debug mode
    debug!(
        "Request details: {} {} with headers: {:?}",
        method,
        uri,
        headers
    );
    
    let response = next.run(request).await;
    
    // Log response details in debug mode
    debug!(
        "Response status: {} with headers: {:?}",
        response.status(),
        response.headers()
    );
    
    response
}

/// Helper function to extract request size
fn extract_request_size(headers: &HeaderMap) -> Option<u64> {
    headers
        .get(axum::http::header::CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|s| s.parse().ok())
}

/// Helper function to extract response size
fn extract_response_size(headers: &HeaderMap) -> Option<u64> {
    headers
        .get(axum::http::header::CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|s| s.parse().ok())
}

/// Helper function to determine if request should be logged
fn should_log_request(path: &str, method: &Method) -> bool {
    // Skip logging for health checks and static assets
    if path == "/health" || path == "/metrics" || path.starts_with("/static/") {
        return false;
    }
    
    // Always log non-GET requests
    if method != Method::GET {
        return true;
    }
    
    // Log API requests
    path.starts_with("/api/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderMap, HeaderValue, Method};

    #[test]
    fn test_extract_request_size() {
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::CONTENT_LENGTH,
            HeaderValue::from_static("1024"),
        );
        
        let size = extract_request_size(&headers);
        assert_eq!(size, Some(1024));
        
        // Test invalid content length
        headers.insert(
            axum::http::header::CONTENT_LENGTH,
            HeaderValue::from_static("invalid"),
        );
        
        let size = extract_request_size(&headers);
        assert_eq!(size, None);
    }

    #[test]
    fn test_should_log_request() {
        assert!(!should_log_request("/health", &Method::GET));
        assert!(!should_log_request("/metrics", &Method::GET));
        assert!(!should_log_request("/static/css/style.css", &Method::GET));
        
        assert!(should_log_request("/api/movies", &Method::GET));
        assert!(should_log_request("/api/auth/login", &Method::POST));
        assert!(should_log_request("/", &Method::POST));
    }

    #[test]
    fn test_request_log_serialization() {
        let request_log = RequestLog {
            request_id: "test-id".to_string(),
            method: "GET".to_string(),
            path: "/api/test".to_string(),
            query: Some("param=value".to_string()),
            status_code: 200,
            duration_ms: 150,
            user_id: Some(1),
            user_agent: Some("Test Agent".to_string()),
            ip_address: Some("192.168.1.1".to_string()),
            timestamp: "2023-01-01T00:00:00Z".to_string(),
            error: None,
            bytes_sent: Some(1024),
            bytes_received: Some(512),
        };
        
        let json = serde_json::to_string(&request_log).unwrap();
        assert!(json.contains("test-id"));
        assert!(json.contains("GET"));
        assert!(json.contains("/api/test"));
    }
}