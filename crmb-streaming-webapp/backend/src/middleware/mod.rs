//! Middleware modules for the CRMB Streaming WebApp
//!
//! This module provides various middleware components:
//! - Authentication middleware for JWT token validation
//! - Logging middleware for request/response tracking
//! - CORS middleware for cross-origin requests
//! - Rate limiting middleware for API protection
//! - Request ID middleware for tracing
//! - Security headers middleware

pub mod auth;
pub mod cors;
pub mod logging;
pub mod rate_limit;
pub mod request_id;
pub mod security;

use axum::{
    extract::Request,
    http::HeaderValue,
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use uuid::Uuid;

/// Request context that can be shared across middleware
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub request_id: String,
    pub start_time: Instant,
    pub user_id: Option<u32>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl RequestContext {
    /// Create a new request context
    pub fn new() -> Self {
        Self {
            request_id: Uuid::new_v4().to_string(),
            start_time: Instant::now(),
            user_id: None,
            ip_address: None,
            user_agent: None,
        }
    }

    /// Get the elapsed time since request start
    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
}

impl Default for RequestContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract real IP address from request headers
pub fn extract_real_ip(headers: &axum::http::HeaderMap) -> Option<String> {
    // Try different headers in order of preference
    let ip_headers = [
        "x-forwarded-for",
        "x-real-ip",
        "cf-connecting-ip", // Cloudflare
        "x-client-ip",
        "x-forwarded",
        "forwarded-for",
        "forwarded",
    ];

    for header_name in &ip_headers {
        if let Some(header_value) = headers.get(*header_name) {
            if let Ok(value_str) = header_value.to_str() {
                // X-Forwarded-For can contain multiple IPs, take the first one
                let ip = value_str.split(',').next()?.trim();
                if !ip.is_empty() && ip != "unknown" {
                    return Some(ip.to_string());
                }
            }
        }
    }

    None
}

/// Extract user agent from request headers
pub fn extract_user_agent(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string())
}

/// Add security headers to response
pub async fn add_security_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    
    let headers = response.headers_mut();
    
    // Add security headers
    headers.insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        "X-Frame-Options",
        HeaderValue::from_static("DENY"),
    );
    headers.insert(
        "X-XSS-Protection",
        HeaderValue::from_static("1; mode=block"),
    );
    headers.insert(
        "Referrer-Policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(
        "Content-Security-Policy",
        HeaderValue::from_static(
            "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self' data:; connect-src 'self' https:; media-src 'self' https:; object-src 'none'; frame-src 'none';"
        ),
    );
    
    response
}

/// Middleware to add request ID to all responses
pub async fn add_request_id(request: Request, next: Next) -> Response {
    let request_id = Uuid::new_v4().to_string();
    
    // Add request ID to request extensions
    let mut request = request;
    request.extensions_mut().insert(request_id.clone());
    
    let mut response = next.run(request).await;
    
    // Add request ID to response headers
    response.headers_mut().insert(
        "X-Request-ID",
        HeaderValue::from_str(&request_id).unwrap_or_else(|_| HeaderValue::from_static("invalid")),
    );
    
    response
}

/// Middleware to handle request timeout
pub async fn request_timeout(
    request: Request,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    let timeout_duration = std::time::Duration::from_secs(30); // 30 seconds timeout
    
    match tokio::time::timeout(timeout_duration, next.run(request)).await {
        Ok(response) => Ok(response),
        Err(_) => {
            tracing::warn!("Request timeout after {:?}", timeout_duration);
            Err(axum::http::StatusCode::REQUEST_TIMEOUT)
        }
    }
}

/// Middleware to compress responses
pub fn compression_layer() -> tower_http::compression::CompressionLayer {
    tower_http::compression::CompressionLayer::new()
        .gzip(true)
        .deflate(true)
        .br(true)
}

/// Middleware to limit request body size
pub fn request_body_limit_layer() -> tower_http::limit::RequestBodyLimitLayer {
    // Limit request body to 10MB
    tower_http::limit::RequestBodyLimitLayer::new(10 * 1024 * 1024)
}

/// Middleware to add cache control headers
pub async fn cache_control(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    
    // Add cache control headers based on the path
    let path = response.extensions().get::<axum::http::Uri>()
        .map(|uri| uri.path())
        .unwrap_or("/");
    
    let cache_header = if path.starts_with("/api/") {
        // API responses - no cache by default
        "no-cache, no-store, must-revalidate"
    } else if path.starts_with("/static/") || path.contains(".") {
        // Static assets - cache for 1 year
        "public, max-age=31536000, immutable"
    } else {
        // Other responses - cache for 5 minutes
        "public, max-age=300"
    };
    
    response.headers_mut().insert(
        axum::http::header::CACHE_CONTROL,
        HeaderValue::from_static(cache_header),
    );
    
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderMap, HeaderName, HeaderValue};

    #[test]
    fn test_extract_real_ip() {
        let mut headers = HeaderMap::new();
        
        // Test X-Forwarded-For header
        headers.insert(
            HeaderName::from_static("x-forwarded-for"),
            HeaderValue::from_static("192.168.1.1, 10.0.0.1"),
        );
        
        let ip = extract_real_ip(&headers);
        assert_eq!(ip, Some("192.168.1.1".to_string()));
        
        // Test X-Real-IP header
        headers.clear();
        headers.insert(
            HeaderName::from_static("x-real-ip"),
            HeaderValue::from_static("203.0.113.1"),
        );
        
        let ip = extract_real_ip(&headers);
        assert_eq!(ip, Some("203.0.113.1".to_string()));
        
        // Test no headers
        headers.clear();
        let ip = extract_real_ip(&headers);
        assert_eq!(ip, None);
    }

    #[test]
    fn test_extract_user_agent() {
        let mut headers = HeaderMap::new();
        
        headers.insert(
            axum::http::header::USER_AGENT,
            HeaderValue::from_static("Mozilla/5.0 (Test Browser)"),
        );
        
        let user_agent = extract_user_agent(&headers);
        assert_eq!(user_agent, Some("Mozilla/5.0 (Test Browser)".to_string()));
        
        // Test no user agent
        headers.clear();
        let user_agent = extract_user_agent(&headers);
        assert_eq!(user_agent, None);
    }

    #[test]
    fn test_request_context() {
        let context = RequestContext::new();
        
        assert!(!context.request_id.is_empty());
        assert!(context.user_id.is_none());
        assert!(context.ip_address.is_none());
        assert!(context.user_agent.is_none());
        
        // Test elapsed time
        std::thread::sleep(std::time::Duration::from_millis(1));
        assert!(context.elapsed().as_millis() > 0);
    }
}