//! Authentication middleware for JWT token validation
//!
//! This module provides middleware for:
//! - JWT token extraction and validation
//! - User context injection into requests
//! - Optional vs required authentication
//! - Token refresh handling
//! - Session validation

use axum::{
    extract::{FromRequestParts, Request, State},
    http::{header::AUTHORIZATION, HeaderMap, StatusCode, request::Parts},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;


use crate::{
    AppState,
    error::{AppError, AppResult},
    models::{
        auth::{AuthError, TokenClaims},
        user::User,
    },
    services::auth::AuthService,
};

/// User context extracted from JWT token
#[derive(Debug, Clone)]
pub struct UserContext {
    pub user: User,
    pub token_claims: TokenClaims,
    pub is_authenticated: bool,
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for UserContext
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<UserContext>()
            .cloned()
            .ok_or(AppError::Unauthorized)
    }
}

/// Optional user context that doesn't fail if not present
#[derive(Debug, Clone)]
pub struct OptionalUserContext(pub Option<UserContext>);

#[axum::async_trait]
impl<S> FromRequestParts<S> for OptionalUserContext
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(OptionalUserContext(
            parts.extensions.get::<UserContext>().cloned(),
        ))
    }
}

/// Authentication middleware that requires valid JWT token
pub async fn require_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = extract_token_from_headers(request.headers())
        .ok_or(AppError::Unauthorized)?;

    let user_context = validate_and_get_user_context(&state.services.auth, &token).await?;
    
    // Insert both user context and token claims for compatibility
    request.extensions_mut().insert(user_context.token_claims.clone());
    request.extensions_mut().insert(user_context.token_claims.clone());
    request.extensions_mut().insert(user_context);
    
    Ok(next.run(request).await)
}

/// Authentication middleware that optionally validates JWT token
pub async fn optional_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    if let Some(token) = extract_token_from_headers(request.headers()) {
        if let Ok(user_context) = validate_and_get_user_context(&state.services.auth, &token).await {
            request.extensions_mut().insert(user_context.token_claims.clone());
            request.extensions_mut().insert(user_context);
        }
    }
    
    next.run(request).await
}

/// Admin authentication middleware that requires admin role
pub async fn require_admin(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = extract_token_from_headers(request.headers())
        .ok_or(AppError::Unauthorized)?;

    let user_context = validate_and_get_user_context(&state.services.auth, &token).await?;
    
    // Check if user has admin role (assuming we have a role field)
    // For now, we'll use a simple check - in a real app, you'd have proper role management
    if user_context.user.username != "admin" {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }
    
    request.extensions_mut().insert(user_context.token_claims.clone());
    request.extensions_mut().insert(user_context);
    
    Ok(next.run(request).await)
}

/// Rate limiting middleware for authenticated users
pub async fn auth_rate_limit(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Extract user context if available
    if let Some(token) = extract_token_from_headers(request.headers()) {
        if let Ok(user_context) = validate_and_get_user_context(&state.services.auth, &token).await {
            // TODO: Implement user-specific rate limiting
            // For now, we'll just add the user context
            request.extensions_mut().insert(user_context);
        }
    }
    
    // TODO: Implement actual rate limiting logic
    // This would check against a rate limiter service
    
    Ok(next.run(request).await)
}

/// Session validation middleware
pub async fn validate_session(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = extract_token_from_headers(request.headers())
        .ok_or(AppError::Unauthorized)?;

    // Validate token and check if session is still active
    let validation = state.services.auth.validate_token(&token).await;
    
    if !validation.valid {
        return Err(AppError::Auth(AuthError::SessionExpired));
    }
    
    // Get user and create context
    let user = state.services.auth.get_user_from_token(&token).await?;
    let token_claims = TokenClaims {
        user_id: validation.user_id.unwrap(),
        username: validation.username.unwrap(),
        email: user.email.clone(),
        iat: 0, // These would be extracted from the actual token
        exp: 0,
        iss: String::new(),
        aud: String::new(),
    };
    
    let user_context = UserContext {
        user,
        token_claims,
        is_authenticated: true,
    };
    
    request.extensions_mut().insert(user_context);
    
    Ok(next.run(request).await)
}

/// Middleware to refresh token if it's close to expiry
pub async fn auto_refresh_token(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let mut response = next.run(request).await;
    
    // TODO: Implement auto token refresh logic
    // This would check if the token is close to expiry and add a new token to the response headers
    
    Ok(response)
}

/// Extract JWT token from Authorization header
fn extract_token_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get(AUTHORIZATION)
        .and_then(|auth_header| auth_header.to_str().ok())
        .and_then(|auth_str| {
            if auth_str.starts_with("Bearer ") {
                Some(auth_str[7..].to_string())
            } else {
                None
            }
        })
}

/// Extract token from query parameter (for WebSocket connections)
fn extract_token_from_query(query: &str) -> Option<String> {
    url::form_urlencoded::parse(query.as_bytes())
        .find(|(key, _)| key == "token")
        .map(|(_, value)| value.to_string())
}

/// Validate token and get user context
async fn validate_and_get_user_context(
    auth_service: &AuthService,
    token: &str,
) -> AppResult<UserContext> {
    // Validate token
    let validation = auth_service.validate_token(token).await;
    
    if !validation.valid {
        return Err(AppError::Auth(AuthError::InvalidToken));
    }
    
    // Get user from token
    let user = auth_service.get_user_from_token(token).await?;
    
    // Create token claims (in a real implementation, these would be extracted from the token)
    let token_claims = TokenClaims {
        user_id: validation.user_id.unwrap(),
        username: validation.username.unwrap(),
        email: user.email.clone(),
        iat: 0, // These would be properly extracted from the JWT
        exp: validation.expires_at
            .unwrap_or_else(std::time::SystemTime::now)
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        iss: String::new(),
        aud: String::new(),
    };
    
    Ok(UserContext {
        user,
        token_claims,
        is_authenticated: true,
    })
}

/// Helper function to get user context from request extensions
pub fn get_user_context(request: &Request) -> Option<&UserContext> {
    request.extensions().get::<UserContext>()
}

/// Helper function to get user from request extensions
pub fn get_current_user(request: &Request) -> Option<&User> {
    get_user_context(request).map(|ctx| &ctx.user)
}

/// Helper function to check if user is authenticated
pub fn is_authenticated(request: &Request) -> bool {
    get_user_context(request)
        .map(|ctx| ctx.is_authenticated)
        .unwrap_or(false)
}

/// Helper function to get user ID from request
pub fn get_user_id(request: &Request) -> Option<u32> {
    get_user_context(request).map(|ctx| ctx.user.id)
}

/// Middleware for WebSocket authentication
pub async fn websocket_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Try to get token from query parameters for WebSocket connections
    let token = request
        .uri()
        .query()
        .and_then(extract_token_from_query)
        .or_else(|| extract_token_from_headers(request.headers()))
        .ok_or(AppError::Unauthorized)?;

    let user_context = validate_and_get_user_context(&state.services.auth, &token).await?;
    
    request.extensions_mut().insert(user_context.token_claims.clone());
    request.extensions_mut().insert(user_context);
    
    Ok(next.run(request).await)
}

/// Middleware to log authentication events
pub async fn auth_logging(
    request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path().to_string();
    let method = request.method().to_string();
    
    let response = next.run(request).await;
    
    // Log authentication events
    if let Some(user_context) = response.extensions().get::<UserContext>() {
        tracing::info!(
            "Authenticated request: {} {} by user {} (ID: {})",
            method,
            path,
            user_context.user.username,
            user_context.user.id
        );
    } else {
        tracing::debug!("Unauthenticated request: {} {}", method, path);
    }
    
    response
}

/// Middleware to handle CORS preflight for authenticated routes
pub async fn auth_cors_preflight(
    request: Request,
    next: Next,
) -> Response {
    if request.method() == axum::http::Method::OPTIONS {
        // Handle CORS preflight for authenticated routes
        return axum::response::Response::builder()
            .status(StatusCode::OK)
            .header("Access-Control-Allow-Origin", "*")
            .header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
            .header("Access-Control-Allow-Headers", "Content-Type, Authorization")
            .header("Access-Control-Max-Age", "86400")
            .body(axum::body::Body::empty())
            .unwrap();
    }
    
    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderMap, HeaderValue};

    #[test]
    fn test_extract_token_from_headers() {
        let mut headers = HeaderMap::new();
        
        // Test valid Bearer token
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_static("Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"),
        );
        
        let token = extract_token_from_headers(&headers);
        assert_eq!(token, Some("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9".to_string()));
        
        // Test invalid format
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_static("Basic dXNlcjpwYXNz"),
        );
        
        let token = extract_token_from_headers(&headers);
        assert_eq!(token, None);
        
        // Test no header
        headers.clear();
        let token = extract_token_from_headers(&headers);
        assert_eq!(token, None);
    }

    #[test]
    fn test_extract_token_from_query() {
        let query = "token=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9&other=value";
        let token = extract_token_from_query(query);
        assert_eq!(token, Some("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9".to_string()));
        
        let query = "other=value&param=test";
        let token = extract_token_from_query(query);
        assert_eq!(token, None);
        
        let query = "";
        let token = extract_token_from_query(query);
        assert_eq!(token, None);
    }

    #[test]
    fn test_user_context() {
        let user = User {
            id: 1,
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "hash".to_string(),
            created_at: "2023-01-01T00:00:00Z".to_string(),
            updated_at: "2023-01-01T00:00:00Z".to_string(),
        };
        
        let token_claims = TokenClaims {
            user_id: 1,
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            iat: 1640995200,
            exp: 1640998800,
            iss: "test".to_string(),
            aud: "test".to_string(),
        };
        
        let context = UserContext {
            user,
            token_claims,
            is_authenticated: true,
        };
        
        assert_eq!(context.user.id, 1);
        assert_eq!(context.user.username, "testuser");
        assert!(context.is_authenticated);
    }
}