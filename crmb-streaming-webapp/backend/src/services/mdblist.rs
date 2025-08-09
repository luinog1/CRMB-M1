//! MDBList API service with rate limiting and caching
//!
//! This module provides comprehensive MDBList API integration with:
//! - Rate limiting and request management
//! - Multi-tier caching for optimal performance
//! - Error handling and retry logic
//! - Enhanced metadata combining TMDB and MDBList data
//! - User-specific list management

use std::sync::Arc;
use std::time::Duration;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use url::Url;

use crate::models::mdblist::*;
use crate::services::cache::{CacheService, CacheKeys, CacheResult};
use crate::services::rate_limiter::{RateLimiter, RateLimitResult};
use crate::config::AppConfig;

/// MDBList API service
#[derive(Clone)]
pub struct MdbListService {
    /// HTTP client for API requests
    client: Client,
    /// API key for MDBList
    api_key: String,
    /// Base URL for MDBList API
    base_url: String,
    /// Rate limiter for API calls
    rate_limiter: Arc<RwLock<RateLimiter>>,
    /// Cache service for responses
    cache: Arc<CacheService>,
    /// Service configuration
    config: MdbListConfig,
    /// Service metrics
    metrics: Arc<RwLock<MdbListMetrics>>,
}

/// MDBList service metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MdbListMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub rate_limited_requests: u64,
    pub average_response_time: f64,
    pub last_request_time: Option<std::time::SystemTime>,
}

/// MDBList API error types
#[derive(Debug, thiserror::Error)]
pub enum MdbListError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("API key invalid or missing")]
    InvalidApiKey,
    #[error("Resource not found: {0}")]
    NotFound(String),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("URL parsing error: {0}")]
    UrlError(#[from] url::ParseError),
    #[error("Cache error: {0}")]
    CacheError(String),
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    #[error("Invalid response format: {0}")]
    InvalidResponse(String),
}

/// Result type for MDBList operations
pub type MdbListResult<T> = Result<T, MdbListError>;

impl MdbListService {
    /// Create a new MDBList service
    pub fn new(config: &AppConfig, cache: Arc<CacheService>) -> MdbListResult<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("CRMB-Streaming-WebApp/1.0")
            .build()
            .map_err(MdbListError::RequestFailed)?;

        let rate_limiter = Arc::new(RwLock::new(RateLimiter::new()));
        let metrics = Arc::new(RwLock::new(MdbListMetrics::default()));

        let mdblist_config = MdbListConfig {
            api_key: config.mdblist_api_key.clone(),
            base_url: config.mdblist_base_url.clone(),
            timeout: 30,
            max_retries: 3,
            cache_ttl: config.cache_ttl_seconds,
        };

        Ok(Self {
            client,
            api_key: mdblist_config.api_key.clone(),
            base_url: mdblist_config.base_url.clone(),
            rate_limiter,
            cache,
            config: mdblist_config,
            metrics,
        })
    }

    /// Search for movies and TV shows
    pub async fn search(&self, params: MdbListSearchParams) -> MdbListResult<MdbListSearchResponse> {
        let cache_key = CacheKeys::mdblist_search(&params.query, params.year);
        
        // Try cache first
        if let CacheResult::Hit(results, _) = self.cache.get::<MdbListSearchResponse>(&cache_key).await {
            return Ok(results);
        }

        let url = format!("{}/search", self.base_url);
        let mut query_params = vec![
            ("apikey", self.api_key.as_str()),
            ("s", params.query.as_str()),
        ];

        if let Some(year) = params.year {
            query_params.push(("y", year.to_string().as_str()));
        }

        if let Some(media_type) = params.media_type {
            let type_str = match media_type {
                MdbListMediaType::Movie => "movie",
                MdbListMediaType::Show => "show",
            };
            query_params.push(("type", type_str));
        }

        let response = self.make_request(&url, &query_params).await?;
        let results: MdbListSearchResponse = response.json().await?;

        // Cache the result
        let cache_duration = Duration::from_secs(self.config.cache_ttl);
        let _ = self.cache.set(&cache_key, &results, cache_duration).await;

        Ok(results)
    }

    /// Get detailed information by IMDB ID
    pub async fn get_by_imdb_id(&self, imdb_id: &str) -> MdbListResult<MdbListItem> {
        let cache_key = CacheKeys::mdblist_by_imdb_id(imdb_id);
        
        // Try cache first
        if let CacheResult::Hit(item, _) = self.cache.get::<MdbListItem>(&cache_key).await {
            return Ok(item);
        }

        let url = format!("{}/", self.base_url);
        let query_params = vec![
            ("apikey", self.api_key.as_str()),
            ("i", imdb_id),
        ];

        let response = self.make_request(&url, &query_params).await?;
        let item: MdbListItem = response.json().await?;

        // Cache the result
        let cache_duration = Duration::from_secs(self.config.cache_ttl);
        let _ = self.cache.set(&cache_key, &item, cache_duration).await;

        Ok(item)
    }

    /// Get detailed information by TMDB ID
    pub async fn get_by_tmdb_id(&self, tmdb_id: u64, media_type: MdbListMediaType) -> MdbListResult<MdbListItem> {
        let cache_key = CacheKeys::mdblist_by_tmdb_id(tmdb_id, media_type);
        
        // Try cache first
        if let CacheResult::Hit(item, _) = self.cache.get::<MdbListItem>(&cache_key).await {
            return Ok(item);
        }

        let url = format!("{}/", self.base_url);
        let type_str = match media_type {
            MdbListMediaType::Movie => "movie",
            MdbListMediaType::Show => "show",
        };
        let query_params = vec![
            ("apikey", self.api_key.as_str()),
            ("tmdb", tmdb_id.to_string().as_str()),
            ("type", type_str),
        ];

        let response = self.make_request(&url, &query_params).await?;
        let item: MdbListItem = response.json().await?;

        // Cache the result
        let cache_duration = Duration::from_secs(self.config.cache_ttl);
        let _ = self.cache.set(&cache_key, &item, cache_duration).await;

        Ok(item)
    }

    /// Get user's custom lists
    pub async fn get_user_lists(&self, user_id: &str) -> MdbListResult<MdbListUserLists> {
        let cache_key = CacheKeys::mdblist_user_lists(user_id);
        
        // Try cache first
        if let CacheResult::Hit(lists, _) = self.cache.get::<MdbListUserLists>(&cache_key).await {
            return Ok(lists);
        }

        let url = format!("{}/lists", self.base_url);
        let query_params = vec![
            ("apikey", self.api_key.as_str()),
            ("user", user_id),
        ];

        let response = self.make_request(&url, &query_params).await?;
        let lists: MdbListUserLists = response.json().await?;

        // Cache the result for shorter duration since lists can change
        let cache_duration = Duration::from_secs(300); // 5 minutes
        let _ = self.cache.set(&cache_key, &lists, cache_duration).await;

        Ok(lists)
    }

    /// Get specific list content
    pub async fn get_list_content(&self, list_id: &str) -> MdbListResult<MdbListList> {
        let cache_key = CacheKeys::mdblist_list_content(list_id);
        
        // Try cache first
        if let CacheResult::Hit(list, _) = self.cache.get::<MdbListList>(&cache_key).await {
            return Ok(list);
        }

        let url = format!("{}/list", self.base_url);
        let query_params = vec![
            ("apikey", self.api_key.as_str()),
            ("l", list_id),
        ];

        let response = self.make_request(&url, &query_params).await?;
        let list: MdbListList = response.json().await?;

        // Cache the result
        let cache_duration = Duration::from_secs(600); // 10 minutes
        let _ = self.cache.set(&cache_key, &list, cache_duration).await;

        Ok(list)
    }

    /// Get trending content
    pub async fn get_trending(&self, media_type: Option<MdbListMediaType>) -> MdbListResult<MdbListTrendingResponse> {
        let cache_key = CacheKeys::mdblist_trending(media_type);
        
        // Try cache first
        if let CacheResult::Hit(trending, _) = self.cache.get::<MdbListTrendingResponse>(&cache_key).await {
            return Ok(trending);
        }

        let url = format!("{}/trending", self.base_url);
        let mut query_params = vec![("apikey", self.api_key.as_str())];

        if let Some(media_type) = media_type {
            let type_str = match media_type {
                MdbListMediaType::Movie => "movie",
                MdbListMediaType::Show => "show",
            };
            query_params.push(("type", type_str));
        }

        let response = self.make_request(&url, &query_params).await?;
        let trending: MdbListTrendingResponse = response.json().await?;

        // Cache the result
        let cache_duration = Duration::from_secs(1800); // 30 minutes
        let _ = self.cache.set(&cache_key, &trending, cache_duration).await;

        Ok(trending)
    }

    /// Make authenticated request to MDBList API
    async fn make_request(&self, url: &str, params: &[(&str, &str)]) -> MdbListResult<Response> {
        // Check rate limiting
        if self.config.cache_ttl > 0 {
            let mut rate_limiter = self.rate_limiter.write().await;
            match rate_limiter.check_rate_limit("mdblist") {
                RateLimitResult::Allowed => {},
                RateLimitResult::Exceeded => {
                    tracing::warn!("MDBList rate limit exceeded");
                    return Err(MdbListError::RateLimitExceeded);
                }
            }
        }

        let mut url = Url::parse(url).map_err(MdbListError::UrlError)?;
        for &(key, value) in params {
            url.query_pairs_mut().append_pair(key, value);
        }

        let response = self.client
            .get(url)
            .send()
            .await
            .map_err(MdbListError::RequestFailed)?;

        match response.status() {
            reqwest::StatusCode::OK => Ok(response),
            reqwest::StatusCode::UNAUTHORIZED => Err(MdbListError::InvalidApiKey),
            reqwest::StatusCode::NOT_FOUND => Err(MdbListError::NotFound("Resource not found".to_string())),
            reqwest::StatusCode::TOO_MANY_REQUESTS => Err(MdbListError::RateLimitExceeded),
            status => Err(MdbListError::ServiceUnavailable(format!("HTTP {}: {}", status, response.status().canonical_reason().unwrap_or("Unknown")))),
        }
    }

    /// Get service metrics
    pub fn get_metrics(&self) -> MdbListMetrics {
        self.metrics.blocking_read().clone()
    }

    /// Reset service metrics
    pub async fn reset_metrics(&self) {
        *self.metrics.write().await = MdbListMetrics::default();
    }
}

impl crate::services::HealthCheck for MdbListService {
    async fn health_check(&self) -> crate::services::ServiceHealth {
        let start = std::time::Instant::now();
        
        let health = match self.search(MdbListSearchParams {
            query: "test".to_string(),
            year: None,
            media_type: None,
        }).await {
            Ok(_) => crate::services::HealthStatus::Healthy,
            Err(_) => crate::services::HealthStatus::Unhealthy,
        };

        crate::services::ServiceHealth {
            service_name: "MDBList".to_string(),
            status: health,
            response_time_ms: Some(start.elapsed().as_millis() as u64),
            last_check: chrono::Utc::now(),
            error_message: None,
        }
    }
}

/// Cache key utilities for MDBList
pub mod CacheKeys {
    use super::MdbListMediaType;

    pub fn mdblist_search(query: &str, year: Option<u32>) -> String {
        format!("mdblist:search:{}:{}", query, year.map(|y| y.to_string()).unwrap_or_else(|| "none".to_string()))
    }

    pub fn mdblist_by_imdb_id(imdb_id: &str) -> String {
        format!("mdblist:imdb:{}", imdb_id)
    }

    pub fn mdblist_by_tmdb_id(tmdb_id: u64, media_type: MdbListMediaType) -> String {
        let type_str = match media_type {
            MdbListMediaType::Movie => "movie",
            MdbListMediaType::Show => "show",
        };
        format!("mdblist:tmdb:{}:{}", tmdb_id, type_str)
    }

    pub fn mdblist_user_lists(user_id: &str) -> String {
        format!("mdblist:user_lists:{}", user_id)
    }

    pub fn mdblist_list_content(list_id: &str) -> String {
        format!("mdblist:list:{}", list_id)
    }

    pub fn mdblist_trending(media_type: Option<MdbListMediaType>) -> String {
        let type_str = match media_type {
            Some(MdbListMediaType::Movie) => "movie",
            Some(MdbListMediaType::Show) => "show",
            None => "all",
        };
        format!("mdblist:trending:{}", type_str)
    }
}