use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    AppState,
    models::{ApiResponse, HealthStatus},
};

/// Health check endpoint
/// GET /health
pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Check database connectivity
    let database_status = match check_database_health(&state).await {
        Ok(_) => "healthy".to_string(),
        Err(e) => {
            tracing::warn!("Database health check failed: {}", e);
            "unhealthy".to_string()
        }
    };

    // Check TMDB API connectivity
    let tmdb_status = match check_tmdb_health(&state).await {
        Ok(_) => "healthy".to_string(),
        Err(e) => {
            tracing::warn!("TMDB health check failed: {}", e);
            "unhealthy".to_string()
        }
    };

    let overall_status = if database_status == "healthy" && tmdb_status == "healthy" {
        "healthy"
    } else {
        "degraded"
    };

    let health_status = HealthStatus {
        status: overall_status.to_string(),
        timestamp,
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime: get_uptime(),
        checks: json!({
            "database": {
                "status": database_status,
                "response_time_ms": 0 // TODO: Implement actual timing
            },
            "tmdb_api": {
                "status": tmdb_status,
                "response_time_ms": 0 // TODO: Implement actual timing
            }
        }),
    };

    let response = ApiResponse {
        success: true,
        data: Some(health_status),
        error: None,
        meta: Some(json!({
            "timestamp": timestamp,
            "request_id": uuid::Uuid::new_v4().to_string()
        })),
    };

    let status_code = match overall_status {
        "healthy" => StatusCode::OK,
        _ => StatusCode::SERVICE_UNAVAILABLE,
    };

    (status_code, Json(response))
}

/// Detailed health check endpoint
/// GET /health/detailed
pub async fn detailed_health_check(State(state): State<AppState>) -> impl IntoResponse {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Perform detailed checks
    let database_check = perform_detailed_database_check(&state).await;
    let tmdb_check = perform_detailed_tmdb_check(&state).await;
    let memory_check = get_memory_usage();
    let disk_check = get_disk_usage();

    let all_healthy = database_check.status == "healthy" 
        && tmdb_check.status == "healthy"
        && memory_check.status == "healthy"
        && disk_check.status == "healthy";

    let overall_status = if all_healthy {
        "healthy"
    } else {
        "degraded"
    };

    let detailed_status = json!({
        "status": overall_status,
        "timestamp": timestamp,
        "version": env!("CARGO_PKG_VERSION"),
        "uptime": get_uptime(),
        "environment": std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()),
        "checks": {
            "database": database_check,
            "tmdb_api": tmdb_check,
            "memory": memory_check,
            "disk": disk_check
        },
        "metrics": {
            "requests_total": 0, // TODO: Implement metrics collection
            "requests_per_second": 0.0,
            "average_response_time_ms": 0.0,
            "error_rate": 0.0
        }
    });

    let response = ApiResponse {
        success: true,
        data: Some(detailed_status),
        error: None,
        meta: Some(json!({
            "timestamp": timestamp,
            "request_id": uuid::Uuid::new_v4().to_string()
        })),
    };

    let status_code = match overall_status {
        "healthy" => StatusCode::OK,
        _ => StatusCode::SERVICE_UNAVAILABLE,
    };

    (status_code, Json(response))
}

/// Readiness probe endpoint
/// GET /health/ready
pub async fn readiness_check(State(state): State<AppState>) -> impl IntoResponse {
    // Check if all critical services are ready
    let database_ready = check_database_health(&state).await.is_ok();
    let tmdb_ready = check_tmdb_health(&state).await.is_ok();

    let ready = database_ready && tmdb_ready;

    let response = json!({
        "ready": ready,
        "checks": {
            "database": database_ready,
            "tmdb_api": tmdb_ready
        }
    });

    let status_code = if ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status_code, Json(response))
}

/// Liveness probe endpoint
/// GET /health/live
pub async fn liveness_check() -> impl IntoResponse {
    let response = json!({
        "alive": true,
        "timestamp": SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });

    (StatusCode::OK, Json(response))
}

// Helper functions

async fn check_database_health(state: &AppState) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Simple query to check database connectivity
    sqlx::query("SELECT 1")
        .fetch_one(&state.database.pool)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
    
    Ok(())
}

async fn check_tmdb_health(state: &AppState) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Simple request to TMDB API to check connectivity
    let url = format!(
        "https://api.themoviedb.org/3/configuration?api_key={}",
        state.config.tmdb_api_key
    );
    
    let response = state.http_client
        .get(&url)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
    
    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("TMDB API returned status: {}", response.status()).into())
    }
}

async fn perform_detailed_database_check(state: &AppState) -> serde_json::Value {
    let start_time = std::time::Instant::now();
    
    match sqlx::query("SELECT 1")
        .fetch_one(&state.database.pool)
        .await
    {
        Ok(_) => {
            let response_time = start_time.elapsed().as_millis();
            json!({
                "status": "healthy",
                "response_time_ms": response_time,
                "connection_pool_size": state.database.pool.size(),
                "idle_connections": state.database.pool.num_idle()
            })
        }
        Err(e) => {
            json!({
                "status": "unhealthy",
                "error": e.to_string(),
                "response_time_ms": start_time.elapsed().as_millis()
            })
        }
    }
}

async fn perform_detailed_tmdb_check(state: &AppState) -> serde_json::Value {
    let start_time = std::time::Instant::now();
    
    let url = format!(
        "https://api.themoviedb.org/3/configuration?api_key={}",
        state.config.tmdb_api_key
    );
    
    match state.http_client
        .get(&url)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
    {
        Ok(response) => {
            let response_time = start_time.elapsed().as_millis();
            let status = if response.status().is_success() {
                "healthy"
            } else {
                "unhealthy"
            };
            
            json!({
                "status": status,
                "response_time_ms": response_time,
                "http_status": response.status().as_u16(),
                "rate_limit_remaining": response.headers()
                    .get("x-ratelimit-remaining")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(0)
            })
        }
        Err(e) => {
            json!({
                "status": "unhealthy",
                "error": e.to_string(),
                "response_time_ms": start_time.elapsed().as_millis()
            })
        }
    }
}

fn get_memory_usage() -> serde_json::Value {
    // Basic memory usage check
    // In a real implementation, you might use system metrics
    json!({
        "status": "healthy",
        "usage_percent": 0, // TODO: Implement actual memory monitoring
        "available_mb": 0,
        "total_mb": 0
    })
}

fn get_disk_usage() -> serde_json::Value {
    // Basic disk usage check
    // In a real implementation, you might use system metrics
    json!({
        "status": "healthy",
        "usage_percent": 0, // TODO: Implement actual disk monitoring
        "available_gb": 0,
        "total_gb": 0
    })
}

fn get_uptime() -> u64 {
    // Simple uptime calculation
    // In a real implementation, you might track application start time
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}