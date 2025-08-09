//! Movie-related API handlers
//!
//! This module contains handlers for movie-related endpoints including:
//! - Movie search
//! - Popular movies
//! - Trending movies
//! - Movie details
//! - Movie discovery

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

use crate::{
    error::{AppError, AppResult},
    middleware::auth::OptionalUserContext,
    models::{
        api::{PaginatedResponse, SearchQuery, DiscoverQuery},
        tmdb::{Movie, MovieDetails, TrendingResponse},
    },
    AppState,
};

/// Search movies endpoint
/// GET /api/v1/movies/search?query=term&page=1
pub async fn search_movies(
    Query(params): Query<SearchQuery>,
    State(state): State<AppState>,
    OptionalUserContext(user_context): OptionalUserContext,
) -> AppResult<Json<PaginatedResponse<Movie>>> {
    let query = if params.query.is_empty() {
        return Err(AppError::BadRequest("Search query is required".to_string()));
    } else {
        params.query
    };
    
    let page = params.page.unwrap_or(1);
    
    info!(
        "Searching movies: query='{}', page={}, user_id={:?}",
        query, page, user_context.as_ref().map(|u| u.user_id)
    );
    
    let result = state.services.tmdb
        .search_movies(&query, page)
        .await
        .map_err(|e| {
            error!("Failed to search movies: {}", e);
            AppError::ExternalApi(ExternalApiError::TmdbError("Failed to search movies".to_string()))
        })?;
    
    Ok(Json(result))
}

/// Get movie details endpoint
/// GET /api/v1/movies/:id
pub async fn get_movie(
    Path(movie_id): Path<u32>,
    State(state): State<AppState>,
    OptionalUserContext(user_context): OptionalUserContext,
) -> AppResult<Json<MovieDetails>> {
    info!(
        "Getting movie details: id={}, user_id={:?}",
        movie_id, user_context.as_ref().map(|u| u.user_id)
    );
    
    let movie = state.services.tmdb
        .get_movie_details(movie_id)
        .await
        .map_err(|e| {
            error!("Failed to get movie details for id {}: {}", movie_id, e);
            match e {
                crate::services::tmdb::TmdbError::NotFound(_) => AppError::NotFound(format!("Movie with id {} not found", movie_id)),
                _ => AppError::ExternalApi(ExternalApiError::TmdbError("Failed to get movie details".to_string())),
            }
        })?;
    
    Ok(Json(movie))
}

/// Get trending movies endpoint
/// GET /api/v1/movies/trending?time_window=day&page=1
pub async fn get_trending_movies(
    Query(params): Query<TrendingQuery>,
    State(state): State<AppState>,
    OptionalUserContext(user_context): OptionalUserContext,
) -> AppResult<Json<PaginatedResponse<Movie>>> {
    let time_window = params.time_window.unwrap_or_else(|| "day".to_string());
    let page = params.page.unwrap_or(1);
    
    info!(
        "Getting trending movies: time_window='{}', page={}, user_id={:?}",
        time_window, page, user_context.as_ref().map(|u| u.user_id)
    );
    
    let result = state.services.tmdb
        .get_trending_movies(&time_window, page)
        .await
        .map_err(|e| {
            error!("Failed to get trending movies: {}", e);
            AppError::ExternalApi(ExternalApiError::TmdbError("Failed to get trending movies".to_string()))
        })?;
    
    Ok(Json(result))
}

/// Get popular movies endpoint
/// GET /api/v1/movies/popular?page=1&region=US
pub async fn get_popular_movies(
    Query(params): Query<PopularQuery>,
    State(state): State<AppState>,
    OptionalUserContext(user_context): OptionalUserContext,
) -> AppResult<Json<PaginatedResponse<Movie>>> {
    let page = params.page.unwrap_or(1);
    let region = params.region;
    
    info!(
        "Getting popular movies: page={}, region={:?}, user_id={:?}",
        page, region, user_context.as_ref().map(|u| u.user_id)
    );
    
    let result = state.services.tmdb
        .get_popular_movies(page, region.as_deref())
        .await
        .map_err(|e| {
            error!("Failed to get popular movies: {}", e);
            AppError::ExternalApi(ExternalApiError::TmdbError("Failed to get popular movies".to_string()))
        })?;
    
    Ok(Json(result))
}

/// Discover movies endpoint
/// GET /api/v1/movies/discover?page=1&sort_by=popularity.desc&with_genres=28
pub async fn discover_movies(
    Query(params): Query<DiscoverQuery>,
    State(state): State<AppState>,
    OptionalUserContext(user_context): OptionalUserContext,
) -> AppResult<Json<PaginatedResponse<Movie>>> {
    let page = params.page.unwrap_or(1);
    
    info!(
        "Discovering movies: page={}, sort_by={:?}, user_id={:?}",
        page, params.sort_by, user_context.as_ref().map(|u| u.user_id)
    );
    
    let result = state.services.tmdb
        .discover_movies(&params, page)
        .await
        .map_err(|e| {
            error!("Failed to discover movies: {}", e);
            AppError::ExternalApi(ExternalApiError::TmdbError("Failed to discover movies".to_string()))
        })?;
    
    Ok(Json(result))
}

/// Query parameters for trending endpoint
#[derive(Debug, Deserialize)]
pub struct TrendingQuery {
    pub time_window: Option<String>,
    pub page: Option<u32>,
}

/// Query parameters for popular endpoint
#[derive(Debug, Deserialize)]
pub struct PopularQuery {
    pub page: Option<u32>,
    pub region: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;
    
    // Helper function to create test state would go here
    // This would require setting up mock services
    
    #[tokio::test]
    async fn test_search_movies_missing_query() {
        // Test that search without query returns bad request
        // Implementation would require mock setup
    }
    
    #[tokio::test]
    async fn test_get_movie_invalid_id() {
        // Test that invalid movie ID returns not found
        // Implementation would require mock setup
    }
    
    #[tokio::test]
    async fn test_trending_movies_default_params() {
        // Test trending movies with default parameters
        // Implementation would require mock setup
    }
}