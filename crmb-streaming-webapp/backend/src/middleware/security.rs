//! Security middleware for comprehensive application protection
//!
//! This module provides security middleware for:
//! - Security headers (HSTS, CSP, X-Frame-Options, etc.)
//! - Input validation and sanitization
//! - XSS protection
//! - CSRF protection
//! - SQL injection prevention
//! - Path traversal protection
//! - Rate limiting for security events

use axum::{
    extract::{Request, State},
    http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, Uri},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tracing::{debug, warn, error};
use regex::Regex;

use crate::{
    error::{AppError, AppResult},
    middleware::{extract_real_ip, extract_user_agent},
};

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable HSTS (HTTP Strict Transport Security)
    pub enable_hsts: bool,
    /// HSTS max age in seconds
    pub hsts_max_age: u32,
    /// Include subdomains in HSTS
    pub hsts_include_subdomains: bool,
    /// Enable CSP (Content Security Policy)
    pub enable_csp: bool,
    /// CSP policy string
    pub csp_policy: String,
    /// Enable X-Frame-Options
    pub enable_frame_options: bool,
    /// X-Frame-Options value
    pub frame_options: String,
    /// Enable X-Content-Type-Options
    pub enable_content_type_options: bool,
    /// Enable X-XSS-Protection
    pub enable_xss_protection: bool,
    /// Enable Referrer-Policy
    pub enable_referrer_policy: bool,
    /// Referrer policy value
    pub referrer_policy: String,
    /// Enable input validation
    pub enable_input_validation: bool,
    /// Enable path traversal protection
    pub enable_path_traversal_protection: bool,
    /// Enable SQL injection protection
    pub enable_sql_injection_protection: bool,
    /// Blocked user agents
    pub blocked_user_agents: Vec<String>,
    /// Blocked IP addresses
    pub blocked_ips: Vec<String>,
    /// Maximum request body size
    pub max_request_body_size: usize,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_hsts: true,
            hsts_max_age: 31536000, // 1 year
            hsts_include_subdomains: true,
            enable_csp: true,
            csp_policy: "default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self' https:; connect-src 'self' https:; media-src 'self' https:; object-src 'none'; frame-src 'none'; base-uri 'self'; form-action 'self';".to_string(),
            enable_frame_options: true,
            frame_options: "DENY".to_string(),
            enable_content_type_options: true,
            enable_xss_protection: true,
            enable_referrer_policy: true,
            referrer_policy: "strict-origin-when-cross-origin".to_string(),
            enable_input_validation: true,
            enable_path_traversal_protection: true,
            enable_sql_injection_protection: true,
            blocked_user_agents: vec![
                "bot".to_string(),
                "crawler".to_string(),
                "spider".to_string(),
                "scraper".to_string(),
            ],
            blocked_ips: vec![],
            max_request_body_size: 10 * 1024 * 1024, // 10MB
        }
    }
}

/// Security headers middleware
pub async fn security_headers_middleware(
    State(config): State<Arc<crate::config::AppConfig>>,
    request: Request,
    next: Next,
) -> Response {
    let security_config = SecurityConfig::default();
    let mut response = next.run(request).await;
    
    add_security_headers(&mut response, &security_config);
    
    response
}

/// Add security headers to response
fn add_security_headers(response: &mut Response, config: &SecurityConfig) {
    let headers = response.headers_mut();
    
    // HSTS (HTTP Strict Transport Security)
    if config.enable_hsts {
        let hsts_value = if config.hsts_include_subdomains {
            format!("max-age={}; includeSubDomains", config.hsts_max_age)
        } else {
            format!("max-age={}", config.hsts_max_age)
        };
        
        if let Ok(header_value) = HeaderValue::from_str(&hsts_value) {
            headers.insert("strict-transport-security", header_value);
        }
    }
    
    // CSP (Content Security Policy)
    if config.enable_csp {
        if let Ok(header_value) = HeaderValue::from_str(&config.csp_policy) {
            headers.insert("content-security-policy", header_value);
        }
    }
    
    // X-Frame-Options
    if config.enable_frame_options {
        if let Ok(header_value) = HeaderValue::from_str(&config.frame_options) {
            headers.insert("x-frame-options", header_value);
        }
    }
    
    // X-Content-Type-Options
    if config.enable_content_type_options {
        headers.insert("x-content-type-options", HeaderValue::from_static("nosniff"));
    }
    
    // X-XSS-Protection
    if config.enable_xss_protection {
        headers.insert("x-xss-protection", HeaderValue::from_static("1; mode=block"));
    }
    
    // Referrer-Policy
    if config.enable_referrer_policy {
        if let Ok(header_value) = HeaderValue::from_str(&config.referrer_policy) {
            headers.insert("referrer-policy", header_value);
        }
    }
    
    // Additional security headers
    headers.insert("x-permitted-cross-domain-policies", HeaderValue::from_static("none"));
    headers.insert("x-download-options", HeaderValue::from_static("noopen"));
    headers.insert("x-dns-prefetch-control", HeaderValue::from_static("off"));
}

/// Input validation middleware
pub async fn input_validation_middleware(
    State(config): State<Arc<crate::config::AppConfig>>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let security_config = SecurityConfig::default();
    
    if !security_config.enable_input_validation {
        return Ok(next.run(request).await);
    }
    
    // Validate request
    validate_request(&request, &security_config)?;
    
    Ok(next.run(request).await)
}

/// Validate incoming request for security threats
fn validate_request(request: &Request, config: &SecurityConfig) -> AppResult<()> {
    let uri = request.uri();
    let headers = request.headers();
    let method = request.method();
    
    // Path traversal protection
    if config.enable_path_traversal_protection {
        validate_path_traversal(uri)?;
    }
    
    // SQL injection protection
    if config.enable_sql_injection_protection {
        validate_sql_injection(uri)?;
    }
    
    // User agent validation
    validate_user_agent(headers, config)?;
    
    // IP address validation
    validate_ip_address(headers, config)?;
    
    // Method validation
    validate_http_method(method)?;
    
    // Header validation
    validate_headers(headers)?;
    
    Ok(())
}

/// Validate against path traversal attacks
fn validate_path_traversal(uri: &Uri) -> AppResult<()> {
    let path = uri.path();
    
    // Check for path traversal patterns
    let dangerous_patterns = [
        "../",
        "..\\",
        "..%2f",
        "..%2F",
        "..%5c",
        "..%5C",
        "%2e%2e%2f",
        "%2e%2e%5c",
        "..\\u002f",
        "..\\u005c",
    ];
    
    for pattern in &dangerous_patterns {
        if path.to_lowercase().contains(pattern) {
            warn!(
                "Path traversal attempt detected: {} in path: {}",
                pattern, path
            );
            return Err(AppError::BadRequest {
                message: "Invalid path detected".to_string(),
            });
        }
    }
    
    // Check for null bytes
    if path.contains('\0') {
        warn!("Null byte in path detected: {}", path);
        return Err(AppError::BadRequest {
            message: "Invalid characters in path".to_string(),
        });
    }
    
    Ok(())
}

/// Validate against SQL injection attacks
fn validate_sql_injection(uri: &Uri) -> AppResult<()> {
    let full_uri = uri.to_string().to_lowercase();
    
    // SQL injection patterns
    let sql_patterns = [
        "union select",
        "drop table",
        "delete from",
        "insert into",
        "update set",
        "exec(",
        "execute(",
        "sp_",
        "xp_",
        "--",
        "/*",
        "*/",
        "char(",
        "nchar(",
        "varchar(",
        "nvarchar(",
        "alter table",
        "create table",
        "drop database",
        "create database",
    ];
    
    for pattern in &sql_patterns {
        if full_uri.contains(pattern) {
            warn!(
                "SQL injection attempt detected: {} in URI: {}",
                pattern, uri
            );
            return Err(AppError::BadRequest {
                message: "Invalid request parameters".to_string(),
            });
        }
    }
    
    Ok(())
}

/// Validate user agent
fn validate_user_agent(headers: &HeaderMap, config: &SecurityConfig) -> AppResult<()> {
    if let Some(user_agent) = extract_user_agent(headers) {
        let user_agent_lower = user_agent.to_lowercase();
        
        // Check against blocked user agents
        for blocked_agent in &config.blocked_user_agents {
            if user_agent_lower.contains(&blocked_agent.to_lowercase()) {
                warn!(
                    "Blocked user agent detected: {} (blocked pattern: {})",
                    user_agent, blocked_agent
                );
                return Err(AppError::Forbidden {
                    message: "Access denied".to_string(),
                });
            }
        }
        
        // Check for suspicious patterns
        let suspicious_patterns = [
            "<script",
            "javascript:",
            "vbscript:",
            "onload=",
            "onerror=",
            "eval(",
            "alert(",
        ];
        
        for pattern in &suspicious_patterns {
            if user_agent_lower.contains(pattern) {
                warn!(
                    "Suspicious user agent detected: {} (pattern: {})",
                    user_agent, pattern
                );
                return Err(AppError::BadRequest {
                    message: "Invalid user agent".to_string(),
                });
            }
        }
    }
    
    Ok(())
}

/// Validate IP address
fn validate_ip_address(headers: &HeaderMap, config: &SecurityConfig) -> AppResult<()> {
    if let Some(ip) = extract_real_ip(headers) {
        // Check against blocked IPs
        if config.blocked_ips.contains(&ip) {
            warn!("Blocked IP address detected: {}", ip);
            return Err(AppError::Forbidden {
                message: "Access denied".to_string(),
            });
        }
        
        // Check for private IP ranges in production
        if std::env::var("ENVIRONMENT").unwrap_or_default() == "production" {
            if is_private_ip(&ip) {
                warn!("Private IP address in production: {}", ip);
                // Don't block, just log for monitoring
            }
        }
    }
    
    Ok(())
}

/// Check if IP is in private range
fn is_private_ip(ip: &str) -> bool {
    // Simple check for common private IP ranges
    ip.starts_with("192.168.") 
        || ip.starts_with("10.") 
        || ip.starts_with("172.16.") 
        || ip.starts_with("172.17.") 
        || ip.starts_with("172.18.") 
        || ip.starts_with("172.19.") 
        || ip.starts_with("172.2") 
        || ip.starts_with("172.30.") 
        || ip.starts_with("172.31.") 
        || ip == "127.0.0.1" 
        || ip == "localhost"
}

/// Validate HTTP method
fn validate_http_method(method: &Method) -> AppResult<()> {
    // Allow common HTTP methods
    let allowed_methods = [
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::DELETE,
        Method::PATCH,
        Method::HEAD,
        Method::OPTIONS,
    ];
    
    if !allowed_methods.contains(method) {
        warn!("Unusual HTTP method detected: {}", method);
        return Err(AppError::BadRequest {
            message: "Method not allowed".to_string(),
        });
    }
    
    Ok(())
}

/// Validate request headers
fn validate_headers(headers: &HeaderMap) -> AppResult<()> {
    // Check for suspicious headers
    let suspicious_headers = [
        "x-forwarded-host",
        "x-original-url",
        "x-rewrite-url",
    ];
    
    for header_name in &suspicious_headers {
        if let Some(header_value) = headers.get(*header_name) {
            if let Ok(value_str) = header_value.to_str() {
                // Check for suspicious values
                if value_str.contains("<script") 
                    || value_str.contains("javascript:") 
                    || value_str.contains("data:text/html") {
                    warn!(
                        "Suspicious header value detected: {}: {}",
                        header_name, value_str
                    );
                    return Err(AppError::BadRequest {
                        message: "Invalid header value".to_string(),
                    });
                }
            }
        }
    }
    
    Ok(())
}

/// XSS protection middleware
pub async fn xss_protection_middleware(
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Validate query parameters for XSS
    if let Some(query) = request.uri().query() {
        validate_xss_in_string(query, "query parameters")?;
    }
    
    // Validate path for XSS
    validate_xss_in_string(request.uri().path(), "path")?;
    
    Ok(next.run(request).await)
}

/// Validate string for XSS patterns
fn validate_xss_in_string(input: &str, context: &str) -> AppResult<()> {
    let input_lower = input.to_lowercase();
    
    // XSS patterns
    let xss_patterns = [
        "<script",
        "</script>",
        "javascript:",
        "vbscript:",
        "onload=",
        "onerror=",
        "onmouseover=",
        "onclick=",
        "onfocus=",
        "onblur=",
        "eval(",
        "alert(",
        "confirm(",
        "prompt(",
        "document.cookie",
        "document.write",
        "window.location",
        "<iframe",
        "<object",
        "<embed",
        "<form",
        "<img src=x onerror=",
        "<svg onload=",
    ];
    
    for pattern in &xss_patterns {
        if input_lower.contains(pattern) {
            warn!(
                "XSS attempt detected in {}: {} (pattern: {})",
                context, input, pattern
            );
            return Err(AppError::BadRequest {
                message: format!("Invalid characters in {}", context),
            });
        }
    }
    
    Ok(())
}

/// CSRF protection middleware (placeholder)
pub async fn csrf_protection_middleware(
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // TODO: Implement CSRF token validation
    // For now, just pass through
    Ok(next.run(request).await)
}

/// Rate limiting for security events
pub async fn security_rate_limit_middleware(
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // TODO: Implement security-specific rate limiting
    // This would track failed authentication attempts, suspicious requests, etc.
    Ok(next.run(request).await)
}

/// Content type validation middleware
pub async fn content_type_validation_middleware(
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let method = request.method();
    let headers = request.headers();
    
    // Validate content type for requests with body
    if method == Method::POST || method == Method::PUT || method == Method::PATCH {
        if let Some(content_type) = headers.get("content-type") {
            if let Ok(content_type_str) = content_type.to_str() {
                validate_content_type(content_type_str)?;
            }
        } else {
            // Require content type for requests with body
            return Err(AppError::BadRequest {
                message: "Content-Type header required".to_string(),
            });
        }
    }
    
    Ok(next.run(request).await)
}

/// Validate content type
fn validate_content_type(content_type: &str) -> AppResult<()> {
    let allowed_content_types = [
        "application/json",
        "application/x-www-form-urlencoded",
        "multipart/form-data",
        "text/plain",
        "application/octet-stream",
    ];
    
    let content_type_lower = content_type.to_lowercase();
    
    for allowed_type in &allowed_content_types {
        if content_type_lower.starts_with(allowed_type) {
            return Ok(());
        }
    }
    
    warn!("Invalid content type: {}", content_type);
    Err(AppError::BadRequest {
        message: "Unsupported content type".to_string(),
    })
}

/// Create development security configuration (less restrictive)
pub fn development_security_config() -> SecurityConfig {
    SecurityConfig {
        enable_hsts: false, // Disable HSTS in development
        csp_policy: "default-src 'self' 'unsafe-inline' 'unsafe-eval'; img-src 'self' data: https:; connect-src 'self' ws: wss: https:;".to_string(),
        blocked_user_agents: vec![], // Don't block user agents in development
        enable_input_validation: false, // Less strict validation in development
        ..Default::default()
    }
}

/// Create production security configuration (more restrictive)
pub fn production_security_config() -> SecurityConfig {
    SecurityConfig {
        enable_hsts: true,
        hsts_max_age: 63072000, // 2 years
        hsts_include_subdomains: true,
        csp_policy: "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self' https:; connect-src 'self' https:; media-src 'self' https:; object-src 'none'; frame-src 'none'; base-uri 'self'; form-action 'self'; upgrade-insecure-requests;".to_string(),
        frame_options: "DENY".to_string(),
        enable_input_validation: true,
        enable_path_traversal_protection: true,
        enable_sql_injection_protection: true,
        max_request_body_size: 5 * 1024 * 1024, // 5MB in production
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderMap, HeaderValue, Method, Uri};

    #[test]
    fn test_validate_path_traversal() {
        // Valid paths
        assert!(validate_path_traversal(&Uri::from_static("/api/movies")).is_ok());
        assert!(validate_path_traversal(&Uri::from_static("/static/css/style.css")).is_ok());
        
        // Invalid paths
        assert!(validate_path_traversal(&Uri::from_static("/api/../etc/passwd")).is_err());
        assert!(validate_path_traversal(&Uri::from_static("/api/..\\windows\\system32")).is_err());
        assert!(validate_path_traversal(&Uri::from_static("/api/%2e%2e%2fpasswd")).is_err());
    }

    #[test]
    fn test_validate_sql_injection() {
        // Valid URIs
        assert!(validate_sql_injection(&Uri::from_static("/api/movies?id=123")).is_ok());
        assert!(validate_sql_injection(&Uri::from_static("/api/search?q=action")).is_ok());
        
        // Invalid URIs
        assert!(validate_sql_injection(&Uri::from_static("/api/movies?id=1 UNION SELECT * FROM users")).is_err());
        assert!(validate_sql_injection(&Uri::from_static("/api/search?q='; DROP TABLE movies; --")).is_err());
    }

    #[test]
    fn test_validate_xss_in_string() {
        // Valid strings
        assert!(validate_xss_in_string("normal text", "test").is_ok());
        assert!(validate_xss_in_string("search query", "test").is_ok());
        
        // Invalid strings
        assert!(validate_xss_in_string("<script>alert('xss')</script>", "test").is_err());
        assert!(validate_xss_in_string("javascript:alert('xss')", "test").is_err());
        assert!(validate_xss_in_string("<img src=x onerror=alert('xss')>", "test").is_err());
    }

    #[test]
    fn test_is_private_ip() {
        assert!(is_private_ip("192.168.1.1"));
        assert!(is_private_ip("10.0.0.1"));
        assert!(is_private_ip("172.16.0.1"));
        assert!(is_private_ip("127.0.0.1"));
        
        assert!(!is_private_ip("8.8.8.8"));
        assert!(!is_private_ip("1.1.1.1"));
        assert!(!is_private_ip("208.67.222.222"));
    }

    #[test]
    fn test_validate_content_type() {
        // Valid content types
        assert!(validate_content_type("application/json").is_ok());
        assert!(validate_content_type("application/json; charset=utf-8").is_ok());
        assert!(validate_content_type("multipart/form-data; boundary=something").is_ok());
        
        // Invalid content types
        assert!(validate_content_type("application/xml").is_err());
        assert!(validate_content_type("text/html").is_err());
    }

    #[test]
    fn test_security_config_default() {
        let config = SecurityConfig::default();
        
        assert!(config.enable_hsts);
        assert_eq!(config.hsts_max_age, 31536000);
        assert!(config.enable_csp);
        assert!(config.enable_input_validation);
        assert_eq!(config.max_request_body_size, 10 * 1024 * 1024);
    }

    #[test]
    fn test_development_security_config() {
        let config = development_security_config();
        
        assert!(!config.enable_hsts);
        assert!(!config.enable_input_validation);
        assert!(config.blocked_user_agents.is_empty());
    }

    #[test]
    fn test_production_security_config() {
        let config = production_security_config();
        
        assert!(config.enable_hsts);
        assert_eq!(config.hsts_max_age, 63072000);
        assert!(config.enable_input_validation);
        assert_eq!(config.max_request_body_size, 5 * 1024 * 1024);
    }
}