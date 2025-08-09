use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
};
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    AppState,
    models::{
        auth::*,
        user::{UserCreate, User},
        ApiResponse,
    },
};

/// Register a new user
/// POST /auth/register
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    tracing::info!("Registration attempt for username: {}", payload.username);

    // Validate input
    if let Err(validation_error) = validate_registration_input(&payload) {
        let response = ApiResponse {
            success: false,
            data: None::<()>,
            error: Some(json!({
                "message": validation_error,
                "code": "VALIDATION_ERROR"
            })),
            meta: None,
        };
        return (StatusCode::BAD_REQUEST, Json(response));
    }

    // Check if user already exists
    if let Ok(_) = state.database.get_user_by_email(&payload.email).await {
        let response = ApiResponse {
            success: false,
            data: None::<()>,
            error: Some(json!({
                "message": "User with this email already exists",
                "code": "USER_EXISTS"
            })),
            meta: None,
        };
        return (StatusCode::CONFLICT, Json(response));
    }

    if let Ok(_) = state.database.get_user_by_username(&payload.username).await {
        let response = ApiResponse {
            success: false,
            data: None::<()>,
            error: Some(json!({
                "message": "User with this username already exists",
                "code": "USERNAME_EXISTS"
            })),
            meta: None,
        };
        return (StatusCode::CONFLICT, Json(response));
    }

    // Hash password
    let password_hash = match hash(&payload.password, DEFAULT_COST) {
        Ok(hash) => hash,
        Err(e) => {
            tracing::error!("Password hashing failed: {}", e);
            let response = ApiResponse {
                success: false,
                data: None::<()>,
                error: Some(json!({
                    "message": "Internal server error",
                    "code": "INTERNAL_ERROR"
                })),
                meta: None,
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response));
        }
    };

    // Create user
    let create_request = UserCreate {
        username: payload.username.clone(),
        email: payload.email.clone(),
        password_hash,
        full_name: payload.full_name,
    };

    match state.database.create_user(create_request).await {
        Ok(user) => {
            tracing::info!("User registered successfully: {}", user.id);
            
            // Generate tokens
            let (access_token, refresh_token) = match generate_tokens(&user, &state.config.jwt_secret) {
                Ok(tokens) => tokens,
                Err(e) => {
                    tracing::error!("Token generation failed: {}", e);
                    let response = ApiResponse {
                        success: false,
                        data: None::<()>,
                        error: Some(json!({
                            "message": "Internal server error",
                            "code": "TOKEN_GENERATION_ERROR"
                        })),
                        meta: None,
                    };
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(response));
                }
            };

            // Create session
            if let Err(e) = state.database.create_session(&user.id, &refresh_token).await {
                tracing::error!("Session creation failed: {}", e);
            }

            let auth_response = AuthResponse {
                access_token,
                refresh_token,
                token_type: "Bearer".to_string(),
                expires_in: 3600, // 1 hour
                user: UserInfo {
                    id: user.id,
                    username: user.username,
                    email: user.email,
                    full_name: user.full_name,
                    created_at: user.created_at,
                    updated_at: user.updated_at,
                },
            };

            let response = ApiResponse {
                success: true,
                data: Some(auth_response),
                error: None,
                meta: Some(json!({
                    "timestamp": chrono::Utc::now().timestamp(),
                    "request_id": Uuid::new_v4().to_string()
                })),
            };

            (StatusCode::CREATED, Json(response))
        }
        Err(e) => {
            tracing::error!("User creation failed: {}", e);
            let response = ApiResponse {
                success: false,
                data: None::<()>,
                error: Some(json!({
                    "message": "Failed to create user",
                    "code": "USER_CREATION_ERROR"
                })),
                meta: None,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        }
    }
}

/// Login user
/// POST /auth/login
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    tracing::info!("Login attempt for: {}", payload.username_or_email);

    // Get user by username or email
    let user = match get_user_by_username_or_email(&state, &payload.username_or_email).await {
        Ok(user) => user,
        Err(_) => {
            let response = ApiResponse {
                success: false,
                data: None::<()>,
                error: Some(json!({
                    "message": "Invalid credentials",
                    "code": "INVALID_CREDENTIALS"
                })),
                meta: None,
            };
            return (StatusCode::UNAUTHORIZED, Json(response));
        }
    };

    // Verify password
    match verify(&payload.password, &user.password_hash) {
        Ok(true) => {
            tracing::info!("Login successful for user: {}", user.id);
            
            // Generate tokens
            let (access_token, refresh_token) = match generate_tokens(&user, &state.config.jwt_secret) {
                Ok(tokens) => tokens,
                Err(e) => {
                    tracing::error!("Token generation failed: {}", e);
                    let response = ApiResponse {
                        success: false,
                        data: None::<()>,
                        error: Some(json!({
                            "message": "Internal server error",
                            "code": "TOKEN_GENERATION_ERROR"
                        })),
                        meta: None,
                    };
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(response));
                }
            };

            // Create session
            if let Err(e) = state.database.create_session(&user.id, &refresh_token).await {
                tracing::error!("Session creation failed: {}", e);
            }

            let auth_response = AuthResponse {
                access_token,
                refresh_token,
                token_type: "Bearer".to_string(),
                expires_in: 3600, // 1 hour
                user: UserInfo {
                    id: user.id,
                    username: user.username,
                    email: user.email,
                    full_name: user.full_name,
                    created_at: user.created_at,
                    updated_at: user.updated_at,
                },
            };

            let response = ApiResponse {
                success: true,
                data: Some(auth_response),
                error: None,
                meta: Some(json!({
                    "timestamp": chrono::Utc::now().timestamp(),
                    "request_id": Uuid::new_v4().to_string()
                })),
            };

            (StatusCode::OK, Json(response))
        }
        Ok(false) | Err(_) => {
            tracing::warn!("Invalid password for user: {}", payload.username_or_email);
            let response = ApiResponse {
                success: false,
                data: None::<()>,
                error: Some(json!({
                    "message": "Invalid credentials",
                    "code": "INVALID_CREDENTIALS"
                })),
                meta: None,
            };
            (StatusCode::UNAUTHORIZED, Json(response))
        }
    }
}

/// Refresh access token
/// POST /auth/refresh
pub async fn refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> impl IntoResponse {
    tracing::info!("Token refresh attempt");

    // Validate refresh token
    let claims = match decode::<RefreshTokenClaims>(
        &payload.refresh_token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_ref()),
        &Validation::default(),
    ) {
        Ok(token_data) => token_data.claims,
        Err(e) => {
            tracing::warn!("Invalid refresh token: {}", e);
            let response = ApiResponse {
                success: false,
                data: None::<()>,
                error: Some(json!({
                    "message": "Invalid refresh token",
                    "code": "INVALID_TOKEN"
                })),
                meta: None,
            };
            return (StatusCode::UNAUTHORIZED, Json(response));
        }
    };

    // Check if session is valid
    match state.database.is_session_valid(&claims.user_id, &payload.refresh_token).await {
        Ok(true) => {
            // Get user
            match state.database.get_user_by_id(&claims.user_id).await {
                Ok(user) => {
                    // Generate new access token
                    let access_token = match generate_access_token(&user, &state.config.jwt_secret) {
                        Ok(token) => token,
                        Err(e) => {
                            tracing::error!("Access token generation failed: {}", e);
                            let response = ApiResponse {
                                success: false,
                                data: None::<()>,
                                error: Some(json!({
                                    "message": "Token generation failed",
                                    "code": "TOKEN_GENERATION_ERROR"
                                })),
                                meta: None,
                            };
                            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response));
                        }
                    };

                    let refresh_response = RefreshTokenResponse {
                        access_token,
                        token_type: "Bearer".to_string(),
                        expires_in: 3600, // 1 hour
                    };

                    let response = ApiResponse {
                        success: true,
                        data: Some(refresh_response),
                        error: None,
                        meta: Some(json!({
                            "timestamp": chrono::Utc::now().timestamp(),
                            "request_id": Uuid::new_v4().to_string()
                        })),
                    };

                    (StatusCode::OK, Json(response))
                }
                Err(e) => {
                    tracing::error!("User not found during token refresh: {}", e);
                    let response = ApiResponse {
                        success: false,
                        data: None::<()>,
                        error: Some(json!({
                            "message": "User not found",
                            "code": "USER_NOT_FOUND"
                        })),
                        meta: None,
                    };
                    (StatusCode::UNAUTHORIZED, Json(response))
                }
            }
        }
        Ok(false) => {
            tracing::warn!("Invalid session for token refresh");
            let response = ApiResponse {
                success: false,
                data: None::<()>,
                error: Some(json!({
                    "message": "Invalid session",
                    "code": "INVALID_SESSION"
                })),
                meta: None,
            };
            (StatusCode::UNAUTHORIZED, Json(response))
        }
        Err(e) => {
            tracing::error!("Session validation failed: {}", e);
            let response = ApiResponse {
                success: false,
                data: None::<()>,
                error: Some(json!({
                    "message": "Session validation failed",
                    "code": "SESSION_ERROR"
                })),
                meta: None,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        }
    }
}

/// Logout user
/// POST /auth/logout
pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<LogoutRequest>,
) -> impl IntoResponse {
    tracing::info!("Logout attempt");

    // Extract user from token
    let user_id = match extract_user_from_headers(&headers, &state.config.jwt_secret) {
        Ok(user_id) => user_id,
        Err(_) => {
            let response = ApiResponse {
                success: false,
                data: None::<()>,
                error: Some(json!({
                    "message": "Invalid or missing token",
                    "code": "INVALID_TOKEN"
                })),
                meta: None,
            };
            return (StatusCode::UNAUTHORIZED, Json(response));
        }
    };

    // Invalidate session
    match state.database.invalidate_session(&user_id, &payload.refresh_token).await {
        Ok(_) => {
            tracing::info!("User logged out successfully: {}", user_id);
            
            let logout_response = LogoutResponse {
                message: "Logged out successfully".to_string(),
            };

            let response = ApiResponse {
                success: true,
                data: Some(logout_response),
                error: None,
                meta: Some(json!({
                    "timestamp": chrono::Utc::now().timestamp(),
                    "request_id": Uuid::new_v4().to_string()
                })),
            };

            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            tracing::error!("Session invalidation failed: {}", e);
            let response = ApiResponse {
                success: false,
                data: None::<()>,
                error: Some(json!({
                    "message": "Logout failed",
                    "code": "LOGOUT_ERROR"
                })),
                meta: None,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        }
    }
}

/// Validate token
/// GET /auth/validate
pub async fn validate_token(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    match extract_user_from_headers(&headers, &state.config.jwt_secret) {
        Ok(user_id) => {
            match state.database.get_user_by_id(&user_id).await {
                Ok(user) => {
                    let validation_response = TokenValidationResponse {
                        valid: true,
                        user: Some(UserInfo {
                            id: user.id,
                            username: user.username,
                            email: user.email,
                            full_name: user.full_name,
                            created_at: user.created_at,
                            updated_at: user.updated_at,
                        }),
                    };

                    let response = ApiResponse {
                        success: true,
                        data: Some(validation_response),
                        error: None,
                        meta: None,
                    };

                    (StatusCode::OK, Json(response))
                }
                Err(_) => {
                    let validation_response = TokenValidationResponse {
                        valid: false,
                        user: None,
                    };

                    let response = ApiResponse {
                        success: true,
                        data: Some(validation_response),
                        error: None,
                        meta: None,
                    };

                    (StatusCode::OK, Json(response))
                }
            }
        }
        Err(_) => {
            let validation_response = TokenValidationResponse {
                valid: false,
                user: None,
            };

            let response = ApiResponse {
                success: true,
                data: Some(validation_response),
                error: None,
                meta: None,
            };

            (StatusCode::OK, Json(response))
        }
    }
}

// Helper functions

fn validate_registration_input(payload: &RegisterRequest) -> Result<(), String> {
    if payload.username.len() < 3 {
        return Err("Username must be at least 3 characters long".to_string());
    }

    if payload.password.len() < 8 {
        return Err("Password must be at least 8 characters long".to_string());
    }

    if !payload.email.contains('@') {
        return Err("Invalid email format".to_string());
    }

    Ok(())
}

async fn get_user_by_username_or_email(
    state: &AppState,
    username_or_email: &str,
) -> Result<User, Box<dyn std::error::Error + Send + Sync>> {
    if username_or_email.contains('@') {
        state.database.get_user_by_email(username_or_email).await
    } else {
        state.database.get_user_by_username(username_or_email).await
    }
}

fn generate_tokens(
    user: &User,
    jwt_secret: &str,
) -> Result<(String, String), jsonwebtoken::errors::Error> {
    let access_token = generate_access_token(user, jwt_secret)?;
    let refresh_token = generate_refresh_token(user, jwt_secret)?;
    Ok((access_token, refresh_token))
}

fn generate_access_token(
    user: &User,
    jwt_secret: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = chrono::Utc::now();
    let claims = TokenClaims {
        sub: user.id.clone(),
        username: user.username.clone(),
        email: user.email.clone(),
        exp: (now + chrono::Duration::hours(1)).timestamp() as usize,
        iat: now.timestamp() as usize,
        token_type: "access".to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    )
}

fn generate_refresh_token(
    user: &User,
    jwt_secret: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = chrono::Utc::now();
    let claims = RefreshTokenClaims {
        user_id: user.id.clone(),
        exp: (now + chrono::Duration::days(30)).timestamp() as usize,
        iat: now.timestamp() as usize,
        token_type: "refresh".to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    )
}

fn extract_user_from_headers(
    headers: &HeaderMap,
    jwt_secret: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let auth_header = headers
        .get("authorization")
        .ok_or("Missing authorization header")?;
    
    let auth_str = auth_header.to_str()?;
    let token = auth_str
        .strip_prefix("Bearer ")
        .ok_or("Invalid authorization header format")?;

    let claims = decode::<TokenClaims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default(),
    )?;

    Ok(claims.claims.sub)
}

/// Change user password
/// POST /auth/change-password
pub async fn change_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ChangePasswordRequest>,
) -> impl IntoResponse {
    tracing::info!("Password change attempt");

    // Extract user from JWT token
    let user_id = match extract_user_from_headers(&headers, &state.config.jwt_secret) {
        Ok(id) => id,
        Err(_) => {
            let response = ApiResponse {
                success: false,
                data: None::<()>,
                error: Some(json!({
                    "message": "Invalid or missing authorization token",
                    "code": "UNAUTHORIZED"
                })),
                meta: None,
            };
            return (StatusCode::UNAUTHORIZED, Json(response));
        }
    };

    // Get user from database
    let user = match state.database.get_user_by_id(&user_id).await {
        Ok(user) => user,
        Err(_) => {
            let response = ApiResponse {
                success: false,
                data: None::<()>,
                error: Some(json!({
                    "message": "User not found",
                    "code": "USER_NOT_FOUND"
                })),
                meta: None,
            };
            return (StatusCode::NOT_FOUND, Json(response));
        }
    };

    // Verify current password
    if !verify(&payload.current_password, &user.password_hash).unwrap_or(false) {
        let response = ApiResponse {
            success: false,
            data: None::<()>,
            error: Some(json!({
                "message": "Current password is incorrect",
                "code": "INVALID_PASSWORD"
            })),
            meta: None,
        };
        return (StatusCode::BAD_REQUEST, Json(response));
    }

    // Validate new password
    if payload.new_password.len() < 8 {
        let response = ApiResponse {
            success: false,
            data: None::<()>,
            error: Some(json!({
                "message": "New password must be at least 8 characters long",
                "code": "VALIDATION_ERROR"
            })),
            meta: None,
        };
        return (StatusCode::BAD_REQUEST, Json(response));
    }

    // Hash new password
    let new_password_hash = match hash(&payload.new_password, DEFAULT_COST) {
        Ok(hash) => hash,
        Err(_) => {
            let response = ApiResponse {
                success: false,
                data: None::<()>,
                error: Some(json!({
                    "message": "Failed to process new password",
                    "code": "INTERNAL_ERROR"
                })),
                meta: None,
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response));
        }
    };

    // Update password in database
    if let Err(_) = state.database.update_user_password(&user_id, &new_password_hash).await {
        let response = ApiResponse {
            success: false,
            data: None::<()>,
            error: Some(json!({
                "message": "Failed to update password",
                "code": "DATABASE_ERROR"
            })),
            meta: None,
        };
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(response));
    }

    tracing::info!("Password changed successfully for user: {}", user_id);

    let response = ApiResponse {
        success: true,
        data: Some(json!({
            "message": "Password changed successfully"
        })),
        error: None,
        meta: None,
    };

    (StatusCode::OK, Json(response))
}