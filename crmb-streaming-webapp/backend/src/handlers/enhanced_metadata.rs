//! Enhanced metadata API handlers
//!
//! Provides endpoints for:
//! - Enhanced search combining TMDB and MDBList data
//! - User-specific MDBList integration
//! - High-quality metadata with fallback mechanisms

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    error::AppResult,
    models::api::{ApiResponse, PaginatedResponse},
    services::enhanced_metadata::{EnhancedMetadataService, EnhancedMediaItem, MediaType},
    AppState,
};

/// Query parameters for enhanced search
#[derive(Debug, Deserialize)]
pub struct EnhancedSearchQuery {
    pub query: String,
    pub year: Option<u32>,
    #[serde(rename = "type")]
    pub media_type: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

/// Query parameters for MDBList user lists
#[derive(Debug, Deserialize)]
pub struct UserListsQuery {
    pub user_id: Option<String>,
    pub include_private: Option<bool>,
}

/// Query parameters for MDBList trending
#[derive(Debug, Deserialize)]
pub struct TrendingQuery {
    #[serde(rename = "type")]
    pub media_type: Option<String>,
    pub period: Option<String>,
}

/// Enhanced search endpoint
pub async fn enhanced_search(
    State(state): State<AppState>,
    Query(params): Query<EnhancedSearchQuery>,
) -> AppResult<Json<ApiResponse<PaginatedResponse<EnhancedMediaItem>>>> {
    let enhanced_service = EnhancedMetadataService::new(
        state.services.tmdb.clone(),
        state.services.mdblist.clone(),
        state.services.cache.clone(),
    );

    let media_type = match params.media_type.as_deref() {
        Some("movie") => Some(MediaType::Movie),
        Some("tv") => Some(MediaType::Tv),
        _ => None,
    };

    let results = enhanced_service
        .search_enhanced(
            &params.query,
            params.year,
            media_type,
            None, // TODO: Extract user ID from auth token
        )
        .await
        .map_err(|e| crate::error::AppError::Internal(format!("Enhanced search failed: {}", e)))?;

    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(20).min(100);
    let start = ((page - 1) * limit) as usize;
    let end = (start + limit as usize).min(results.len());

    let paginated_results = results[start..end].to_vec();
    let total = results.len() as u32;
    let total_pages = (total as f32 / limit as f32).ceil() as u32;

    let response = PaginatedResponse {
        data: paginated_results,
        pagination: crate::models::api::PaginationInfo {
            page,
            limit,
            total,
            total_pages,
            has_next: page < total_pages,
            has_prev: page > 1,
        },
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Get enhanced media item by TMDB ID
pub async fn get_enhanced_by_tmdb_id(
    State(state): State<AppState>,
    axum::extract::Path((media_type, tmdb_id)): axum::extract::Path<(String, u64)>,
) -> AppResult<Json<ApiResponse<EnhancedMediaItem>>> {
    let enhanced_service = EnhancedMetadataService::new(
        state.services.tmdb.clone(),
        state.services.mdblist.clone(),
        state.services.cache.clone(),
    );

    let media_type = match media_type.as_str() {
        "movie" => MediaType::Movie,
        "tv" => MediaType::Tv,
        _ => return Err(crate::error::AppError::BadRequest("Invalid media type".to_string())),
    };

    let item = enhanced_service
        .get_by_tmdb_id(tmdb_id, media_type, None)
        .await
        .map_err(|e| crate::error::AppError::Internal(format!("Failed to get enhanced item: {}", e)))?;

    match item {
        Some(item) => Ok(Json(ApiResponse::success(item))),
        None => Err(crate::error::AppError::NotFound("Media item not found".to_string())),
    }
}

/// Get user's MDBList watchlists
pub async fn get_user_watchlists(
    State(state): State<AppState>,
    Query(params): Query<UserListsQuery>,
) -> AppResult<Json<ApiResponse<Vec<crate::models::mdblist::MdbListUserList>>>> {
    let user_id = params.user_id
        .ok_or_else(|| crate::error::AppError::BadRequest("User ID required".to_string()))?;

    let lists = state.services.mdblist
        .get_user_lists(&user_id)
        .await
        .map_err(|e| crate::error::AppError::Internal(format!("Failed to get user lists: {}", e)))?;

    Ok(Json(ApiResponse::success(lists.lists)))
}

/// Get specific MDBList content
pub async fn get_mdblist_list(
    State(state): State<AppState>,
    axum::extract::Path(list_id): axum::extract::Path<String>,
) -> AppResult<Json<ApiResponse<crate::models::mdblist::MdbListList>>> {
    let list = state.services.mdblist
        .get_list_content(&list_id)
        .await
        .map_err(|e| crate::error::AppError::Internal(format!("Failed to get list: {}", e)))?;

    Ok(Json(ApiResponse::success(list)))
}

/// Get MDBList trending content
pub async fn get_mdblist_trending(
    State(state): State<AppState>,
    Query(params): Query<TrendingQuery>,
) -> AppResult<Json<ApiResponse<Vec<EnhancedMediaItem>>>> {
    let enhanced_service = EnhancedMetadataService::new(
        state.services.tmdb.clone(),
        state.services.mdblist.clone(),
        state.services.cache.clone(),
    );

    let media_type = match params.media_type.as_deref() {
        Some("movie") => Some(crate::services::mdblist::MdbListMediaType::Movie),
        Some("tv") => Some(crate::services::mdblist::MdbListMediaType::Show),
        _ => None,
    };

    let trending = state.services.mdblist
        .get_trending(media_type)
        .await
        .map_err(|e| crate::error::AppError::Internal(format!("Failed to get trending: {}", e)))?;

    // Convert MDBList trending to enhanced format
    let mut enhanced_items = Vec::new();
    for item in trending.trending {
        let enhanced = enhanced_service
            .get_by_tmdb_id(
                item.tmdb_id.unwrap_or(0),
                match item.media_type.as_str() {
                    "movie" => MediaType::Movie,
                    "show" => MediaType::Tv,
                    _ => MediaType::Movie,
                },
                None,
            )
            .await
            .unwrap_or_else(|_| None);

        if let Some(enhanced) = enhanced {
            enhanced_items.push(enhanced);
        }
    }

    Ok(Json(ApiResponse::success(enhanced_items)))
}

/// Health check for MDBList service
pub async fn mdblist_health(
    State(state): State<AppState>,
) -> AppResult<Json<ApiResponse<crate::services::ServiceHealth>>> {
    let health = state.services.mdblist.health_check().await;
    Ok(Json(ApiResponse::success(health)))
}

/// Health check for enhanced metadata service
pub async fn enhanced_metadata_health(
    State(state): State<AppState>,
) -> AppResult<Json<ApiResponse<Vec<crate::services::ServiceHealth>>>> {
    let mut health_checks = Vec::new();
    
    // Check TMDB service
    health_checks.push(state.services.tmdb.health_check().await);
    
    // Check MDBList service
    health_checks.push(state.services.mdblist.health_check().await);
    
    Ok(Json(ApiResponse::success(health_checks)))
}

/// API documentation response
#[derive(Debug, Serialize)]
pub struct ApiDocsResponse {
    pub endpoints: Vec<EndpointDoc>,
    pub features: Vec<String>,
    pub rate_limits: RateLimitInfo,
}

#[derive(Debug, Serialize)]
pub struct EndpointDoc {
    pub path: String,
    pub method: String,
    pub description: String,
    pub parameters: Vec<ParameterDoc>,
    pub response_example: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ParameterDoc {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub example: String,
}

#[derive(Debug, Serialize)]
pub struct RateLimitInfo {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub burst_limit: u32,
}

/// Get API documentation
pub async fn get_api_docs() -> Json<ApiDocsResponse> {
    Json(ApiDocsResponse {
        endpoints: vec![
            EndpointDoc {
                path: "/api/v1/enhanced/search".to_string(),
                method: "GET".to_string(),
                description: "Enhanced search combining TMDB and MDBList data".to_string(),
                parameters: vec![
                    ParameterDoc {
                        name: "query".to_string(),
                        description: "Search query".to_string(),
                        required: true,
                        example: "The Shawshank Redemption".to_string(),
                    },
                    ParameterDoc {
                        name: "year".to_string(),
                        description: "Release year filter".to_string(),
                        required: false,
                        example: "1994".to_string(),
                    },
                    ParameterDoc {
                        name: "type".to_string(),
                        description: "Media type filter".to_string(),
                        required: false,
                        example: "movie|tv".to_string(),
                    },
                    ParameterDoc {
                        name: "page".to_string(),
                        description: "Page number".to_string(),
                        required: false,
                        example: "1".to_string(),
                    },
                    ParameterDoc {
                        name: "limit".to_string(),
                        description: "Results per page (max 100)".to_string(),
                        required: false,
                        example: "20".to_string(),
                    },
                ],
                response_example: serde_json::json!({
                    "success": true,
                    "data": {
                        "data": [
                            {
                                "tmdb_id": 278,
                                "imdb_id": "tt0111161",
                                "media_type": "movie",
                                "title": "The Shawshank Redemption",
                                "overview": "Framed in the 1940s...",
                                "posters": {
                                    "w500": "https://image.tmdb.org/t/p/w500/...",
                                    "w780": "https://image.tmdb.org/t/p/w780/...",
                                    "original": "https://image.tmdb.org/t/p/original/..."
                                },
                                "ratings": {
                                    "tmdb": 8.7,
                                    "imdb": 9.3
                                }
                            }
                        ],
                        "pagination": {
                            "page": 1,
                            "limit": 20,
                            "total": 1,
                            "total_pages": 1,
                            "has_next": false,
                            "has_prev": false
                        }
                    }
                }),
            },
        ],
        features: vec![
            "Dual API integration (TMDB + MDBList)".to_string(),
            "High-quality images (multiple sizes)".to_string(),
            "User-specific MDBList integration".to_string(),
            "Advanced caching with fallback".to_string(),
            "Real-time search across APIs".to_string(),
            "Comprehensive error handling".to_string(),
        ],
        rate_limits: RateLimitInfo {
            requests_per_minute: 100,
            requests_per_hour: 1000,
            burst_limit: 10,
        },
    })
}