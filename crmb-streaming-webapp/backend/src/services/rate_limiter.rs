//! Rate limiting service implementing token bucket algorithm
//!
//! This module provides rate limiting functionality for external API calls,
//! specifically designed for TMDB API's 40 requests per 10 seconds limit.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Token bucket rate limiter
#[derive(Debug)]
pub struct RateLimiter {
    buckets: HashMap<String, TokenBucket>,
}

/// Individual token bucket for a specific service/endpoint
#[derive(Debug, Clone)]
pub struct TokenBucket {
    /// Maximum number of tokens the bucket can hold
    capacity: u32,
    /// Current number of tokens in the bucket
    tokens: u32,
    /// Rate at which tokens are refilled (tokens per second)
    refill_rate: f64,
    /// Last time the bucket was refilled
    last_refill: Instant,
    /// Minimum time between requests (for burst control)
    min_interval: Duration,
    /// Last request time
    last_request: Option<Instant>,
}

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests allowed
    pub max_requests: u32,
    /// Time window for the rate limit
    pub time_window: Duration,
    /// Minimum interval between requests
    pub min_interval: Option<Duration>,
    /// Enable burst protection
    pub burst_protection: bool,
}

/// Rate limiting result
#[derive(Debug, Clone)]
pub enum RateLimitResult {
    /// Request is allowed
    Allowed,
    /// Request is denied, retry after duration
    Denied { retry_after: Duration },
    /// Request is allowed but should be delayed
    Delayed { delay: Duration },
}

/// Rate limiting error
#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    #[error("Rate limit exceeded for service: {service}, retry after {retry_after:?}")]
    Exceeded {
        service: String,
        retry_after: Duration,
    },
    #[error("Invalid rate limit configuration: {0}")]
    InvalidConfig(String),
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new() -> Self {
        Self {
            buckets: HashMap::new(),
        }
    }

    /// Add a rate limit configuration for a service
    pub fn add_service(&mut self, service_name: String, config: RateLimitConfig) {
        let bucket = TokenBucket::new(config);
        self.buckets.insert(service_name, bucket);
    }

    /// Check if a request is allowed for a service
    pub fn check_rate_limit(&mut self, service_name: &str) -> RateLimitResult {
        if let Some(bucket) = self.buckets.get_mut(service_name) {
            bucket.check_request()
        } else {
            // If no rate limit is configured, allow the request
            RateLimitResult::Allowed
        }
    }

    /// Wait for rate limit if necessary and then allow the request
    pub async fn acquire_permit(&mut self, service_name: &str) -> Result<(), RateLimitError> {
        match self.check_rate_limit(service_name) {
            RateLimitResult::Allowed => {
                self.consume_token(service_name);
                Ok(())
            }
            RateLimitResult::Delayed { delay } => {
                tracing::debug!(
                    "Rate limiting: delaying request for {} by {:?}",
                    service_name,
                    delay
                );
                sleep(delay).await;
                self.consume_token(service_name);
                Ok(())
            }
            RateLimitResult::Denied { retry_after } => {
                Err(RateLimitError::Exceeded {
                    service: service_name.to_string(),
                    retry_after,
                })
            }
        }
    }

    /// Consume a token for a service
    pub fn consume_token(&mut self, service_name: &str) {
        if let Some(bucket) = self.buckets.get_mut(service_name) {
            bucket.consume_token();
        }
    }

    /// Get current token count for a service
    pub fn get_token_count(&mut self, service_name: &str) -> Option<u32> {
        self.buckets.get_mut(service_name).map(|bucket| {
            bucket.refill();
            bucket.tokens
        })
    }

    /// Get time until next token is available
    pub fn time_until_available(&mut self, service_name: &str) -> Option<Duration> {
        self.buckets.get_mut(service_name).and_then(|bucket| {
            bucket.refill();
            if bucket.tokens > 0 {
                None
            } else {
                Some(Duration::from_secs_f64(1.0 / bucket.refill_rate))
            }
        })
    }

    /// Reset rate limits for all services
    pub fn reset_all(&mut self) {
        for bucket in self.buckets.values_mut() {
            bucket.reset();
        }
    }

    /// Get rate limit status for all services
    pub fn get_status(&mut self) -> HashMap<String, RateLimitStatus> {
        self.buckets
            .iter_mut()
            .map(|(name, bucket)| {
                bucket.refill();
                let status = RateLimitStatus {
                    service_name: name.clone(),
                    tokens_available: bucket.tokens,
                    capacity: bucket.capacity,
                    refill_rate: bucket.refill_rate,
                    time_until_refill: if bucket.tokens < bucket.capacity {
                        Some(Duration::from_secs_f64(
                            (bucket.capacity - bucket.tokens) as f64 / bucket.refill_rate,
                        ))
                    } else {
                        None
                    },
                };
                (name.clone(), status)
            })
            .collect()
    }
}

impl RateLimiter {
    /// Create a new rate limiter with configuration
    pub fn new_with_config(config: &crate::config::AppConfig) -> Self {
        let mut limiter = Self::new();
        
        // Add TMDB rate limit from configuration
        limiter.add_service(
            "tmdb".to_string(),
            RateLimitConfig {
                max_requests: config.rate_limit_requests,
                time_window: Duration::from_secs(config.rate_limit_window),
                min_interval: Some(Duration::from_millis(250)), // 4 requests per second max
                burst_protection: true,
            },
        );
        
        // Add general rate limit for other services
        limiter.add_service(
            "general".to_string(),
            RateLimitConfig {
                max_requests: 100,
                time_window: Duration::from_secs(60),
                min_interval: Some(Duration::from_millis(100)),
                burst_protection: false,
            },
        );
        
        limiter
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        // Default implementation for backward compatibility
        let mut limiter = Self::new();
        
        limiter.add_service(
            "tmdb".to_string(),
            RateLimitConfig {
                max_requests: 40,
                time_window: Duration::from_secs(10),
                min_interval: Some(Duration::from_millis(250)),
                burst_protection: true,
            },
        );
        
        limiter.add_service(
            "general".to_string(),
            RateLimitConfig {
                max_requests: 100,
                time_window: Duration::from_secs(60),
                min_interval: Some(Duration::from_millis(100)),
                burst_protection: false,
            },
        );
        
        limiter
    }
}

impl TokenBucket {
    /// Create a new token bucket from configuration
    pub fn new(config: RateLimitConfig) -> Self {
        let capacity = config.max_requests;
        let refill_rate = capacity as f64 / config.time_window.as_secs_f64();
        let min_interval = config.min_interval.unwrap_or(Duration::from_millis(0));
        
        Self {
            capacity,
            tokens: capacity, // Start with full bucket
            refill_rate,
            last_refill: Instant::now(),
            min_interval,
            last_request: None,
        }
    }

    /// Check if a request can be made
    pub fn check_request(&mut self) -> RateLimitResult {
        self.refill();
        
        // Check minimum interval
        if let Some(last_request) = self.last_request {
            let time_since_last = last_request.elapsed();
            if time_since_last < self.min_interval {
                let delay = self.min_interval - time_since_last;
                return RateLimitResult::Delayed { delay };
            }
        }
        
        // Check token availability
        if self.tokens > 0 {
            RateLimitResult::Allowed
        } else {
            let retry_after = Duration::from_secs_f64(1.0 / self.refill_rate);
            RateLimitResult::Denied { retry_after }
        }
    }

    /// Consume a token
    pub fn consume_token(&mut self) {
        if self.tokens > 0 {
            self.tokens -= 1;
            self.last_request = Some(Instant::now());
        }
    }

    /// Refill tokens based on elapsed time
    pub fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        
        if elapsed > 0.0 {
            let tokens_to_add = (elapsed * self.refill_rate) as u32;
            self.tokens = (self.tokens + tokens_to_add).min(self.capacity);
            self.last_refill = now;
        }
    }

    /// Reset the bucket to full capacity
    pub fn reset(&mut self) {
        self.tokens = self.capacity;
        self.last_refill = Instant::now();
        self.last_request = None;
    }
}

/// Rate limit status information
#[derive(Debug, Clone, serde::Serialize)]
pub struct RateLimitStatus {
    pub service_name: String,
    pub tokens_available: u32,
    pub capacity: u32,
    pub refill_rate: f64,
    pub time_until_refill: Option<Duration>,
}

/// Predefined rate limit configurations
pub struct RateLimitConfigs;

impl RateLimitConfigs {
    /// TMDB API rate limit (40 requests per 10 seconds)
    pub fn tmdb() -> RateLimitConfig {
        RateLimitConfig {
            max_requests: 40,
            time_window: Duration::from_secs(10),
            min_interval: Some(Duration::from_millis(250)),
            burst_protection: true,
        }
    }

    /// Stremio addon rate limit (more lenient)
    pub fn stremio() -> RateLimitConfig {
        RateLimitConfig {
            max_requests: 100,
            time_window: Duration::from_secs(60),
            min_interval: Some(Duration::from_millis(100)),
            burst_protection: false,
        }
    }

    /// MDBList API rate limit
    pub fn mdblist() -> RateLimitConfig {
        RateLimitConfig {
            max_requests: 1000,
            time_window: Duration::from_secs(3600), // 1 hour
            min_interval: Some(Duration::from_millis(50)),
            burst_protection: false,
        }
    }

    /// Conservative rate limit for unknown APIs
    pub fn conservative() -> RateLimitConfig {
        RateLimitConfig {
            max_requests: 10,
            time_window: Duration::from_secs(60),
            min_interval: Some(Duration::from_millis(1000)),
            burst_protection: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[test]
    fn test_token_bucket_creation() {
        let config = RateLimitConfigs::tmdb();
        let bucket = TokenBucket::new(config);
        
        assert_eq!(bucket.capacity, 40);
        assert_eq!(bucket.tokens, 40);
        assert_eq!(bucket.refill_rate, 4.0); // 40 tokens per 10 seconds
    }

    #[test]
    fn test_token_consumption() {
        let config = RateLimitConfigs::tmdb();
        let mut bucket = TokenBucket::new(config);
        
        // Consume a token
        bucket.consume_token();
        assert_eq!(bucket.tokens, 39);
        
        // Check that last_request is set
        assert!(bucket.last_request.is_some());
    }

    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let mut limiter = RateLimiter::new();
        limiter.add_service("test".to_string(), RateLimitConfig {
            max_requests: 2,
            time_window: Duration::from_secs(1),
            min_interval: None,
            burst_protection: false,
        });
        
        // First request should be allowed
        assert!(matches!(
            limiter.check_rate_limit("test"),
            RateLimitResult::Allowed
        ));
        limiter.consume_token("test");
        
        // Second request should be allowed
        assert!(matches!(
            limiter.check_rate_limit("test"),
            RateLimitResult::Allowed
        ));
        limiter.consume_token("test");
        
        // Third request should be denied
        assert!(matches!(
            limiter.check_rate_limit("test"),
            RateLimitResult::Denied { .. }
        ));
    }

    #[tokio::test]
    async fn test_rate_limiter_refill() {
        let mut limiter = RateLimiter::new();
        limiter.add_service("test".to_string(), RateLimitConfig {
            max_requests: 1,
            time_window: Duration::from_millis(100),
            min_interval: None,
            burst_protection: false,
        });
        
        // Consume the only token
        limiter.consume_token("test");
        
        // Should be denied immediately
        assert!(matches!(
            limiter.check_rate_limit("test"),
            RateLimitResult::Denied { .. }
        ));
        
        // Wait for refill
        sleep(Duration::from_millis(150)).await;
        
        // Should be allowed again
        assert!(matches!(
            limiter.check_rate_limit("test"),
            RateLimitResult::Allowed
        ));
    }

    #[test]
    fn test_min_interval() {
        let config = RateLimitConfig {
            max_requests: 10,
            time_window: Duration::from_secs(1),
            min_interval: Some(Duration::from_millis(100)),
            burst_protection: true,
        };
        let mut bucket = TokenBucket::new(config);
        
        // First request should be allowed
        assert!(matches!(bucket.check_request(), RateLimitResult::Allowed));
        bucket.consume_token();
        
        // Immediate second request should be delayed
        assert!(matches!(
            bucket.check_request(),
            RateLimitResult::Delayed { .. }
        ));
    }

    #[tokio::test]
    async fn test_acquire_permit() {
        let mut limiter = RateLimiter::new();
        limiter.add_service("test".to_string(), RateLimitConfig {
            max_requests: 1,
            time_window: Duration::from_millis(100),
            min_interval: Some(Duration::from_millis(50)),
            burst_protection: false,
        });
        
        // First permit should be acquired immediately
        let start = Instant::now();
        assert!(limiter.acquire_permit("test").await.is_ok());
        let first_duration = start.elapsed();
        
        // Second permit should be delayed due to min_interval
        let start = Instant::now();
        assert!(limiter.acquire_permit("test").await.is_ok());
        let second_duration = start.elapsed();
        
        // Second request should have taken longer due to delay
        assert!(second_duration > first_duration);
        assert!(second_duration >= Duration::from_millis(40)); // Allow some tolerance
    }
}