use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use anyhow::{Result, anyhow};

use crate::services::rate_limiter::RateLimiter;
use crate::models::movie::{MovieResponse, SearchResponse};

#[derive(Clone)]
pub struct TMDBService {
    client: Client,
    api_key: String,
    base_url: String,
    image_base_url: String,
    rate_limiter: Arc<Mutex<RateLimiter>>,
}

#[derive(Deserialize)]
struct ConfigurationResponse {
    images: ImageConfiguration,
}

#[derive(Deserialize)]
struct ImageConfiguration {
    secure_base_url: String,
    backdrop_sizes: Vec<String>,
    poster_sizes: Vec<String>,
}

impl TMDBService {
    pub async fn new(api_key: String) -> Result<Self> {
        let client = Client::new();
        let mut service = Self {
            client,
            api_key: api_key.clone(),
            base_url: "https://api.themoviedb.org/3".to_string(),
            image_base_url: "https://image.tmdb.org/t/p/".to_string(), // Default fallback
            rate_limiter: Arc::new(Mutex::new(RateLimiter::new(40, Duration::from_secs(10)))),
        };
        
        // Only initialize config if we have a valid API key
        if api_key != "your_tmdb_api_key_here" && !api_key.is_empty() {
            if let Err(e) = service.initialize_image_config().await {
                tracing::warn!("Failed to initialize TMDB image config: {}", e);
                // Continue with default image base URL
            }
        } else {
            tracing::warn!("TMDB API key not configured, using default image URLs");
        }
        
        Ok(service)
    }
    
    async fn initialize_image_config(&mut self) -> Result<()> {
        let url = format!("{}/configuration", self.base_url);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?
            .json::<ConfigurationResponse>()
            .await?;
            
        self.image_base_url = response.images.secure_base_url;
        Ok(())
    }
    
    async fn make_request<T>(&self, endpoint: &str, params: &[(String, String)]) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        // Rate limiting
        self.rate_limiter.lock().await.acquire().await;
        
        let url = format!("{}{}", self.base_url, endpoint);
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .query(params)
            .send()
            .await?;
            
        match response.status() {
            reqwest::StatusCode::OK => {
                let result = response.json().await?;
                Ok(result)
            },
            reqwest::StatusCode::TOO_MANY_REQUESTS => {
                tracing::warn!("Rate limited by TMDB, retrying after delay");
                sleep(Duration::from_secs(1)).await;
                Box::pin(self.make_request(endpoint, params)).await
            },
            status => {
                let error_text = response.text().await.unwrap_or_default();
                Err(anyhow!("TMDB API error {}: {}", status, error_text))
            }
        }
    }
    
    pub async fn get_popular_movies(&self, page: u32) -> Result<MovieResponse> {
        let params = vec![("page".to_string(), page.to_string())];
        self.make_request("/movie/popular", &params).await
    }
    
    pub async fn get_upcoming_movies(&self, page: u32) -> Result<MovieResponse> {
        let params = vec![("page".to_string(), page.to_string())];
        self.make_request("/movie/upcoming", &params).await
    }
    
    pub async fn get_trending_movies(&self, page: u32) -> Result<MovieResponse> {
        let params = vec![("page".to_string(), page.to_string())];
        self.make_request("/trending/movie/week", &params).await
    }
    
    pub async fn search_movies(&self, query: &str, page: u32) -> Result<SearchResponse> {
        let params = vec![
            ("query".to_string(), query.to_string()),
            ("page".to_string(), page.to_string()),
        ];
        self.make_request("/search/movie", &params).await
    }
    
    pub fn get_image_url(&self, path: &str, size: &str) -> String {
        format!("{}{}{}", self.image_base_url, size, path)
    }
}