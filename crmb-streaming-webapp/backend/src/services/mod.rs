//! Services module for external API integrations and business logic
//!
//! This module contains services for:
//! - TMDB API integration with rate limiting and caching
//! - Stremio addon protocol implementation
//! - MDBList integration for ratings and lists
//! - Authentication and user management
//! - Caching strategies and performance optimization

pub mod tmdb;
pub mod stremio;
pub mod cache;
pub mod rate_limiter;
pub mod auth;

use std::sync::Arc;
use tokio::sync::RwLock;

/// Service container for dependency injection
#[derive(Clone)]
pub struct Services {
    pub tmdb: Arc<tmdb::TmdbService>,
    pub stremio: Arc<stremio::StremioService>,
    pub cache: Arc<cache::CacheService>,
    pub rate_limiter: Arc<RwLock<rate_limiter::RateLimiter>>,
    pub auth: Arc<auth::AuthService>,
}

impl Services {
    /// Create new services container from AppConfig
    pub fn new(
        config: &crate::config::AppConfig,
        http_client: reqwest::Client,
        database: Arc<crate::database::Database>,
    ) -> Self {
        let cache = Arc::new(cache::CacheService::new());
        let rate_limiter = Arc::new(RwLock::new(rate_limiter::RateLimiter::new_with_config(config)));
        let tmdb = Arc::new(tmdb::TmdbService::new(
            config,
            cache.clone(),
        ).expect("Failed to create TMDB service"));
        let stremio = Arc::new(stremio::StremioService::new(
            tmdb.clone(),
            cache.clone(),
            config,
        ));
        let auth = Arc::new(auth::AuthService::new(database, config));

        Self {
            tmdb,
            stremio,
            cache,
            rate_limiter,
            auth,
        }
    }
}

/// Common error types for services
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),
    
    #[error("External API error: {0}")]
    ExternalApiError(String),
    
    #[error("Cache error: {0}")]
    CacheError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("HTTP client error: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("Invalid configuration: {0}")]
    ConfigError(String),
    
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}

/// Result type for service operations
pub type ServiceResult<T> = Result<T, ServiceError>;

impl Services {
    /// Create test services container for testing
    #[cfg(test)]
    pub fn test_services() -> Self {
        use crate::config::AppConfig;
        use std::sync::Arc;
        use tokio::sync::RwLock;

        let config = AppConfig::test_config();
        let http_client = reqwest::Client::new();
        let database = Arc::new(crate::database::Database::new_test().unwrap());

        Self::new(&config, http_client, database)
    }
}

/// Health check status for services
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServiceHealth {
    pub service_name: String,
    pub status: HealthStatus,
    pub response_time_ms: Option<u64>,
    pub last_check: chrono::DateTime<chrono::Utc>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Trait for service health checks
#[async_trait::async_trait]
pub trait HealthCheck {
    async fn health_check(&self) -> ServiceHealth;
}

/// Performance metrics for services
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServiceMetrics {
    pub service_name: String,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time_ms: f64,
    pub cache_hit_rate: f64,
    pub rate_limit_hits: u64,
    pub last_reset: chrono::DateTime<chrono::Utc>,
}

/// Trait for service metrics collection
pub trait MetricsCollector {
    fn get_metrics(&self) -> ServiceMetrics;
    fn reset_metrics(&mut self);
    fn record_request(&mut self, success: bool, response_time_ms: u64);
    fn record_cache_hit(&mut self, hit: bool);
    fn record_rate_limit_hit(&mut self);
}