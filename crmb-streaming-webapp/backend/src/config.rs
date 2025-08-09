use anyhow::{Context, Result};
use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server_address: String,
    pub database_url: String,
    pub tmdb_api_key: String,
    pub tmdb_base_url: String,
    pub mdblist_api_key: String,
    pub mdblist_base_url: String,
    pub jwt_secret: String,
    pub jwt_expiration: i64,
    pub cors_origins: Vec<String>,
    pub rate_limit_requests: u32,
    pub rate_limit_window: u64,
    pub cache_ttl_seconds: u64,
    pub log_level: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        // Load .env file if it exists
        dotenv::dotenv().ok();

        let server_address = env::var("SERVER_ADDRESS")
            .unwrap_or_else(|_| "0.0.0.0:8080".to_string());

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite:./crmb.db".to_string());

        let tmdb_api_key = env::var("TMDB_API_KEY")
            .context("TMDB_API_KEY environment variable is required")?;

        let tmdb_base_url = env::var("TMDB_BASE_URL")
            .unwrap_or_else(|_| "https://api.themoviedb.org/3".to_string());

        let mdblist_api_key = env::var("MDBLIST_API_KEY")
            .context("MDBLIST_API_KEY environment variable is required")?;

        let mdblist_base_url = env::var("MDBLIST_BASE_URL")
            .unwrap_or_else(|_| "https://mdblist.com/api".to_string());

        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| {
                tracing::warn!("JWT_SECRET not set, using default (not secure for production)");
                "your-super-secret-jwt-key-change-this-in-production".to_string()
            });

        let jwt_expiration = env::var("JWT_EXPIRATION")
            .unwrap_or_else(|_| "86400".to_string()) // 24 hours
            .parse::<i64>()
            .context("JWT_EXPIRATION must be a valid number")?;

        let cors_origins = env::var("CORS_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3000,http://localhost:5173".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let rate_limit_requests = env::var("RATE_LIMIT_REQUESTS")
            .unwrap_or_else(|_| "40".to_string()) // TMDB limit: 40 requests per 10 seconds
            .parse::<u32>()
            .context("RATE_LIMIT_REQUESTS must be a valid number")?;

        let rate_limit_window = env::var("RATE_LIMIT_WINDOW")
            .unwrap_or_else(|_| "10".to_string()) // 10 seconds
            .parse::<u64>()
            .context("RATE_LIMIT_WINDOW must be a valid number")?;

        let cache_ttl_seconds = env::var("CACHE_TTL_SECONDS")
            .unwrap_or_else(|_| "3600".to_string()) // 1 hour
            .parse::<u64>()
            .context("CACHE_TTL_SECONDS must be a valid number")?;

        let log_level = env::var("LOG_LEVEL")
            .unwrap_or_else(|_| "info".to_string());

        Ok(Self {
            server_address,
            database_url,
            tmdb_api_key,
            tmdb_base_url,
            mdblist_api_key,
            mdblist_base_url,
            jwt_secret,
            jwt_expiration,
            cors_origins,
            rate_limit_requests,
            rate_limit_window,
            cache_ttl_seconds,
            log_level,
        })
    }

    pub fn validate(&self) -> Result<()> {
        if self.tmdb_api_key.is_empty() {
            anyhow::bail!("TMDB API key cannot be empty");
        }

        if self.jwt_secret.len() < 32 {
            tracing::warn!("JWT secret is shorter than 32 characters, consider using a longer secret");
        }

        if self.rate_limit_requests == 0 {
            anyhow::bail!("Rate limit requests must be greater than 0");
        }

        if self.rate_limit_window == 0 {
            anyhow::bail!("Rate limit window must be greater than 0");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    impl AppConfig {
        pub fn test_config() -> Self {
            Self {
                server_address: "0.0.0.0:8080".to_string(),
                database_url: "sqlite::memory:".to_string(),
                tmdb_api_key: "test_api_key".to_string(),
                tmdb_base_url: "https://api.themoviedb.org/3".to_string(),
                jwt_secret: "a_very_long_and_secure_jwt_secret_key_for_testing".to_string(),
                jwt_expiration: 86400,
                cors_origins: vec!["http://localhost:3000".to_string()],
                rate_limit_requests: 40,
                rate_limit_window: 10,
                cache_ttl_seconds: 3600,
                log_level: "info".to_string(),
            }
        }
    }

    #[test]
    fn test_config_validation() {
        let config = AppConfig::test_config();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_empty_api_key() {
        let mut config = AppConfig::test_config();
        config.tmdb_api_key = "".to_string();
        assert!(config.validate().is_err());
    }
}