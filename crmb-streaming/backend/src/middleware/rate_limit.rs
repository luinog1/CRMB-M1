use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
    http::StatusCode,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct ClientRateLimit {
    requests: Vec<Instant>,
    last_cleanup: Instant,
}

type RateLimitMap = Arc<Mutex<HashMap<String, ClientRateLimit>>>;

const MAX_REQUESTS_PER_MINUTE: usize = 60;
const CLEANUP_INTERVAL: Duration = Duration::from_secs(60);

pub async fn rate_limit_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // For now, we'll use a simple in-memory rate limiter
    // In production, you'd want to use Redis or similar
    static RATE_LIMIT_MAP: std::sync::OnceLock<RateLimitMap> = std::sync::OnceLock::new();
    
    let rate_limits = RATE_LIMIT_MAP.get_or_init(|| {
        Arc::new(Mutex::new(HashMap::new()))
    });
    
    // Get client IP (simplified - in production use proper IP extraction)
    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|hv| hv.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    
    let now = Instant::now();
    let mut limits = rate_limits.lock().await;
    
    // Get or create client rate limit entry
    let client_limit = limits.entry(client_ip.clone()).or_insert_with(|| ClientRateLimit {
        requests: Vec::new(),
        last_cleanup: now,
    });
    
    // Clean up old requests if needed
    if now.duration_since(client_limit.last_cleanup) > CLEANUP_INTERVAL {
        client_limit.requests.retain(|&req_time| {
            now.duration_since(req_time) < Duration::from_secs(60)
        });
        client_limit.last_cleanup = now;
    }
    
    // Remove requests older than 1 minute
    client_limit.requests.retain(|&req_time| {
        now.duration_since(req_time) < Duration::from_secs(60)
    });
    
    // Check if client has exceeded rate limit
    if client_limit.requests.len() >= MAX_REQUESTS_PER_MINUTE {
        tracing::warn!("Rate limit exceeded for client: {}", client_ip);
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }
    
    // Add current request
    client_limit.requests.push(now);
    
    // Clean up the map periodically (remove old entries)
    if limits.len() > 1000 {
        limits.retain(|_, limit| {
            now.duration_since(limit.last_cleanup) < Duration::from_secs(300)
        });
    }
    
    drop(limits); // Release the lock
    
    // Continue with the request
    let response = next.run(request).await;
    
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, Method},
        middleware,
        response::Response,
        routing::get,
        Router,
    };
    use tower::ServiceExt;
    
    async fn test_handler() -> &'static str {
        "OK"
    }
    
    #[tokio::test]
    async fn test_rate_limiting() {
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(rate_limit_middleware));
        
        // First request should succeed
        let request = Request::builder()
            .method(Method::GET)
            .uri("/test")
            .body(Body::empty())
            .unwrap();
            
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}