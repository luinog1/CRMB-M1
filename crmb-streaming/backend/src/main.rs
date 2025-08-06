use axum::{
    routing::get,
    Router,
    middleware::from_fn,
    response::Json,
};
use tower_http::cors::CorsLayer;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;

mod handlers;
mod services;
mod models;
mod middleware;
use middleware as custom_middleware;

use handlers::movies::*;
use services::tmdb::TMDBService;

#[derive(Deserialize)]
struct MovieQuery {
    page: Option<u32>,
    query: Option<String>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    message: String,
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        message: "CRMB Streaming API is running".to_string(),
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();
    
    // Initialize tracing
    tracing_subscriber::fmt::init();
    let tmdb_api_key = std::env::var("TMDB_API_KEY")
        .expect("TMDB_API_KEY must be set in environment");
    
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3001".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid number");
    
    // Initialize TMDB service
    let tmdb_service = Arc::new(
        TMDBService::new(tmdb_api_key)
            .await
            .expect("Failed to initialize TMDB service")
    );
    
    // Build the application router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/movies/popular", get(get_popular_movies))
        .route("/api/movies/upcoming", get(get_upcoming_movies))
        .route("/api/movies/trending", get(get_trending_movies))
        .route("/api/search", get(search_movies))
        .layer(CorsLayer::permissive())
        .layer(from_fn(custom_middleware::rate_limit::rate_limit_middleware))
        .with_state(tmdb_service);
    
    let addr = format!("0.0.0.0:{}", port);
    println!("🚀 CRMB Streaming API server starting on {}", addr);
    
    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
