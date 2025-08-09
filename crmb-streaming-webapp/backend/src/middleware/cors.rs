//! CORS middleware for cross-origin request handling
//!
//! This module provides CORS (Cross-Origin Resource Sharing) middleware for:
//! - Frontend integration with proper origin validation
//! - Development and production CORS policies
//! - Preflight request handling
//! - Credential support for authenticated requests
//! - Custom headers and methods configuration

use axum::{
    extract::{Request, State},
    http::{header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, ORIGIN},
    HeaderValue, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tower_http::cors::{Any, CorsLayer};
use tracing::{debug, warn};

/// CORS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    /// Allowed origins
    pub allowed_origins: Vec<String>,
    /// Allowed methods
    pub allowed_methods: Vec<String>,
    /// Allowed headers
    pub allowed_headers: Vec<String>,
    /// Exposed headers
    pub exposed_headers: Vec<String>,
    /// Allow credentials
    pub allow_credentials: bool,
    /// Max age for preflight cache
    pub max_age_seconds: u64,
    /// Development mode (allows all origins)
    pub development_mode: bool,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec![
                "http://localhost:3000".to_string(),
                "http://localhost:5173".to_string(), // Vite dev server
                "http://127.0.0.1:3000".to_string(),
                "http://127.0.0.1:5173".to_string(),
            ],
            allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "PATCH".to_string(),
                "OPTIONS".to_string(),
            ],
            allowed_headers: vec![
                "accept".to_string(),
                "authorization".to_string(),
                "content-type".to_string(),
                "user-agent".to_string(),
                "x-csrf-token".to_string(),
                "x-requested-with".to_string(),
                "x-request-id".to_string(),
            ],
            exposed_headers: vec![
                "x-request-id".to_string(),
                "x-rate-limit-remaining".to_string(),
                "x-rate-limit-reset".to_string(),
            ],
            allow_credentials: true,
            max_age_seconds: 86400, // 24 hours
            development_mode: false,
        }
    }
}

/// Create CORS layer with configuration
pub fn create_cors_layer(config: &CorsConfig) -> CorsLayer {
    let mut cors = CorsLayer::new();
    
    // Configure origins
    if config.development_mode {
        cors = cors.allow_origin(Any);
        debug!("CORS: Development mode enabled, allowing all origins");
    } else {
        let origins: Result<Vec<HeaderValue>, _> = config
            .allowed_origins
            .iter()
            .map(|origin| origin.parse())
            .collect();
        
        match origins {
            Ok(origins) => {
                cors = cors.allow_origin(origins);
                debug!("CORS: Configured allowed origins: {:?}", config.allowed_origins);
            }
            Err(e) => {
                warn!("CORS: Invalid origin configuration: {}", e);
                cors = cors.allow_origin(Any);
            }
        }
    }
    
    // Configure methods
    let methods: Result<Vec<Method>, _> = config
        .allowed_methods
        .iter()
        .map(|method| method.parse())
        .collect();
    
    match methods {
        Ok(methods) => {
            cors = cors.allow_methods(methods);
        }
        Err(e) => {
            warn!("CORS: Invalid method configuration: {}", e);
            cors = cors.allow_methods(Any);
        }
    }
    
    // Configure headers
    let headers: Result<Vec<HeaderValue>, _> = config
        .allowed_headers
        .iter()
        .map(|header| header.parse())
        .collect();
    
    match headers {
        Ok(headers) => {
            cors = cors.allow_headers(headers);
        }
        Err(e) => {
            warn!("CORS: Invalid header configuration: {}", e);
            cors = cors.allow_headers(Any);
        }
    }
    
    // Configure exposed headers
    let exposed_headers: Result<Vec<HeaderValue>, _> = config
        .exposed_headers
        .iter()
        .map(|header| header.parse())
        .collect();
    
    if let Ok(exposed_headers) = exposed_headers {
        cors = cors.expose_headers(exposed_headers);
    }
    
    // Configure credentials
    if config.allow_credentials {
        cors = cors.allow_credentials(true);
    }
    
    // Configure max age
    cors = cors.max_age(std::time::Duration::from_secs(config.max_age_seconds));
    
    cors
}

/// Custom CORS middleware for advanced origin validation
pub async fn custom_cors_middleware(
    State(config): State<Arc<crate::config::AppConfig>>,
    request: Request,
    next: Next,
) -> Response {
    let origin = request
        .headers()
        .get(ORIGIN)
        .and_then(|value| value.to_str().ok());
    
    let method = request.method().clone();
    let is_preflight = method == Method::OPTIONS;
    
    // Handle preflight requests
    if is_preflight {
        return handle_preflight_request(origin, &config.cors_origins).await;
    }
    
    // Process the request
    let mut response = next.run(request).await;
    
    // Add CORS headers to response
    add_cors_headers(&mut response, origin, &config.cors_origins);
    
    response
}

/// Handle preflight OPTIONS requests
async fn handle_preflight_request(origin: Option<&str>, allowed_origins: &[String]) -> Response {
    let mut response = StatusCode::NO_CONTENT.into_response();
    
    // Add preflight headers
    let headers = response.headers_mut();
    
    // Access-Control-Allow-Origin
    if let Some(origin) = origin {
        if is_origin_allowed(origin, allowed_origins) {
            headers.insert(
                "access-control-allow-origin",
                HeaderValue::from_str(origin).unwrap_or_else(|_| HeaderValue::from_static("*")),
            );
        }
    }
    
    // Access-Control-Allow-Methods
    headers.insert(
        "access-control-allow-methods",
        HeaderValue::from_static("GET, POST, PUT, DELETE, PATCH, OPTIONS"),
    );
    
    // Access-Control-Allow-Headers
    headers.insert(
        "access-control-allow-headers",
        HeaderValue::from_static(
            "accept, authorization, content-type, user-agent, x-csrf-token, x-requested-with, x-request-id"
        ),
    );
    
    // Access-Control-Allow-Credentials
    headers.insert(
        "access-control-allow-credentials",
        HeaderValue::from_static("true"),
    );
    
    // Access-Control-Max-Age
    headers.insert(
        "access-control-max-age",
        HeaderValue::from_static("86400"),
    );
    
    debug!("CORS: Handled preflight request for origin: {:?}", origin);
    
    response
}

/// Add CORS headers to response
fn add_cors_headers(response: &mut Response, origin: Option<&str>, allowed_origins: &[String]) {
    let headers = response.headers_mut();
    
    // Access-Control-Allow-Origin
    if let Some(origin) = origin {
        if is_origin_allowed(origin, allowed_origins) {
            headers.insert(
                "access-control-allow-origin",
                HeaderValue::from_str(origin).unwrap_or_else(|_| HeaderValue::from_static("*")),
            );
        }
    }
    
    // Access-Control-Allow-Credentials
    headers.insert(
        "access-control-allow-credentials",
        HeaderValue::from_static("true"),
    );
    
    // Access-Control-Expose-Headers
    headers.insert(
        "access-control-expose-headers",
        HeaderValue::from_static("x-request-id, x-rate-limit-remaining, x-rate-limit-reset"),
    );
}

/// Check if origin is allowed based on allowed origins list
fn is_origin_allowed(origin: &str, allowed_origins: &[String]) -> bool {
    // Check if origin is in allowed list
    if allowed_origins.iter().any(|allowed| origin == allowed) {
        return true;
    }
    
    // Check for development patterns
    if is_development_origin(origin) {
        return true;
    }
    
    // Check for production patterns
    if is_production_origin(origin) {
        return true;
    }
    
    warn!("CORS: Blocked request from unauthorized origin: {}", origin);
    false
}

/// Get allowed origins from configuration
fn get_allowed_origins() -> HashSet<String> {
    let mut origins = HashSet::new();
    
    // Default development origins
    origins.insert("http://localhost:3000".to_string());
    origins.insert("http://localhost:5173".to_string());
    origins.insert("http://127.0.0.1:3000".to_string());
    origins.insert("http://127.0.0.1:5173".to_string());
    
    // Add origins from environment variable
    if let Ok(env_origins) = std::env::var("CORS_ALLOWED_ORIGINS") {
        for origin in env_origins.split(',') {
            origins.insert(origin.trim().to_string());
        }
    }
    
    origins
}

/// Check if origin is a development origin
fn is_development_origin(origin: &str) -> bool {
    // Allow localhost and 127.0.0.1 in development
    if std::env::var("ENVIRONMENT").unwrap_or_default() == "development" {
        return origin.starts_with("http://localhost:") 
            || origin.starts_with("http://127.0.0.1:") 
            || origin.starts_with("https://localhost:") 
            || origin.starts_with("https://127.0.0.1:");
    }
    
    false
}

/// Check if origin is a production origin
fn is_production_origin(origin: &str) -> bool {
    // Get production domain from environment
    if let Ok(production_domain) = std::env::var("PRODUCTION_DOMAIN") {
        return origin.starts_with(&format!("https://{}", production_domain))
            || origin.starts_with(&format!("https://www.{}", production_domain));
    }
    
    false
}

/// Middleware for API-specific CORS handling
pub async fn api_cors_middleware(
    request: Request,
    next: Next,
) -> Response {
    let origin = request
        .headers()
        .get(ORIGIN)
        .and_then(|value| value.to_str().ok());
    
    let method = request.method().clone();
    let path = request.uri().path();
    
    // Special handling for API endpoints
    if path.starts_with("/api/") {
        // Log API CORS requests
        debug!(
            "CORS: API request from origin: {:?}, method: {}, path: {}",
            origin, method, path
        );
        
        // Handle preflight for API endpoints
        if method == Method::OPTIONS {
            return handle_api_preflight_request(origin, path).await;
        }
    }
    
    let mut response = next.run(request).await;
    
    // Add API-specific CORS headers
    if path.starts_with("/api/") {
        add_api_cors_headers(&mut response, origin);
    }
    
    response
}

/// Handle preflight requests for API endpoints
async fn handle_api_preflight_request(origin: Option<&str>, path: &str) -> Response {
    let mut response = StatusCode::NO_CONTENT.into_response();
    let headers = response.headers_mut();
    
    // More restrictive CORS for sensitive API endpoints
    if path.starts_with("/api/auth/") || path.starts_with("/api/user/") {
        // Only allow specific origins for auth endpoints
        if let Some(origin) = origin {
            if is_trusted_origin(origin) {
                headers.insert(
                    "access-control-allow-origin",
                    HeaderValue::from_str(origin).unwrap_or_else(|_| HeaderValue::from_static("null")),
                );
            } else {
                // Reject untrusted origins for sensitive endpoints
                return StatusCode::FORBIDDEN.into_response();
            }
        }
    } else {
        // Standard CORS for other API endpoints
        let allowed_origins: Vec<String> = get_allowed_origins().into_iter().collect();
        if let Some(origin) = origin {
            if is_origin_allowed(origin, &allowed_origins) {
                headers.insert(
                    "access-control-allow-origin",
                    HeaderValue::from_str(origin).unwrap_or_else(|_| HeaderValue::from_static("*")),
                );
            }
        }
    }
    
    // API-specific headers
    headers.insert(
        "access-control-allow-methods",
        HeaderValue::from_static("GET, POST, PUT, DELETE, PATCH, OPTIONS"),
    );
    
    headers.insert(
        "access-control-allow-headers",
        HeaderValue::from_static(
            "accept, authorization, content-type, x-csrf-token, x-request-id"
        ),
    );
    
    headers.insert(
        "access-control-allow-credentials",
        HeaderValue::from_static("true"),
    );
    
    headers.insert(
        "access-control-max-age",
        HeaderValue::from_static("3600"), // Shorter cache for API endpoints
    );
    
    response
}

/// Add API-specific CORS headers
fn add_api_cors_headers(response: &mut Response, origin: Option<&str>) {
    let headers = response.headers_mut();
    
    let allowed_origins: Vec<String> = get_allowed_origins().into_iter().collect();
    if let Some(origin) = origin {
        if is_origin_allowed(origin, &allowed_origins) {
            headers.insert(
                "access-control-allow-origin",
                HeaderValue::from_str(origin).unwrap_or_else(|_| HeaderValue::from_static("*")),
            );
        }
    }
    
    headers.insert(
        "access-control-allow-credentials",
        HeaderValue::from_static("true"),
    );
    
    headers.insert(
        "access-control-expose-headers",
        HeaderValue::from_static("x-request-id, x-rate-limit-remaining, x-rate-limit-reset, authorization"),
    );
}

/// Check if origin is trusted for sensitive operations
fn is_trusted_origin(origin: &str) -> bool {
    // Get trusted origins from environment
    if let Ok(trusted_origins) = std::env::var("CORS_TRUSTED_ORIGINS") {
        for trusted_origin in trusted_origins.split(',') {
            if origin == trusted_origin.trim() {
                return true;
            }
        }
    }
    
    // Default trusted origins
    let trusted_origins = [
        "http://localhost:3000",
        "http://127.0.0.1:3000",
    ];
    
    trusted_origins.contains(&origin)
}

/// Create development CORS configuration
pub fn development_cors_config() -> CorsConfig {
    CorsConfig {
        development_mode: true,
        allow_credentials: true,
        max_age_seconds: 3600, // Shorter cache in development
        ..Default::default()
    }
}

/// Create production CORS configuration
pub fn production_cors_config(allowed_origins: Vec<String>) -> CorsConfig {
    CorsConfig {
        allowed_origins,
        development_mode: false,
        allow_credentials: true,
        max_age_seconds: 86400, // Longer cache in production
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_development_origin() {
        std::env::set_var("ENVIRONMENT", "development");
        
        assert!(is_development_origin("http://localhost:3000"));
        assert!(is_development_origin("http://127.0.0.1:5173"));
        assert!(is_development_origin("https://localhost:8080"));
        assert!(!is_development_origin("https://example.com"));
        
        std::env::remove_var("ENVIRONMENT");
    }

    #[test]
    fn test_is_production_origin() {
        std::env::set_var("PRODUCTION_DOMAIN", "example.com");
        
        assert!(is_production_origin("https://example.com"));
        assert!(is_production_origin("https://www.example.com"));
        assert!(!is_production_origin("http://example.com"));
        assert!(!is_production_origin("https://malicious.com"));
        
        std::env::remove_var("PRODUCTION_DOMAIN");
    }

    #[test]
    fn test_is_trusted_origin() {
        assert!(is_trusted_origin("http://localhost:3000"));
        assert!(is_trusted_origin("http://127.0.0.1:3000"));
        assert!(!is_trusted_origin("https://malicious.com"));
    }

    #[test]
    fn test_cors_config_default() {
        let config = CorsConfig::default();
        
        assert!(!config.development_mode);
        assert!(config.allow_credentials);
        assert_eq!(config.max_age_seconds, 86400);
        assert!(config.allowed_origins.contains(&"http://localhost:3000".to_string()));
    }

    #[test]
    fn test_development_cors_config() {
        let config = development_cors_config();
        
        assert!(config.development_mode);
        assert!(config.allow_credentials);
        assert_eq!(config.max_age_seconds, 3600);
    }

    #[test]
    fn test_production_cors_config() {
        let origins = vec!["https://example.com".to_string()];
        let config = production_cors_config(origins.clone());
        
        assert!(!config.development_mode);
        assert!(config.allow_credentials);
        assert_eq!(config.max_age_seconds, 86400);
        assert_eq!(config.allowed_origins, origins);
    }
}