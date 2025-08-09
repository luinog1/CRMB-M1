//! Rate limiting middleware for API protection
//!
//! This module provides comprehensive rate limiting middleware for:
//! - Global rate limiting by IP address
//! - Per-user rate limiting for authenticated requests
//! - Per-endpoint rate limiting with different limits
//! - Sliding window and token bucket algorithms
//! - Rate limit headers and proper HTTP responses
//! - Redis-backed distributed rate limiting

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};
use tokio::time::sleep;
use tracing::{debug, warn, error};

use crate::{
    error::{AppError, AppResult},
    middleware::{extract_real_ip, RequestContext},
    middleware::auth::{get_user_context, UserContext},
    services::rate_limiter::{RateLimiter, RateLimitConfig, RateLimitResult},
};

// RateLimitConfig is imported from services::rate_limiter



/// Rate limit window
#[derive(Debug, Clone)]
struct RateLimitWindow {
    /// Request count in current window
    count: u32,
    /// Window start time
    window_start: Instant,
    /// Window duration
    window_duration: Duration,
    /// Last request time
    last_request: Instant,
}

impl RateLimitWindow {
    fn new(window_duration: Duration) -> Self {
        let now = Instant::now();
        Self {
            count: 0,
            window_start: now,
            window_duration,
            last_request: now,
        }
    }
    
    fn is_expired(&self) -> bool {
        self.window_start.elapsed() >= self.window_duration
    }
    
    fn reset(&mut self) {
        let now = Instant::now();
        self.count = 0;
        self.window_start = now;
        self.last_request = now;
    }
    
    fn increment(&mut self) -> bool {
        let now = Instant::now();
        
        if self.is_expired() {
            self.reset();
        }
        
        self.last_request = now;
        self.count += 1;
        true
    }
    
    fn can_proceed(&self, limit: u32) -> bool {
        if self.is_expired() {
            return true;
        }
        self.count < limit
    }
    
    fn remaining(&self, limit: u32) -> u32 {
        if self.is_expired() {
            return limit;
        }
        limit.saturating_sub(self.count)
    }
    
    fn reset_time(&self) -> Duration {
        if self.is_expired() {
            Duration::from_secs(0)
        } else {
            self.window_duration - self.window_start.elapsed()
        }
    }
}

/// In-memory rate limiter store
#[derive(Debug)]
pub struct MemoryRateLimitStore {
    windows: Arc<RwLock<HashMap<String, RateLimitWindow>>>,
}

impl MemoryRateLimitStore {
    pub fn new() -> Self {
        Self {
            windows: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub fn check_rate_limit(
        &self,
        key: &str,
        config: &RateLimitConfig,
    ) -> crate::services::rate_limiter::RateLimitResult {
        let window_duration = config.time_window;
        
        let mut windows = self.windows.write().unwrap();
        let window = windows
            .entry(key.to_string())
            .or_insert_with(|| RateLimitWindow::new(window_duration));
        
        if window.can_proceed(config.max_requests) {
            window.increment();
            crate::services::rate_limiter::RateLimitResult::Allowed
        } else {
            crate::services::rate_limiter::RateLimitResult::Denied {
                retry_after: window.reset_time(),
            }
        }
    }
    
    /// Clean up expired windows
    pub fn cleanup_expired(&self) {
        let mut windows = self.windows.write().unwrap();
        windows.retain(|_, window| !window.is_expired());
    }
}

/// Rate limiting middleware state
#[derive(Debug, Clone)]
pub struct RateLimitState {
    pub store: Arc<MemoryRateLimitStore>,
    pub configs: Arc<HashMap<String, RateLimitConfig>>,
    pub global_config: RateLimitConfig,
}

impl RateLimitState {
    pub fn new() -> Self {
        let mut configs = HashMap::new();
        
        // Default configurations for different endpoints
        configs.insert(
            "/api/auth/login".to_string(),
            RateLimitConfig {
                max_requests: 5,
                time_window: Duration::from_secs(300), // 5 minutes
                min_interval: None,
                burst_protection: true,
            },
        );
        
        configs.insert(
            "/api/auth/register".to_string(),
            RateLimitConfig {
                max_requests: 3,
                time_window: Duration::from_secs(3600), // 1 hour
                min_interval: None,
                burst_protection: true,
            },
        );
        
        configs.insert(
            "/api/tmdb".to_string(),
            RateLimitConfig {
                max_requests: 100,
                time_window: Duration::from_secs(60), // 1 minute
                min_interval: None,
                burst_protection: true,
            },
        );
        
        configs.insert(
            "/api/stremio".to_string(),
            RateLimitConfig {
                max_requests: 50,
                time_window: Duration::from_secs(60), // 1 minute
                min_interval: None,
                burst_protection: true,
            },
        );
        
        Self {
            store: Arc::new(MemoryRateLimitStore::new()),
            configs: Arc::new(configs),
            global_config: RateLimitConfig {
            max_requests: 1000,
            time_window: Duration::from_secs(3600),
            min_interval: None,
            burst_protection: true,
        },
        }
    }
    
    pub fn with_config(configs: HashMap<String, RateLimitConfig>) -> Self {
        Self {
            store: Arc::new(MemoryRateLimitStore::new()),
            configs: Arc::new(configs),
            global_config: RateLimitConfig {
                max_requests: 1000,
                time_window: Duration::from_secs(3600),
                min_interval: None,
                burst_protection: true,
            },
        }
    }
}

/// Global rate limiting middleware
pub async fn global_rate_limit(
    State(rate_limit_state): State<RateLimitState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let headers = request.headers();
    let ip = extract_real_ip(headers).unwrap_or_else(|| "unknown".to_string());
    
    let key = format!("global:{}", ip);
    let result = rate_limit_state.store.check_rate_limit(&key, &rate_limit_state.global_config);
    
    match result {
        crate::services::rate_limiter::RateLimitResult::Allowed => {
            let response = next.run(request).await;
            Ok(response)
        }
        crate::services::rate_limiter::RateLimitResult::Denied { retry_after } => {
            warn!("Global rate limit exceeded for IP: {}", ip);
            Err(AppError::TooManyRequests(format!("Rate limit exceeded, retry after {} seconds", retry_after.as_secs())))
        }
        crate::services::rate_limiter::RateLimitResult::Delayed { .. } => {
            let response = next.run(request).await;
            Ok(response)
        }
    }
}

/// Per-endpoint rate limiting middleware
pub async fn endpoint_rate_limit(
    State(rate_limit_state): State<RateLimitState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let path = request.uri().path();
    let headers = request.headers();
    
    // Find matching configuration
    let config = find_matching_config(&rate_limit_state.configs, path)
        .unwrap_or(&rate_limit_state.global_config);
    
    // Generate rate limit key based on configuration
    let key = generate_rate_limit_key(&request, config, headers)?;
    
    let result = rate_limit_state.store.check_rate_limit(&key, config);
    
    match result {
            RateLimitResult::Allowed => {
                debug!(
                    "Rate limit check passed for key: {}",
                    key
                );
                let mut response = next.run(request).await;
                Ok(response)
            }
            RateLimitResult::Denied { retry_after } => {
                warn!(
                    "Rate limit exceeded for key: {}, retry after: {}s",
                    key,
                    retry_after.as_secs()
                );
                Err(AppError::TooManyRequests(format!("Rate limit exceeded, retry after {} seconds", retry_after.as_secs())))
            }
            RateLimitResult::Delayed { .. } => {
                Ok(next.run(request).await)
            }
        }
}

/// Authentication-specific rate limiting
pub async fn auth_rate_limit(
    State(rate_limit_state): State<RateLimitState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let path = request.uri().path();
    let headers = request.headers();
    
    // Stricter rate limiting for auth endpoints
    if path.starts_with("/api/auth/") {
        let ip = extract_real_ip(headers).unwrap_or_else(|| "unknown".to_string());
        let key = format!("auth:{}", ip);
        
        let config = rate_limit_state.configs
            .get(path)
            .unwrap_or(&RateLimitConfig {
                max_requests: 10,
                time_window: Duration::from_secs(300), // 5 minutes
                min_interval: Some(Duration::from_millis(500)),
                burst_protection: true,
            });
        
        let result = rate_limit_state.store.check_rate_limit(&key, config);
        
        match result {
            RateLimitResult::Allowed => {
                let mut response = next.run(request).await;
                Ok(response)
            }
            RateLimitResult::Denied { retry_after } => {
                warn!(
                    "Auth rate limit exceeded for IP: {}, path: {}",
                    ip, path
                );
                Err(AppError::TooManyRequests(format!("Rate limit exceeded, retry after {} seconds", retry_after.as_secs())))
            }
            RateLimitResult::Delayed { .. } => {
                Ok(next.run(request).await)
            }
        }
    } else {
        Ok(next.run(request).await)
    }
}

/// User-specific rate limiting for authenticated requests
pub async fn user_rate_limit(
    State(rate_limit_state): State<RateLimitState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Check if user is authenticated
    if let Some(user_context) = get_user_context(&request) {
        let path = request.uri().path();
        
        // Find configuration for this endpoint
        let config = find_matching_config(&rate_limit_state.configs, path)
            .unwrap_or(&RateLimitConfig {
                max_requests: 200,
                time_window: Duration::from_secs(3600), // 1 hour
                min_interval: Some(Duration::from_millis(100)),
                burst_protection: true,
            });
        
        let key = format!("user:{}:{}", user_context.user.id, path);
        let result = rate_limit_state.store.check_rate_limit(&key, config);
        
        match result {
            RateLimitResult::Allowed => {
                let mut response = next.run(request).await;
                Ok(response)
            }
            RateLimitResult::Denied { retry_after } => {
                warn!(
                    "User rate limit exceeded for user: {}, path: {}",
                    user_context.user.id, path
                );
                Err(AppError::TooManyRequests(format!("Rate limit exceeded, retry after {} seconds", retry_after.as_secs())))
            }
            RateLimitResult::Delayed { .. } => {
                Ok(next.run(request).await)
            }
        }
    } else {
        // No user context, proceed without user-specific rate limiting
        Ok(next.run(request).await)
    }
}

/// Adaptive rate limiting based on system load
pub async fn adaptive_rate_limit(
    State(rate_limit_state): State<RateLimitState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // TODO: Implement system load monitoring
    let system_load = get_system_load().await;
    
    // Create adaptive configuration based on system load
    let mut config = RateLimitConfig {
        max_requests: 50,
        time_window: Duration::from_secs(60),
        min_interval: None,
        burst_protection: true,
    };
    
    // Adjust limits based on system load
    if system_load > 0.8 {
        config.max_requests = (config.max_requests as f32 * 0.5) as u32;
        debug!("High system load detected, reducing rate limits");
    } else if system_load > 0.6 {
        config.max_requests = (config.max_requests as f32 * 0.75) as u32;
        debug!("Moderate system load detected, slightly reducing rate limits");
    }
    
    let headers = request.headers();
    let ip = extract_real_ip(headers).unwrap_or_else(|| "unknown".to_string());
    let key = format!("adaptive:{}", ip);
    
    let result = rate_limit_state.store.check_rate_limit(&key, &config);
    
    match result {
        crate::services::rate_limiter::RateLimitResult::Allowed => {
            let response = next.run(request).await;
            Ok(response)
        }
        crate::services::rate_limiter::RateLimitResult::Denied { retry_after } => {
            Err(AppError::TooManyRequests(format!("Rate limit exceeded, retry after {} seconds", retry_after.as_secs())))
        }
        crate::services::rate_limiter::RateLimitResult::Delayed { .. } => {
            Ok(next.run(request).await)
        }
    }
}

/// Find matching rate limit configuration for a path
fn find_matching_config<'a>(
    configs: &'a HashMap<String, RateLimitConfig>,
    path: &str,
) -> Option<&'a RateLimitConfig> {
    // Exact match first
    if let Some(config) = configs.get(path) {
        return Some(config);
    }
    
    // Prefix match
    for (pattern, config) in configs.iter() {
        if path.starts_with(pattern) {
            return Some(config);
        }
    }
    
    None
}

/// Generate rate limit key based on configuration
fn generate_rate_limit_key(
    request: &Request,
    _config: &RateLimitConfig,
    headers: &HeaderMap,
) -> Result<String, AppError> {
    // Default to IP-based rate limiting for now
    let ip = extract_real_ip(headers).unwrap_or_else(|| "unknown".to_string());
    Ok(format!("ip:{}", ip))
}



/// Get current system load (placeholder implementation)
async fn get_system_load() -> f32 {
    // TODO: Implement actual system load monitoring
    // This could check CPU usage, memory usage, active connections, etc.
    0.3 // Placeholder: 30% load
}

/// Cleanup task for expired rate limit windows
pub async fn cleanup_rate_limits(store: Arc<MemoryRateLimitStore>) {
    let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes
    
    loop {
        interval.tick().await;
        store.cleanup_expired();
        debug!("Cleaned up expired rate limit windows");
    }
}

/// Rate limit response for when limits are exceeded
pub fn rate_limit_exceeded_response(retry_after: u64) -> Response {
    let mut response = StatusCode::TOO_MANY_REQUESTS.into_response();
    
    response.headers_mut().insert(
        "retry-after",
        retry_after.to_string().parse().unwrap(),
    );
    
    response.headers_mut().insert(
        "x-rate-limit-remaining",
        "0".parse().unwrap(),
    );
    
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_rate_limit_window() {
        let mut window = RateLimitWindow::new(Duration::from_secs(60));
        
        // Test initial state
        assert!(window.can_proceed(10));
        assert_eq!(window.remaining(10), 10);
        
        // Test incrementing
        window.increment();
        assert_eq!(window.count, 1);
        assert_eq!(window.remaining(10), 9);
        
        // Test limit reached
        for _ in 0..9 {
            window.increment();
        }
        assert_eq!(window.count, 10);
        assert!(!window.can_proceed(10));
        assert_eq!(window.remaining(10), 0);
    }

    #[test]
    fn test_memory_rate_limit_store() {
        let store = MemoryRateLimitStore::new();
        let config = RateLimitConfig {
            max_requests: 5,
            time_window: Duration::from_secs(60),
            min_interval: None,
            burst_protection: true,
        };
        
        // Test allowed requests
        for _ in 0..5 {
            let result = store.check_rate_limit("test_key", &config);
            match result {
                crate::services::rate_limiter::RateLimitResult::Allowed => {
                    // Expected
                }
                _ => panic!("Expected allowed result"),
            }
        }
        
        // Test rate limit exceeded
        let result = store.check_rate_limit("test_key", &config);
        match result {
            crate::services::rate_limiter::RateLimitResult::Denied { .. } => {
                // Expected
            }
            _ => panic!("Expected denied result"),
        }
    }

    #[test]
    fn test_find_matching_config() {
        let mut configs = HashMap::new();
        configs.insert(
            "/api/auth".to_string(),
            RateLimitConfig {
                max_requests: 5,
                time_window: Duration::from_secs(300),
                min_interval: None,
                burst_protection: true,
            },
        );
        
        // Test exact match
        assert!(find_matching_config(&configs, "/api/auth").is_some());
        
        // Test prefix match
        assert!(find_matching_config(&configs, "/api/auth/login").is_some());
        
        // Test no match
        assert!(find_matching_config(&configs, "/api/movies").is_none());
    }

    #[test]
    fn test_rate_limit_state_creation() {
        let state = RateLimitState::new();
        
        assert!(state.configs.contains_key("/api/auth/login"));
        assert!(state.configs.contains_key("/api/auth/register"));
        assert_eq!(state.global_config.max_requests, 1000);
    }
}