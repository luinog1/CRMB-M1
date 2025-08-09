//! Request ID middleware for request tracking and correlation
//!
//! This module provides request ID middleware for:
//! - Generating unique request IDs for tracking
//! - Propagating request IDs through the request lifecycle
//! - Adding request IDs to response headers
//! - Supporting distributed tracing
//! - Request correlation across services

use axum::{
    extract::Request,
    http::{HeaderMap, HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::middleware::RequestContext;

/// Request ID header name
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// Correlation ID header name
pub const CORRELATION_ID_HEADER: &str = "x-correlation-id";

/// Trace ID header name
pub const TRACE_ID_HEADER: &str = "x-trace-id";

/// Request ID configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestIdConfig {
    /// Header name for request ID
    pub request_id_header: String,
    /// Header name for correlation ID
    pub correlation_id_header: String,
    /// Header name for trace ID
    pub trace_id_header: String,
    /// Generate new ID if not present
    pub generate_if_missing: bool,
    /// Include in response headers
    pub include_in_response: bool,
    /// Use UUID v4 format
    pub use_uuid_v4: bool,
}

impl Default for RequestIdConfig {
    fn default() -> Self {
        Self {
            request_id_header: REQUEST_ID_HEADER.to_string(),
            correlation_id_header: CORRELATION_ID_HEADER.to_string(),
            trace_id_header: TRACE_ID_HEADER.to_string(),
            generate_if_missing: true,
            include_in_response: true,
            use_uuid_v4: true,
        }
    }
}

/// Request tracking information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestTracking {
    /// Unique request ID
    pub request_id: String,
    /// Correlation ID for related requests
    pub correlation_id: Option<String>,
    /// Trace ID for distributed tracing
    pub trace_id: Option<String>,
    /// Parent request ID
    pub parent_id: Option<String>,
    /// Request depth (for nested requests)
    pub depth: u32,
}

impl RequestTracking {
    pub fn new(request_id: String) -> Self {
        Self {
            request_id,
            correlation_id: None,
            trace_id: None,
            parent_id: None,
            depth: 0,
        }
    }
    
    pub fn with_correlation_id(mut self, correlation_id: String) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }
    
    pub fn with_trace_id(mut self, trace_id: String) -> Self {
        self.trace_id = Some(trace_id);
        self
    }
    
    pub fn with_parent_id(mut self, parent_id: String) -> Self {
        self.parent_id = Some(parent_id);
        self.depth += 1;
        self
    }
}

/// Request ID middleware with comprehensive tracking
pub async fn request_id_middleware(request: Request, next: Next) -> Response {
    let config = RequestIdConfig::default();
    request_id_middleware_with_config(request, next, config).await
}

/// Request ID middleware with custom configuration
pub async fn request_id_middleware_with_config(
    mut request: Request,
    next: Next,
    config: RequestIdConfig,
) -> Response {
    let headers = request.headers();
    
    // Extract or generate request ID
    let request_id = extract_or_generate_request_id(headers, &config);
    
    // Extract correlation ID
    let correlation_id = extract_header_value(headers, &config.correlation_id_header);
    
    // Extract trace ID
    let trace_id = extract_header_value(headers, &config.trace_id_header);
    
    // Extract parent ID
    let parent_id = extract_header_value(headers, "x-parent-id");
    
    // Create request tracking
    let mut tracking = RequestTracking::new(request_id.clone());
    if let Some(correlation_id) = correlation_id {
        tracking = tracking.with_correlation_id(correlation_id);
    }
    if let Some(trace_id) = trace_id {
        tracking = tracking.with_trace_id(trace_id);
    }
    if let Some(parent_id) = parent_id {
        tracking = tracking.with_parent_id(parent_id);
    }
    
    // Add request ID to request context
    if let Some(mut context) = request.extensions_mut().get_mut::<RequestContext>() {
        context.request_id = request_id.clone();
        context.correlation_id = tracking.correlation_id.clone();
        context.trace_id = tracking.trace_id.clone();
    } else {
        let mut context = RequestContext::new();
        context.request_id = request_id.clone();
        context.correlation_id = tracking.correlation_id.clone();
        context.trace_id = tracking.trace_id.clone();
        request.extensions_mut().insert(context);
    }
    
    // Add tracking to request extensions
    request.extensions_mut().insert(tracking.clone());
    
    // Set tracing span fields
    tracing::Span::current().record("request_id", &request_id);
    if let Some(ref correlation_id) = tracking.correlation_id {
        tracing::Span::current().record("correlation_id", correlation_id);
    }
    if let Some(ref trace_id) = tracking.trace_id {
        tracing::Span::current().record("trace_id", trace_id);
    }
    
    debug!(
        "Request tracking initialized: ID={}, correlation={:?}, trace={:?}",
        request_id, tracking.correlation_id, tracking.trace_id
    );
    
    // Process request
    let mut response = next.run(request).await;
    
    // Add tracking headers to response
    if config.include_in_response {
        add_tracking_headers(&mut response, &tracking, &config);
    }
    
    response
}

/// Extract or generate request ID from headers
fn extract_or_generate_request_id(headers: &HeaderMap, config: &RequestIdConfig) -> String {
    // Try to extract existing request ID
    if let Some(existing_id) = extract_header_value(headers, &config.request_id_header) {
        if is_valid_request_id(&existing_id) {
            debug!("Using existing request ID: {}", existing_id);
            return existing_id;
        } else {
            warn!("Invalid request ID format: {}, generating new one", existing_id);
        }
    }
    
    // Generate new request ID if missing or invalid
    if config.generate_if_missing {
        let new_id = generate_request_id(config.use_uuid_v4);
        debug!("Generated new request ID: {}", new_id);
        new_id
    } else {
        "unknown".to_string()
    }
}

/// Extract header value as string
fn extract_header_value(headers: &HeaderMap, header_name: &str) -> Option<String> {
    headers
        .get(header_name)
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string())
}

/// Generate a new request ID
fn generate_request_id(use_uuid_v4: bool) -> String {
    if use_uuid_v4 {
        Uuid::new_v4().to_string()
    } else {
        // Generate a shorter ID using timestamp and random component
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let random = rand::random::<u32>();
        format!("{:x}-{:x}", timestamp, random)
    }
}

/// Validate request ID format
fn is_valid_request_id(id: &str) -> bool {
    // Check if it's a valid UUID
    if Uuid::from_str(id).is_ok() {
        return true;
    }
    
    // Check if it's a valid custom format (alphanumeric with hyphens)
    if id.len() >= 8 && id.len() <= 64 {
        return id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_');
    }
    
    false
}

/// Add tracking headers to response
fn add_tracking_headers(
    response: &mut Response,
    tracking: &RequestTracking,
    config: &RequestIdConfig,
) {
    let headers = response.headers_mut();
    
    // Add request ID
    if let Ok(header_name) = HeaderName::from_str(&config.request_id_header) {
        if let Ok(header_value) = HeaderValue::from_str(&tracking.request_id) {
            headers.insert(header_name, header_value);
        }
    }
    
    // Add correlation ID
    if let Some(ref correlation_id) = tracking.correlation_id {
        if let Ok(header_name) = HeaderName::from_str(&config.correlation_id_header) {
            if let Ok(header_value) = HeaderValue::from_str(correlation_id) {
                headers.insert(header_name, header_value);
            }
        }
    }
    
    // Add trace ID
    if let Some(ref trace_id) = tracking.trace_id {
        if let Ok(header_name) = HeaderName::from_str(&config.trace_id_header) {
            if let Ok(header_value) = HeaderValue::from_str(trace_id) {
                headers.insert(header_name, header_value);
            }
        }
    }
    
    // Add depth for nested requests
    if tracking.depth > 0 {
        if let Ok(header_value) = HeaderValue::from_str(&tracking.depth.to_string()) {
            headers.insert("x-request-depth", header_value);
        }
    }
}

/// Middleware for correlation ID propagation
pub async fn correlation_id_middleware(request: Request, next: Next) -> Response {
    let headers = request.headers();
    
    // Extract or generate correlation ID
    let correlation_id = extract_header_value(headers, CORRELATION_ID_HEADER)
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    
    // Add to request context
    let mut request = request;
    if let Some(mut context) = request.extensions_mut().get_mut::<RequestContext>() {
        context.correlation_id = Some(correlation_id.clone());
    }
    
    // Set tracing field
    tracing::Span::current().record("correlation_id", &correlation_id);
    
    // Process request
    let mut response = next.run(request).await;
    
    // Add correlation ID to response
    if let Ok(header_value) = HeaderValue::from_str(&correlation_id) {
        response.headers_mut().insert(
            HeaderName::from_str(CORRELATION_ID_HEADER).unwrap(),
            header_value,
        );
    }
    
    response
}

/// Middleware for trace ID propagation (for distributed tracing)
pub async fn trace_id_middleware(request: Request, next: Next) -> Response {
    let headers = request.headers();
    
    // Extract or generate trace ID
    let trace_id = extract_header_value(headers, TRACE_ID_HEADER)
        .unwrap_or_else(|| generate_trace_id());
    
    // Add to request context
    let mut request = request;
    if let Some(mut context) = request.extensions_mut().get_mut::<RequestContext>() {
        context.trace_id = Some(trace_id.clone());
    }
    
    // Set tracing field
    tracing::Span::current().record("trace_id", &trace_id);
    
    // Process request
    let mut response = next.run(request).await;
    
    // Add trace ID to response
    if let Ok(header_value) = HeaderValue::from_str(&trace_id) {
        response.headers_mut().insert(
            HeaderName::from_str(TRACE_ID_HEADER).unwrap(),
            header_value,
        );
    }
    
    response
}

/// Generate a trace ID compatible with distributed tracing systems
fn generate_trace_id() -> String {
    // Generate a 128-bit trace ID (32 hex characters)
    let high = rand::random::<u64>();
    let low = rand::random::<u64>();
    format!("{:016x}{:016x}", high, low)
}

/// Extract request tracking from request extensions
pub fn get_request_tracking(request: &Request) -> Option<&RequestTracking> {
    request.extensions().get::<RequestTracking>()
}

/// Extract request ID from request extensions
pub fn get_request_id(request: &Request) -> Option<String> {
    request
        .extensions()
        .get::<RequestTracking>()
        .map(|tracking| tracking.request_id.clone())
        .or_else(|| {
            request
                .extensions()
                .get::<RequestContext>()
                .map(|context| context.request_id.clone())
        })
}

/// Create a child request ID for nested operations
pub fn create_child_request_id(parent_tracking: &RequestTracking) -> RequestTracking {
    let child_id = generate_request_id(true);
    RequestTracking {
        request_id: child_id,
        correlation_id: parent_tracking.correlation_id.clone(),
        trace_id: parent_tracking.trace_id.clone(),
        parent_id: Some(parent_tracking.request_id.clone()),
        depth: parent_tracking.depth + 1,
    }
}

/// Middleware for request depth tracking
pub async fn request_depth_middleware(request: Request, next: Next) -> Response {
    let headers = request.headers();
    
    // Extract current depth
    let current_depth = extract_header_value(headers, "x-request-depth")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);
    
    // Increment depth
    let new_depth = current_depth + 1;
    
    // Add to request context
    let mut request = request;
    if let Some(mut tracking) = request.extensions_mut().get_mut::<RequestTracking>() {
        tracking.depth = new_depth;
    }
    
    // Process request
    let mut response = next.run(request).await;
    
    // Add depth to response
    if let Ok(header_value) = HeaderValue::from_str(&new_depth.to_string()) {
        response.headers_mut().insert("x-request-depth", header_value);
    }
    
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderMap, HeaderValue};

    #[test]
    fn test_generate_request_id() {
        let uuid_id = generate_request_id(true);
        assert!(Uuid::from_str(&uuid_id).is_ok());
        
        let custom_id = generate_request_id(false);
        assert!(custom_id.len() > 8);
        assert!(custom_id.contains('-'));
    }

    #[test]
    fn test_is_valid_request_id() {
        // Valid UUID
        let uuid = Uuid::new_v4().to_string();
        assert!(is_valid_request_id(&uuid));
        
        // Valid custom format
        assert!(is_valid_request_id("abc123-def456"));
        assert!(is_valid_request_id("request_12345"));
        
        // Invalid formats
        assert!(!is_valid_request_id(""));
        assert!(!is_valid_request_id("a"));
        assert!(!is_valid_request_id("invalid@id"));
    }

    #[test]
    fn test_extract_header_value() {
        let mut headers = HeaderMap::new();
        headers.insert("x-test-header", HeaderValue::from_static("test-value"));
        
        let value = extract_header_value(&headers, "x-test-header");
        assert_eq!(value, Some("test-value".to_string()));
        
        let missing = extract_header_value(&headers, "x-missing-header");
        assert_eq!(missing, None);
    }

    #[test]
    fn test_request_tracking() {
        let tracking = RequestTracking::new("test-id".to_string())
            .with_correlation_id("corr-id".to_string())
            .with_trace_id("trace-id".to_string())
            .with_parent_id("parent-id".to_string());
        
        assert_eq!(tracking.request_id, "test-id");
        assert_eq!(tracking.correlation_id, Some("corr-id".to_string()));
        assert_eq!(tracking.trace_id, Some("trace-id".to_string()));
        assert_eq!(tracking.parent_id, Some("parent-id".to_string()));
        assert_eq!(tracking.depth, 1);
    }

    #[test]
    fn test_create_child_request_id() {
        let parent = RequestTracking::new("parent-id".to_string())
            .with_correlation_id("corr-id".to_string())
            .with_trace_id("trace-id".to_string());
        
        let child = create_child_request_id(&parent);
        
        assert_ne!(child.request_id, parent.request_id);
        assert_eq!(child.correlation_id, parent.correlation_id);
        assert_eq!(child.trace_id, parent.trace_id);
        assert_eq!(child.parent_id, Some(parent.request_id));
        assert_eq!(child.depth, parent.depth + 1);
    }

    #[test]
    fn test_generate_trace_id() {
        let trace_id = generate_trace_id();
        assert_eq!(trace_id.len(), 32); // 128 bits = 32 hex characters
        assert!(trace_id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_request_id_config() {
        let config = RequestIdConfig::default();
        
        assert_eq!(config.request_id_header, REQUEST_ID_HEADER);
        assert_eq!(config.correlation_id_header, CORRELATION_ID_HEADER);
        assert_eq!(config.trace_id_header, TRACE_ID_HEADER);
        assert!(config.generate_if_missing);
        assert!(config.include_in_response);
        assert!(config.use_uuid_v4);
    }
}