//! CRMB Streaming WebApp Backend Server
//!
//! A high-performance streaming media center backend built with Rust and Axum.
//! Features include:
//! - TMDB API integration with rate limiting and caching
//! - Stremio addon protocol support
//! - JWT-based authentication
//! - Comprehensive middleware stack
//! - SQLite/PostgreSQL database support
//! - Real-time WebSocket connections

use axum::{
    extract::DefaultBodyLimit,
    http::Method,
    routing::{get, post, put, delete, patch},
    Router,
};
use std::{
    env,
    net::SocketAddr,
    sync::Arc,
    time::Duration,
};
use tokio::signal;
use tower::{
    timeout::TimeoutLayer,
    ServiceBuilder,
};
use tower_http::{
    compression::CompressionLayer,
    trace::TraceLayer,
};
use tracing::{info, warn, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod database;
mod error;
mod handlers;
mod middleware;
mod models;
mod services;
mod utils;

use crate::{
    config::AppConfig,
    database::Database,
    error::{AppError, AppResult},
    handlers::{
        auth::{login, register, refresh_token, logout, change_password},
        health::health_check,
        movies::{get_movie, search_movies, get_trending_movies, get_popular_movies, discover_movies},
        stremio::{get_manifest, get_catalog, get_meta, get_streams},
        tv::{get_tv_show, search_tv_shows, get_trending_tv, get_popular_tv, discover_tv},
        user::{
            get_profile, update_profile, get_watchlist, add_to_watchlist,
            remove_from_watchlist, check_watchlist_item, get_preferences,
            update_preferences, delete_account,
        },
    },
    middleware::{
        auth::{require_auth, optional_auth},
        cors::custom_cors_middleware,
        logging::request_logging,
        rate_limit::global_rate_limit,
        request_id::request_id_middleware,
        security::security_headers_middleware,
    },
    services::{
        auth::AuthService,
        cache::CacheService,
        rate_limiter::RateLimiter,
        stremio::StremioService,
        tmdb::TmdbService,
        Services,
    },
};

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub database: Arc<Database>,
    pub services: Arc<Services>,
}

#[tokio::main]
async fn main() -> AppResult<()> {
    // Initialize tracing
    init_tracing();
    
    info!("Starting CRMB Streaming WebApp Backend");
    
    // Load configuration
    let config = Arc::new(AppConfig::from_env()?);
    info!("Configuration loaded successfully");
    
    // Initialize database
    let database = Arc::new(Database::new(&config.database).await?);
    info!("Database connection established");
    
    // Run database migrations
    database.migrate().await?;
    info!("Database migrations completed");
    
    // Initialize services
    let services = Arc::new(initialize_services(&config).await?);
    info!("Services initialized successfully");
    
    // Create application state
    let app_state = AppState {
        config: config.clone(),
        database,
        services,
    };
    
    // Build the application router
    let app = create_app(app_state).await?;
    
    // Start the server
    let addr: SocketAddr = config.server_address
        .parse()
        .map_err(|e| AppError::Internal(format!("Invalid server address: {}", e)))?;
    info!("Server starting on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await
        .map_err(|e| AppError::Internal(format!("Failed to bind to address {}: {}", addr, e)))?;
    
    info!("🚀 CRMB Streaming WebApp Backend is running on http://{}", addr);
    
    // Start server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| AppError::Internal(format!("Server error: {}", e)))?;
    
    info!("Server shutdown complete");
    Ok(())
}

/// Initialize tracing/logging
fn init_tracing() {
    let log_level = env::var("RUST_LOG")
        .unwrap_or_else(|_| "info,crmb_streaming_webapp=debug".to_string());
    
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| log_level.into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

/// Initialize all services
async fn initialize_services(config: &AppConfig) -> AppResult<Services> {
    // Initialize HTTP client
    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| AppError::Internal(format!("Failed to create HTTP client: {}", e)))?;
    
    // Initialize services using the Services::new constructor
    let services = Services::new(
        config,
        http_client,
        database.clone(),
    ).await?;
    
    info!("All services initialized successfully");
    Ok(services)
}

/// Create the main application router
async fn create_app(state: AppState) -> AppResult<Router> {
    // Create API routes
    let api_routes = create_api_routes();
    
    // Create Stremio addon routes
    let stremio_routes = create_stremio_routes();
    
    // Build the main router
    let app = Router::new()
        // Health check endpoint
        .route("/health", get(health_check))
        
        // API routes
        .nest("/api/v1", api_routes)
        
        // Stremio addon routes
        .nest("/stremio", stremio_routes)
        
        // Add application state
        .with_state(state.clone())
        
        // Add middleware layers (order matters - last added is executed first)
        .layer(
            ServiceBuilder::new()
                // Timeout layer
                .layer(TimeoutLayer::new(Duration::from_secs(30)))
                
                // Compression
                .layer(CompressionLayer::new())
                
                // Request body size limit (10MB)
                .layer(DefaultBodyLimit::max(10 * 1024 * 1024))
                
                // Security headers
                .layer(middleware::from_fn_with_state(
                    state.config.clone(),
                    security_headers_middleware
                ))
                
                // CORS
                .layer(middleware::from_fn_with_state(
                    state.config.clone(),
                    custom_cors_middleware
                ))
                
                // Rate limiting
                .layer(middleware::from_fn_with_state(
                    state.services.rate_limiter.state.clone(),
                    global_rate_limit
                ))
                
                // Request logging
                .layer(middleware::from_fn(request_logging))
                
                // Request ID
                .layer(middleware::from_fn(request_id_middleware))
                
                // Tracing
                .layer(TraceLayer::new_for_http())
        );
    
    Ok(app)
}

/// Create API routes
fn create_api_routes() -> Router<AppState> {
    Router::new()
        // Authentication routes
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/refresh", post(refresh_token))
        .route("/auth/logout", post(logout).layer(middleware::from_fn(require_auth)))
        .route("/auth/change-password", post(change_password).layer(middleware::from_fn(require_auth)))
        
        // User routes
        .route("/user/profile", get(get_profile).layer(middleware::from_fn(require_auth)))
        .route("/user/profile", put(update_profile).layer(middleware::from_fn(require_auth)))
        .route("/user/watchlist", get(get_watchlist).layer(middleware::from_fn(require_auth)))
        .route("/user/watchlist", post(add_to_watchlist).layer(middleware::from_fn(require_auth)))
        .route("/user/watchlist/:id", delete(remove_from_watchlist).layer(middleware::from_fn(require_auth)))
        .route("/user/watchlist/:id/check", get(check_watchlist_item).layer(middleware::from_fn(require_auth)))
        .route("/user/preferences", get(get_preferences).layer(middleware::from_fn(require_auth)))
        .route("/user/preferences", put(update_preferences).layer(middleware::from_fn(require_auth)))
        .route("/user/account", delete(delete_account).layer(middleware::from_fn(require_auth)))
        
        // Movie routes
        .route("/movies/search", get(search_movies).layer(middleware::from_fn(optional_auth)))
        .route("/movies/trending", get(get_trending_movies).layer(middleware::from_fn(optional_auth)))
        .route("/movies/popular", get(get_popular_movies).layer(middleware::from_fn(optional_auth)))
        .route("/movies/discover", get(discover_movies).layer(middleware::from_fn(optional_auth)))
        .route("/movies/:id", get(get_movie).layer(middleware::from_fn(optional_auth)))
        
        // TV show routes
        .route("/tv/search", get(search_tv_shows).layer(middleware::from_fn(optional_auth)))
        .route("/tv/trending", get(get_trending_tv).layer(middleware::from_fn(optional_auth)))
        .route("/tv/popular", get(get_popular_tv).layer(middleware::from_fn(optional_auth)))
        .route("/tv/discover", get(discover_tv).layer(middleware::from_fn(optional_auth)))
        .route("/tv/:id", get(get_tv_show).layer(middleware::from_fn(optional_auth)))
}

/// Create Stremio addon routes
fn create_stremio_routes() -> Router<AppState> {
    Router::new()
        // Addon manifest
        .route("/manifest.json", get(get_manifest))
        
        // Catalog endpoints
        .route("/catalog/:type/:id.json", get(get_catalog))
        .route("/catalog/:type/:id/:extra.json", get(get_catalog))
        
        // Meta endpoints
        .route("/meta/:type/:id.json", get(get_meta))
        
        // Stream endpoints
        .route("/stream/:type/:id.json", get(get_streams))
        .route("/stream/:type/:id/:extra.json", get(get_streams))
}

/// Graceful shutdown signal handler
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };
    
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };
    
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    
    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, starting graceful shutdown");
        },
        _ = terminate => {
            info!("Received terminate signal, starting graceful shutdown");
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;
    
    /// Create a test app instance
    async fn create_test_app() -> Router {
        // Create minimal test configuration
        let config = Arc::new(AppConfig::test_config());
        
        // Create test database
        let database = Arc::new(Database::new_test().await.unwrap());
        
        // Create test services
        let services = Arc::new(Services::test_services());
        
        let state = AppState {
            config,
            database,
            services,
        };
        
        create_app(state).await.unwrap()
    }
    
    #[tokio::test]
    async fn test_health_check() {
        let app = create_test_app().await;
        
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        
        assert_eq!(response.status(), StatusCode::OK);
    }
    
    #[tokio::test]
    async fn test_stremio_manifest() {
        let app = create_test_app().await;
        
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/stremio/manifest.json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        
        assert_eq!(response.status(), StatusCode::OK);
    }
    
    #[tokio::test]
    async fn test_api_routes_require_auth() {
        let app = create_test_app().await;
        
        // Test that protected routes return 401 without auth
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/user/profile")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
    
    #[tokio::test]
    async fn test_cors_headers() {
        let app = create_test_app().await;
        
        let response = app
            .oneshot(
                Request::builder()
                    .method("OPTIONS")
                    .uri("/api/v1/movies/popular")
                    .header("Origin", "http://localhost:3000")
                    .header("Access-Control-Request-Method", "GET")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        
        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.headers().contains_key("access-control-allow-origin"));
    }
}