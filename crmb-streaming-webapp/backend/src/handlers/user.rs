use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    Extension,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::{
    AppState,
    database::Database,
    models::{
        auth::{AuthError, TokenClaims},
        user::*,
        ApiResponse, ErrorResponse,
    },
};

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

/// Get user profile
/// GET /api/user/profile
pub async fn get_profile(
    State(state): State<AppState>,
    Extension(claims): Extension<TokenClaims>,
) -> impl IntoResponse {
    tracing::info!("Getting profile for user: {}", claims.sub);

    match state.database.get_user_by_id(claims.sub).await {
        Ok(Some(user)) => {
            let profile = UserProfile::from(user);
            let response = ApiResponse {
                success: true,
                data: Some(profile),
                error: None,
                meta: Some(json!({
                    "timestamp": chrono::Utc::now().timestamp(),
                    "request_id": Uuid::new_v4().to_string()
                })),
            };
            (StatusCode::OK, Json(response))
        }
        Ok(None) => {
            tracing::warn!("User not found: {}", claims.sub);
            let error_response = ErrorResponse {
                message: "User not found".to_string(),
                code: "USER_NOT_FOUND".to_string(),
                details: None,
            };
            let response = ApiResponse {
                success: false,
                data: None::<()>,
                error: Some(error_response),
                meta: Some(json!({
                    "timestamp": chrono::Utc::now().timestamp(),
                    "request_id": Uuid::new_v4().to_string()
                })),
            };
            (StatusCode::NOT_FOUND, Json(response))
        }
        Err(e) => {
            tracing::error!("Database error getting user profile: {}", e);
            handle_user_error(AuthError::DatabaseError(e.to_string()))
        }
    }
}

/// Update user profile
/// PUT /api/user/profile
pub async fn update_profile(
    State(state): State<AppState>,
    Extension(claims): Extension<TokenClaims>,
    Json(update_request): Json<UpdateUserRequest>,
) -> impl IntoResponse {
    tracing::info!("Updating profile for user: {}", claims.sub);

    // Validate the update request
    if let Err(validation_error) = validate_update_request(&update_request) {
        return handle_user_error(AuthError::ValidationError(validation_error));
    }

    // Get current user
    let current_user = match state.database.get_user_by_id(claims.sub).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return handle_user_error(AuthError::UserNotFound);
        }
        Err(e) => {
            return handle_user_error(AuthError::DatabaseError(e.to_string()));
        }
    };

    // Check if email is being changed and if it's already taken
    if let Some(ref new_email) = update_request.email {
        if new_email != &current_user.email {
            match state.database.get_user_by_email(new_email).await {
                Ok(Some(_)) => {
                    return handle_user_error(AuthError::EmailAlreadyExists);
                }
                Ok(None) => {}, // Email is available
                Err(e) => {
                    return handle_user_error(AuthError::DatabaseError(e.to_string()));
                }
            }
        }
    }

    // Check if username is being changed and if it's already taken
    if let Some(ref new_username) = update_request.username {
        if new_username != &current_user.username {
            match state.database.get_user_by_username(new_username).await {
                Ok(Some(_)) => {
                    return handle_user_error(AuthError::UsernameAlreadyExists);
                }
                Ok(None) => {}, // Username is available
                Err(e) => {
                    return handle_user_error(AuthError::DatabaseError(e.to_string()));
                }
            }
        }
    }

    // Create updated user
    let mut updated_user = current_user;
    if let Some(email) = update_request.email {
        updated_user.email = email;
    }
    if let Some(username) = update_request.username {
        updated_user.username = username;
    }
    if let Some(full_name) = update_request.full_name {
        updated_user.full_name = Some(full_name);
    }
    if let Some(avatar_url) = update_request.avatar_url {
        updated_user.avatar_url = Some(avatar_url);
    }
    if let Some(preferences) = update_request.preferences {
        updated_user.preferences = Some(serde_json::to_string(&preferences).unwrap_or_default());
    }
    updated_user.updated_at = chrono::Utc::now();

    // TODO: Implement user update in database
    // For now, we'll return the updated profile
    let profile = UserProfile::from(updated_user);
    let response = ApiResponse {
        success: true,
        data: Some(profile),
        error: None,
        meta: Some(json!({
            "timestamp": chrono::Utc::now().timestamp(),
            "request_id": Uuid::new_v4().to_string(),
            "note": "User update not yet implemented in database layer"
        })),
    };

    (StatusCode::OK, Json(response))
}

/// Get user watchlist
/// GET /api/user/watchlist
pub async fn get_watchlist(
    State(state): State<AppState>,
    Extension(claims): Extension<TokenClaims>,
    Query(pagination): Query<PaginationQuery>,
) -> impl IntoResponse {
    tracing::info!("Getting watchlist for user: {}", claims.sub);

    let page = pagination.page.unwrap_or(1);
    let limit = pagination.limit.unwrap_or(20).min(100); // Max 100 items per page

    match state.database.get_user_watchlist(claims.sub).await {
        Ok(watchlist_items) => {
            // Simple pagination (in a real app, you'd do this in the database)
            let total_items = watchlist_items.len();
            let start_index = ((page - 1) * limit) as usize;
            let end_index = (start_index + limit as usize).min(total_items);
            
            let paginated_items = if start_index < total_items {
                watchlist_items[start_index..end_index].to_vec()
            } else {
                vec![]
            };

            let response = ApiResponse {
                success: true,
                data: Some(paginated_items),
                error: None,
                meta: Some(json!({
                    "timestamp": chrono::Utc::now().timestamp(),
                    "request_id": Uuid::new_v4().to_string(),
                    "pagination": {
                        "page": page,
                        "limit": limit,
                        "total_items": total_items,
                        "total_pages": (total_items as f64 / limit as f64).ceil() as u32,
                        "has_next": end_index < total_items,
                        "has_prev": page > 1
                    }
                })),
            };
            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            tracing::error!("Database error getting watchlist: {}", e);
            handle_user_error(AuthError::DatabaseError(e.to_string()))
        }
    }
}

/// Add item to watchlist
/// POST /api/user/watchlist
pub async fn add_to_watchlist(
    State(state): State<AppState>,
    Extension(claims): Extension<TokenClaims>,
    Json(add_request): Json<AddToWatchlistRequest>,
) -> impl IntoResponse {
    tracing::info!(
        "Adding item to watchlist for user: {}, tmdb_id: {}, media_type: {}",
        claims.sub, add_request.tmdb_id, add_request.media_type
    );

    // Validate media type
    if !matches!(add_request.media_type.as_str(), "movie" | "tv") {
        return handle_user_error(AuthError::ValidationError(
            "Invalid media type. Must be 'movie' or 'tv'".to_string(),
        ));
    }

    // Check if item already exists in watchlist
    match state.database.get_user_watchlist(claims.sub).await {
        Ok(watchlist) => {
            if watchlist.iter().any(|item| {
                item.tmdb_id == add_request.tmdb_id && item.media_type == add_request.media_type
            }) {
                return handle_user_error(AuthError::ValidationError(
                    "Item already exists in watchlist".to_string(),
                ));
            }
        }
        Err(e) => {
            return handle_user_error(AuthError::DatabaseError(e.to_string()));
        }
    }

    // Add to watchlist
    match state
        .database
        .add_to_watchlist(
            claims.sub,
            add_request.tmdb_id,
            &add_request.media_type,
            add_request.title.as_deref(),
            add_request.poster_path.as_deref(),
        )
        .await
    {
        Ok(watchlist_item) => {
            let response = ApiResponse {
                success: true,
                data: Some(watchlist_item),
                error: None,
                meta: Some(json!({
                    "timestamp": chrono::Utc::now().timestamp(),
                    "request_id": Uuid::new_v4().to_string(),
                    "action": "added_to_watchlist"
                })),
            };
            (StatusCode::CREATED, Json(response))
        }
        Err(e) => {
            tracing::error!("Database error adding to watchlist: {}", e);
            handle_user_error(AuthError::DatabaseError(e.to_string()))
        }
    }
}

/// Remove item from watchlist
/// DELETE /api/user/watchlist/{tmdb_id}/{media_type}
pub async fn remove_from_watchlist(
    State(state): State<AppState>,
    Extension(claims): Extension<TokenClaims>,
    Path((tmdb_id, media_type)): Path<(u32, String)>,
) -> impl IntoResponse {
    tracing::info!(
        "Removing item from watchlist for user: {}, tmdb_id: {}, media_type: {}",
        claims.sub, tmdb_id, media_type
    );

    // Validate media type
    if !matches!(media_type.as_str(), "movie" | "tv") {
        return handle_user_error(AuthError::ValidationError(
            "Invalid media type. Must be 'movie' or 'tv'".to_string(),
        ));
    }

    match state
        .database
        .remove_from_watchlist(claims.sub, tmdb_id, &media_type)
        .await
    {
        Ok(removed) => {
            if removed {
                let response = ApiResponse {
                    success: true,
                    data: Some(json!({
                        "message": "Item removed from watchlist",
                        "tmdb_id": tmdb_id,
                        "media_type": media_type
                    })),
                    error: None,
                    meta: Some(json!({
                        "timestamp": chrono::Utc::now().timestamp(),
                        "request_id": Uuid::new_v4().to_string(),
                        "action": "removed_from_watchlist"
                    })),
                };
                (StatusCode::OK, Json(response))
            } else {
                let error_response = ErrorResponse {
                    message: "Item not found in watchlist".to_string(),
                    code: "ITEM_NOT_FOUND".to_string(),
                    details: Some(json!({
                        "tmdb_id": tmdb_id,
                        "media_type": media_type
                    })),
                };
                let response = ApiResponse {
                    success: false,
                    data: None::<()>,
                    error: Some(error_response),
                    meta: Some(json!({
                        "timestamp": chrono::Utc::now().timestamp(),
                        "request_id": Uuid::new_v4().to_string()
                    })),
                };
                (StatusCode::NOT_FOUND, Json(response))
            }
        }
        Err(e) => {
            tracing::error!("Database error removing from watchlist: {}", e);
            handle_user_error(AuthError::DatabaseError(e.to_string()))
        }
    }
}

/// Check if item is in watchlist
/// GET /api/user/watchlist/{tmdb_id}/{media_type}/check
pub async fn check_watchlist_item(
    State(state): State<AppState>,
    Extension(claims): Extension<TokenClaims>,
    Path((tmdb_id, media_type)): Path<(u32, String)>,
) -> impl IntoResponse {
    tracing::debug!(
        "Checking watchlist item for user: {}, tmdb_id: {}, media_type: {}",
        claims.sub, tmdb_id, media_type
    );

    // Validate media type
    if !matches!(media_type.as_str(), "movie" | "tv") {
        return handle_user_error(AuthError::ValidationError(
            "Invalid media type. Must be 'movie' or 'tv'".to_string(),
        ));
    }

    match state.database.get_user_watchlist(claims.sub).await {
        Ok(watchlist) => {
            let is_in_watchlist = watchlist.iter().any(|item| {
                item.tmdb_id == tmdb_id && item.media_type == media_type
            });

            let response = ApiResponse {
                success: true,
                data: Some(json!({
                    "in_watchlist": is_in_watchlist,
                    "tmdb_id": tmdb_id,
                    "media_type": media_type
                })),
                error: None,
                meta: Some(json!({
                    "timestamp": chrono::Utc::now().timestamp(),
                    "request_id": Uuid::new_v4().to_string()
                })),
            };
            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            tracing::error!("Database error checking watchlist: {}", e);
            handle_user_error(AuthError::DatabaseError(e.to_string()))
        }
    }
}

/// Get user preferences
/// GET /api/user/preferences
pub async fn get_preferences(
    State(state): State<AppState>,
    Extension(claims): Extension<TokenClaims>,
) -> impl IntoResponse {
    tracing::info!("Getting preferences for user: {}", claims.sub);

    match state.database.get_user_by_id(claims.sub).await {
        Ok(Some(user)) => {
            let preferences: UserPreferences = user
                .preferences
                .as_ref()
                .and_then(|p| serde_json::from_str(p).ok())
                .unwrap_or_default();

            let response = ApiResponse {
                success: true,
                data: Some(preferences),
                error: None,
                meta: Some(json!({
                    "timestamp": chrono::Utc::now().timestamp(),
                    "request_id": Uuid::new_v4().to_string()
                })),
            };
            (StatusCode::OK, Json(response))
        }
        Ok(None) => handle_user_error(AuthError::UserNotFound),
        Err(e) => {
            tracing::error!("Database error getting preferences: {}", e);
            handle_user_error(AuthError::DatabaseError(e.to_string()))
        }
    }
}

/// Update user preferences
/// PUT /api/user/preferences
pub async fn update_preferences(
    State(state): State<AppState>,
    Extension(claims): Extension<TokenClaims>,
    Json(preferences): Json<UserPreferences>,
) -> impl IntoResponse {
    tracing::info!("Updating preferences for user: {}", claims.sub);

    // Get current user
    let mut user = match state.database.get_user_by_id(claims.sub).await {
        Ok(Some(user)) => user,
        Ok(None) => return handle_user_error(AuthError::UserNotFound),
        Err(e) => {
            return handle_user_error(AuthError::DatabaseError(e.to_string()));
        }
    };

    // Update preferences
    user.preferences = Some(serde_json::to_string(&preferences).unwrap_or_default());
    user.updated_at = chrono::Utc::now();

    // TODO: Implement user update in database
    // For now, we'll return the updated preferences
    let response = ApiResponse {
        success: true,
        data: Some(preferences),
        error: None,
        meta: Some(json!({
            "timestamp": chrono::Utc::now().timestamp(),
            "request_id": Uuid::new_v4().to_string(),
            "note": "Preferences update not yet implemented in database layer"
        })),
    };

    (StatusCode::OK, Json(response))
}

/// Delete user account
/// DELETE /api/user/account
pub async fn delete_account(
    State(state): State<AppState>,
    Extension(claims): Extension<TokenClaims>,
    Json(delete_request): Json<DeleteAccountRequest>,
) -> impl IntoResponse {
    tracing::warn!("Account deletion requested for user: {}", claims.sub);

    // Get current user to verify password
    let user = match state.database.get_user_by_id(claims.sub).await {
        Ok(Some(user)) => user,
        Ok(None) => return handle_user_error(AuthError::UserNotFound),
        Err(e) => {
            return handle_user_error(AuthError::DatabaseError(e.to_string()));
        }
    };

    // Verify password
    if !bcrypt::verify(&delete_request.password, &user.password_hash)
        .unwrap_or(false)
    {
        return handle_user_error(AuthError::InvalidCredentials);
    }

    // TODO: Implement account deletion
    // This should:
    // 1. Delete user data
    // 2. Delete watchlist items
    // 3. Invalidate all sessions
    // 4. Log the deletion for audit purposes

    let response = ApiResponse {
        success: true,
        data: Some(json!({
            "message": "Account deletion initiated",
            "note": "Account deletion not yet fully implemented"
        })),
        error: None,
        meta: Some(json!({
            "timestamp": chrono::Utc::now().timestamp(),
            "request_id": Uuid::new_v4().to_string(),
            "user_id": claims.sub
        })),
    };

    (StatusCode::OK, Json(response))
}

// Helper functions

fn validate_update_request(request: &UpdateUserRequest) -> Result<(), String> {
    if let Some(ref email) = request.email {
        if !email.contains('@') || email.len() < 5 {
            return Err("Invalid email format".to_string());
        }
    }

    if let Some(ref username) = request.username {
        if username.len() < 3 || username.len() > 30 {
            return Err("Username must be between 3 and 30 characters".to_string());
        }
        if !username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err("Username can only contain alphanumeric characters, underscores, and hyphens".to_string());
        }
    }

    if let Some(ref full_name) = request.full_name {
        if full_name.len() > 100 {
            return Err("Full name cannot exceed 100 characters".to_string());
        }
    }

    if let Some(ref avatar_url) = request.avatar_url {
        if !avatar_url.starts_with("http://") && !avatar_url.starts_with("https://") {
            return Err("Avatar URL must be a valid HTTP/HTTPS URL".to_string());
        }
    }

    Ok(())
}

fn handle_user_error(error: AuthError) -> (StatusCode, Json<ApiResponse<()>>) {
    let (status_code, error_code, message) = match error {
        AuthError::ValidationError(msg) => (
            StatusCode::BAD_REQUEST,
            "VALIDATION_ERROR",
            msg,
        ),
        AuthError::UserNotFound => (
            StatusCode::NOT_FOUND,
            "USER_NOT_FOUND",
            "User not found".to_string(),
        ),
        AuthError::EmailAlreadyExists => (
            StatusCode::CONFLICT,
            "EMAIL_ALREADY_EXISTS",
            "Email address is already registered".to_string(),
        ),
        AuthError::UsernameAlreadyExists => (
            StatusCode::CONFLICT,
            "USERNAME_ALREADY_EXISTS",
            "Username is already taken".to_string(),
        ),
        AuthError::InvalidCredentials => (
            StatusCode::UNAUTHORIZED,
            "INVALID_CREDENTIALS",
            "Invalid credentials".to_string(),
        ),
        AuthError::DatabaseError(msg) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "DATABASE_ERROR",
            msg,
        ),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL_ERROR",
            "An internal error occurred".to_string(),
        ),
    };

    tracing::error!("User operation error: {} - {}", error_code, message);

    let error_response = ErrorResponse {
        message,
        code: error_code.to_string(),
        details: None,
    };

    let response = ApiResponse {
        success: false,
        data: None,
        error: Some(error_response),
        meta: Some(json!({
            "timestamp": chrono::Utc::now().timestamp(),
            "request_id": Uuid::new_v4().to_string()
        })),
    };

    (status_code, Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_update_request() {
        // Valid request
        let valid_request = UpdateUserRequest {
            email: Some("test@example.com".to_string()),
            username: Some("testuser".to_string()),
            full_name: Some("Test User".to_string()),
            avatar_url: Some("https://example.com/avatar.jpg".to_string()),
            preferences: None,
        };
        assert!(validate_update_request(&valid_request).is_ok());

        // Invalid email
        let invalid_email = UpdateUserRequest {
            email: Some("invalid-email".to_string()),
            username: None,
            full_name: None,
            avatar_url: None,
            preferences: None,
        };
        assert!(validate_update_request(&invalid_email).is_err());

        // Invalid username (too short)
        let invalid_username = UpdateUserRequest {
            email: None,
            username: Some("ab".to_string()),
            full_name: None,
            avatar_url: None,
            preferences: None,
        };
        assert!(validate_update_request(&invalid_username).is_err());

        // Invalid avatar URL
        let invalid_avatar = UpdateUserRequest {
            email: None,
            username: None,
            full_name: None,
            avatar_url: Some("not-a-url".to_string()),
            preferences: None,
        };
        assert!(validate_update_request(&invalid_avatar).is_err());
    }
}