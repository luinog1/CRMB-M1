//! TMDB API service with rate limiting and caching
//!
//! This module provides a comprehensive TMDB API client with:
//! - Rate limiting (40 requests per 10 seconds)
//! - Multi-tier caching for optimal performance
//! - Error handling and retry logic
//! - Request/response transformation
//! - Health monitoring

use std::sync::Arc;
use std::time::Duration;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use url::Url;

use crate::models::tmdb::*;
use crate::services::cache::{CacheService, CacheKeys, CacheResult};
use crate::services::rate_limiter::{RateLimiter, RateLimitResult};

/// TMDB API service
#[derive(Clone)]
pub struct TmdbService {
    /// HTTP client for API requests
    client: Client,
    /// API key for TMDB
    api_key: String,
    /// Base URL for TMDB API
    base_url: String,
    /// Rate limiter for API calls
    rate_limiter: Arc<RwLock<RateLimiter>>,
    /// Cache service for responses
    cache: Arc<CacheService>,
    /// Service configuration
    config: TmdbConfig,
    /// Service metrics
    metrics: Arc<RwLock<TmdbMetrics>>,
}

/// TMDB service configuration
#[derive(Debug, Clone)]
pub struct TmdbConfig {
    /// API key
    pub api_key: String,
    /// Base URL (default: https://api.themoviedb.org/3)
    pub base_url: String,
    /// Request timeout
    pub timeout: Duration,
    /// Maximum retries for failed requests
    pub max_retries: u32,
    /// Retry delay
    pub retry_delay: Duration,
    /// Enable caching
    pub enable_cache: bool,
    /// Default cache TTL
    pub cache_ttl: Duration,
    /// Enable rate limiting
    pub enable_rate_limiting: bool,
}

/// TMDB service metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TmdbMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub rate_limited_requests: u64,
    pub average_response_time: f64,
    pub last_request_time: Option<std::time::SystemTime>,
}

/// TMDB API error types
#[derive(Debug, thiserror::Error)]
pub enum TmdbError {
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

/// Result type for TMDB operations
pub type TmdbResult<T> = Result<T, TmdbError>;

/// Search parameters for movies and TV shows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchParams {
    pub query: String,
    pub page: Option<u32>,
    pub include_adult: Option<bool>,
    pub region: Option<String>,
    pub year: Option<u32>,
    pub primary_release_year: Option<u32>,
}

/// Discover parameters for content discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverParams {
    pub page: Option<u32>,
    pub sort_by: Option<String>,
    pub with_genres: Option<String>,
    pub without_genres: Option<String>,
    pub with_companies: Option<String>,
    pub with_keywords: Option<String>,
    pub release_date_gte: Option<String>,
    pub release_date_lte: Option<String>,
    pub vote_average_gte: Option<f32>,
    pub vote_average_lte: Option<f32>,
    pub vote_count_gte: Option<u32>,
    pub runtime_gte: Option<u32>,
    pub runtime_lte: Option<u32>,
}

impl TmdbService {
    /// Create a new TMDB service
    pub fn new(config: &crate::config::AppConfig, cache: Arc<CacheService>) -> TmdbResult<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("CRMB-Streaming-WebApp/1.0")
            .build()
            .map_err(TmdbError::RequestFailed)?;

        let rate_limiter = Arc::new(RwLock::new(RateLimiter::new()));
        let metrics = Arc::new(RwLock::new(TmdbMetrics::default()));

        let tmdb_config = TmdbConfig {
            api_key: config.tmdb_api_key.clone(),
            base_url: config.tmdb_base_url.clone(),
            timeout: Duration::from_secs(30),
            max_retries: 3,
            retry_delay: Duration::from_millis(500),
            enable_cache: true,
            cache_ttl: Duration::from_secs(config.cache_ttl_seconds),
            enable_rate_limiting: true,
        };

        Ok(Self {
            client,
            api_key: tmdb_config.api_key.clone(),
            base_url: tmdb_config.base_url.clone(),
            rate_limiter,
            cache,
            config: tmdb_config,
            metrics,
        })
    }

    /// Get TMDB configuration
    pub async fn get_configuration(&self) -> TmdbResult<Configuration> {
        let cache_key = "tmdb:configuration";
        
        // Try cache first
        if self.config.enable_cache {
            if let CacheResult::Hit(config, _) = self.cache.get::<Configuration>(cache_key).await {
                return Ok(config);
            }
        }

        let url = format!("{}/configuration", self.base_url);
        let response = self.make_request(&url, &[]).await?;
        let config: TmdbConfiguration = response.json().await?;

        // Cache the result
        if self.config.enable_cache {
            let _ = self.cache.set(cache_key, &config, Duration::from_secs(86400)).await; // 24 hours
        }

        Ok(config)
    }

    /// Search for movies
    pub async fn search_movies(&self, params: SearchParams) -> TmdbResult<TmdbResponse<Movie>> {
        let cache_key = CacheKeys::tmdb_search(&params.query, params.page.unwrap_or(1));
        
        // Try cache first
        if self.config.enable_cache {
            if let CacheResult::Hit(results, _) = self.cache.get::<TmdbResponse<Movie>>(&cache_key).await {
                return Ok(results);
            }
        }

        let url = format!("{}/search/movie", self.base_url);
        let query_params = self.build_search_params(&params)?;
        let response = self.make_request(&url, &query_params).await?;
        let results: TmdbResponse<Movie> = response.json().await?;

        // Cache the result
        if self.config.enable_cache {
            let _ = self.cache.set(&cache_key, &results, self.config.cache_ttl).await;
        }

        Ok(results)
    }

    /// Search for TV shows
    pub async fn search_tv(&self, params: SearchParams) -> TmdbResult<TmdbResponse<TvShow>> {
        let cache_key = format!("tmdb:search:tv:{}:{}", params.query, params.page.unwrap_or(1));
        
        // Try cache first
        if self.config.enable_cache {
            if let CacheResult::Hit(results, _) = self.cache.get::<TmdbResponse<TvShow>>(&cache_key).await {
                return Ok(results);
            }
        }

        let url = format!("{}/search/tv", self.base_url);
        let query_params = self.build_search_params(&params)?;
        let response = self.make_request(&url, &query_params).await?;
        let results: TmdbResponse<TvShow> = response.json().await?;

        // Cache the result
        if self.config.enable_cache {
            let _ = self.cache.set(&cache_key, &results, self.config.cache_ttl).await;
        }

        Ok(results)
    }

    /// Get movie details
    pub async fn get_movie_details(&self, movie_id: u32) -> TmdbResult<MovieDetails> {
        let cache_key = CacheKeys::tmdb_movie(movie_id);
        
        // Try cache first
        if self.config.enable_cache {
            if let CacheResult::Hit(details, _) = self.cache.get::<MovieDetails>(&cache_key).await {
                return Ok(details);
            }
        }

        let url = format!("{}/movie/{}", self.base_url, movie_id);
        let query_params = [("append_to_response", "credits,videos,images,similar")];
        let response = self.make_request(&url, &query_params).await?;
        
        if response.status() == 404 {
            return Err(TmdbError::NotFound(format!("Movie with ID {} not found", movie_id)));
        }

        let details: MovieDetails = response.json().await?;

        // Cache the result
        if self.config.enable_cache {
            let _ = self.cache.set(&cache_key, &details, self.config.cache_ttl).await;
        }

        Ok(details)
    }

    /// Get TV show details
    pub async fn get_tv_details(&self, tv_id: u32) -> TmdbResult<TvShowDetails> {
        let cache_key = CacheKeys::tmdb_tv(tv_id);
        
        // Try cache first
        if self.config.enable_cache {
            if let CacheResult::Hit(details, _) = self.cache.get::<TvShowDetails>(&cache_key).await {
                return Ok(details);
            }
        }

        let url = format!("{}/tv/{}", self.base_url, tv_id);
        let query_params = [("append_to_response", "credits,videos,images,similar")];
        let response = self.make_request(&url, &query_params).await?;
        
        if response.status() == 404 {
            return Err(TmdbError::NotFound(format!("TV show with ID {} not found", tv_id)));
        }

        let details: TvShowDetails = response.json().await?;

        // Cache the result
        if self.config.enable_cache {
            let _ = self.cache.set(&cache_key, &details, self.config.cache_ttl).await;
        }

        Ok(details)
    }

    /// Get trending content
    pub async fn get_trending(&self, media_type: &str, time_window: &str) -> TmdbResult<TmdbResponse<TrendingItem>> {
        let cache_key = format!("tmdb:trending:{}:{}", media_type, time_window);
        
        // Try cache first
        if self.config.enable_cache {
            if let CacheResult::Hit(results, _) = self.cache.get::<TmdbResponse<TrendingItem>>(&cache_key).await {
                return Ok(results);
            }
        }

        let url = format!("{}/trending/{}/{}", self.base_url, media_type, time_window);
        let response = self.make_request(&url, &[]).await?;
        let results: TmdbResponse<TrendingItem> = response.json().await?;

        // Cache the result with shorter TTL for trending content
        if self.config.enable_cache {
            let ttl = Duration::from_secs(1800); // 30 minutes
            let _ = self.cache.set(&cache_key, &results, ttl).await;
        }

        Ok(results)
    }

    /// Discover movies
    pub async fn discover_movies(&self, params: DiscoverParams) -> TmdbResult<TmdbResponse<Movie>> {
        let cache_key = format!("tmdb:discover:movie:{}", self.hash_discover_params(&params));
        
        // Try cache first
        if self.config.enable_cache {
            if let CacheResult::Hit(results, _) = self.cache.get::<TmdbResponse<Movie>>(&cache_key).await {
                return Ok(results);
            }
        }

        let url = format!("{}/discover/movie", self.base_url);
        let query_params = self.build_discover_params(&params)?;
        let response = self.make_request(&url, &query_params).await?;
        let results: TmdbResponse<Movie> = response.json().await?;

        // Cache the result
        if self.config.enable_cache {
            let _ = self.cache.set(&cache_key, &results, self.config.cache_ttl).await;
        }

        Ok(results)
    }

    /// Discover TV shows
    pub async fn discover_tv(&self, params: DiscoverParams) -> TmdbResult<TmdbResponse<TvShow>> {
        let cache_key = format!("tmdb:discover:tv:{}", self.hash_discover_params(&params));
        
        // Try cache first
        if self.config.enable_cache {
            if let CacheResult::Hit(results, _) = self.cache.get::<TmdbResponse<TvShow>>(&cache_key).await {
                return Ok(results);
            }
        }

        let url = format!("{}/discover/tv", self.base_url);
        let query_params = self.build_discover_params(&params)?;
        let response = self.make_request(&url, &query_params).await?;
        let results: TmdbResponse<TvShow> = response.json().await?;

        // Cache the result
        if self.config.enable_cache {
            let _ = self.cache.set(&cache_key, &results, self.config.cache_ttl).await;
        }

        Ok(results)
    }

    /// Get popular movies
    pub async fn get_popular_movies(&self, page: Option<u32>) -> TmdbResult<TmdbResponse<Movie>> {
        let page = page.unwrap_or(1);
        let cache_key = format!("tmdb:movie:popular:{}", page);
        
        // Try cache first
        if self.config.enable_cache {
            if let CacheResult::Hit(results, _) = self.cache.get::<TmdbResponse<Movie>>(&cache_key).await {
                return Ok(results);
            }
        }

        let url = format!("{}/movie/popular", self.base_url);
        let query_params = [("page", page.to_string())];
        let response = self.make_request(&url, &query_params).await?;
        let results: TmdbResponse<Movie> = response.json().await?;

        // Cache the result
        if self.config.enable_cache {
            let _ = self.cache.set(&cache_key, &results, self.config.cache_ttl).await;
        }

        Ok(results)
    }

    /// Get popular TV shows
    pub async fn get_popular_tv(&self, page: Option<u32>) -> TmdbResult<TmdbResponse<TvShow>> {
        let page = page.unwrap_or(1);
        let cache_key = format!("tmdb:tv:popular:{}", page);
        
        // Try cache first
        if self.config.enable_cache {
            if let CacheResult::Hit(results, _) = self.cache.get::<TmdbResponse<TvShow>>(&cache_key).await {
                return Ok(results);
            }
        }

        let url = format!("{}/tv/popular", self.base_url);
        let query_params = [("page", page.to_string())];
        let response = self.make_request(&url, &query_params).await?;
        let results: TmdbResponse<TvShow> = response.json().await?;

        // Cache the result
        if self.config.enable_cache {
            let _ = self.cache.set(&cache_key, &results, self.config.cache_ttl).await;
        }

        Ok(results)
    }

    /// Get service metrics
    pub async fn get_metrics(&self) -> TmdbMetrics {
        self.metrics.read().await.clone()
    }

    /// Check service health
    pub async fn health_check(&self) -> TmdbResult<bool> {
        let url = format!("{}/configuration", self.base_url);
        let response = self.make_request(&url, &[]).await?;
        Ok(response.status().is_success())
    }

    // Private helper methods

    /// Make HTTP request with rate limiting and retry logic
    async fn make_request(&self, url: &str, params: &[(impl AsRef<str>, impl AsRef<str>)]) -> TmdbResult<Response> {
        let start_time = std::time::Instant::now();
        
        // Check rate limit
        if self.config.enable_rate_limiting {
            let mut rate_limiter = self.rate_limiter.write().await;
            match rate_limiter.check_and_acquire("tmdb").await {
                RateLimitResult::Allowed => {},
                RateLimitResult::RateLimited { retry_after } => {
                    self.update_metrics(|m| m.rate_limited_requests += 1).await;
                    tracing::warn!("TMDB rate limit exceeded, retry after: {:?}", retry_after);
                    return Err(TmdbError::RateLimitExceeded);
                }
                RateLimitResult::Error(e) => {
                    tracing::error!("Rate limiter error: {}", e);
                }
            }
        }

        let mut url = Url::parse(url)?;
        
        // Add API key
        url.query_pairs_mut().append_pair("api_key", &self.api_key);
        
        // Add additional parameters
        for (key, value) in params {
            url.query_pairs_mut().append_pair(key.as_ref(), value.as_ref());
        }

        let mut retries = 0;
        loop {
            self.update_metrics(|m| m.total_requests += 1).await;
            
            let response = self.client.get(url.clone()).send().await;
            
            match response {
                Ok(resp) => {
                    let elapsed = start_time.elapsed();
                    self.update_metrics(|m| {
                        m.successful_requests += 1;
                        m.last_request_time = Some(std::time::SystemTime::now());
                        // Update average response time (simple moving average)
                        m.average_response_time = (m.average_response_time + elapsed.as_millis() as f64) / 2.0;
                    }).await;
                    
                    if resp.status().is_success() {
                        return Ok(resp);
                    } else if resp.status() == 401 {
                        return Err(TmdbError::InvalidApiKey);
                    } else if resp.status() == 404 {
                        return Ok(resp); // Let caller handle 404
                    } else if resp.status().is_server_error() && retries < self.config.max_retries {
                        retries += 1;
                        tracing::warn!("Server error {}, retrying ({}/{})", resp.status(), retries, self.config.max_retries);
                        tokio::time::sleep(self.config.retry_delay).await;
                        continue;
                    } else {
                        return Err(TmdbError::ServiceUnavailable(format!("HTTP {}", resp.status())));
                    }
                }
                Err(e) => {
                    self.update_metrics(|m| m.failed_requests += 1).await;
                    
                    if retries < self.config.max_retries {
                        retries += 1;
                        tracing::warn!("Request failed, retrying ({}/{}): {}", retries, self.config.max_retries, e);
                        tokio::time::sleep(self.config.retry_delay).await;
                        continue;
                    } else {
                        return Err(TmdbError::RequestFailed(e));
                    }
                }
            }
        }
    }

    /// Build query parameters for search requests
    fn build_search_params(&self, params: &SearchParams) -> TmdbResult<Vec<(String, String)>> {
        let mut query_params = vec![
            ("query".to_string(), params.query.clone()),
        ];

        if let Some(page) = params.page {
            query_params.push(("page".to_string(), page.to_string()));
        }

        if let Some(include_adult) = params.include_adult {
            query_params.push(("include_adult".to_string(), include_adult.to_string()));
        }

        if let Some(region) = &params.region {
            query_params.push(("region".to_string(), region.clone()));
        }

        if let Some(year) = params.year {
            query_params.push(("year".to_string(), year.to_string()));
        }

        if let Some(primary_release_year) = params.primary_release_year {
            query_params.push(("primary_release_year".to_string(), primary_release_year.to_string()));
        }

        Ok(query_params)
    }

    /// Build query parameters for discover requests
    fn build_discover_params(&self, params: &DiscoverParams) -> TmdbResult<Vec<(String, String)>> {
        let mut query_params = Vec::new();

        if let Some(page) = params.page {
            query_params.push(("page".to_string(), page.to_string()));
        }

        if let Some(sort_by) = &params.sort_by {
            query_params.push(("sort_by".to_string(), sort_by.clone()));
        }

        if let Some(with_genres) = &params.with_genres {
            query_params.push(("with_genres".to_string(), with_genres.clone()));
        }

        if let Some(without_genres) = &params.without_genres {
            query_params.push(("without_genres".to_string(), without_genres.clone()));
        }

        // Add other discover parameters as needed
        // ... (similar pattern for other optional fields)

        Ok(query_params)
    }

    /// Generate hash for discover parameters (for caching)
    fn hash_discover_params(&self, params: &DiscoverParams) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        format!("{:?}", params).hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Update service metrics
    async fn update_metrics<F>(&self, update_fn: F)
    where
        F: FnOnce(&mut TmdbMetrics),
    {
        let mut metrics = self.metrics.write().await;
        update_fn(&mut *metrics);
    }
}

impl Default for TmdbConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: "https://api.themoviedb.org/3".to_string(),
            timeout: Duration::from_secs(30),
            max_retries: 3,
            retry_delay: Duration::from_millis(1000),
            enable_cache: true,
            cache_ttl: Duration::from_secs(3600), // 1 hour
            enable_rate_limiting: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::cache::CacheService;

    fn create_test_config() -> TmdbConfig {
        TmdbConfig {
            api_key: "test_api_key".to_string(),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_tmdb_service_creation() {
        let config = create_test_config();
        let cache = Arc::new(CacheService::new());
        let service = TmdbService::new(config, cache);
        assert!(service.is_ok());
    }

    #[test]
    fn test_search_params_building() {
        let config = create_test_config();
        let cache = Arc::new(CacheService::new());
        let service = TmdbService::new(config, cache).unwrap();
        
        let params = SearchParams {
            query: "test movie".to_string(),
            page: Some(1),
            include_adult: Some(false),
            region: Some("US".to_string()),
            year: Some(2023),
            primary_release_year: None,
        };

        let query_params = service.build_search_params(&params).unwrap();
        assert!(query_params.iter().any(|(k, v)| k == "query" && v == "test movie"));
        assert!(query_params.iter().any(|(k, v)| k == "page" && v == "1"));
        assert!(query_params.iter().any(|(k, v)| k == "include_adult" && v == "false"));
        assert!(query_params.iter().any(|(k, v)| k == "region" && v == "US"));
        assert!(query_params.iter().any(|(k, v)| k == "year" && v == "2023"));
    }

    #[test]
    fn test_discover_params_hashing() {
        let config = create_test_config();
        let cache = Arc::new(CacheService::new());
        let service = TmdbService::new(config, cache).unwrap();
        
        let params1 = DiscoverParams {
            page: Some(1),
            sort_by: Some("popularity.desc".to_string()),
            with_genres: Some("28,12".to_string()),
            ..Default::default()
        };

        let params2 = DiscoverParams {
            page: Some(1),
            sort_by: Some("popularity.desc".to_string()),
            with_genres: Some("28,12".to_string()),
            ..Default::default()
        };

        let params3 = DiscoverParams {
            page: Some(2),
            sort_by: Some("popularity.desc".to_string()),
            with_genres: Some("28,12".to_string()),
            ..Default::default()
        };

        let hash1 = service.hash_discover_params(&params1);
        let hash2 = service.hash_discover_params(&params2);
        let hash3 = service.hash_discover_params(&params3);

        assert_eq!(hash1, hash2); // Same parameters should produce same hash
        assert_ne!(hash1, hash3); // Different parameters should produce different hash
    }
}

/// Default implementation for DiscoverParams
impl Default for DiscoverParams {
    fn default() -> Self {
        Self {
            page: None,
            sort_by: None,
            with_genres: None,
            without_genres: None,
            with_companies: None,
            with_keywords: None,
            release_date_gte: None,
            release_date_lte: None,
            vote_average_gte: None,
            vote_average_lte: None,
            vote_count_gte: None,
            runtime_gte: None,
            runtime_lte: None,
        }
    }
}