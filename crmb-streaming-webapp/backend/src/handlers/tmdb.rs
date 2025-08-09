use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    AppState,
    models::{
        tmdb::*,
        ApiResponse, PaginatedResponse,
    },
};

/// Get TMDB configuration
/// GET /api/tmdb/configuration
pub async fn get_configuration(State(state): State<AppState>) -> impl IntoResponse {
    tracing::info!("Fetching TMDB configuration");

    let cache_key = "tmdb:configuration";
    
    // Try to get from cache first
    if let Some(cached_config) = get_from_cache(&state, cache_key).await {
        tracing::debug!("Returning cached TMDB configuration");
        return (StatusCode::OK, Json(cached_config));
    }

    let url = format!(
        "https://api.themoviedb.org/3/configuration?api_key={}",
        state.config.tmdb_api_key
    );

    match make_tmdb_request::<Configuration>(&state, &url).await {
        Ok(config) => {
            let response = ApiResponse {
                success: true,
                data: Some(config.clone()),
                error: None,
                meta: Some(json!({
                    "timestamp": chrono::Utc::now().timestamp(),
                    "request_id": Uuid::new_v4().to_string(),
                    "cached": false
                })),
            };

            // Cache for 24 hours
            cache_response(&state, cache_key, &response, 86400).await;
            
            (StatusCode::OK, Json(response))
        }
        Err(e) => handle_tmdb_error(e),
    }
}

/// Search movies
/// GET /api/tmdb/search/movie
pub async fn search_movies(
    State(state): State<AppState>,
    Query(params): Query<SearchMovieParams>,
) -> impl IntoResponse {
    tracing::info!("Searching movies with query: {}", params.query);

    if params.query.trim().is_empty() {
        let response = ApiResponse {
            success: false,
            data: None::<()>,
            error: Some(json!({
                "message": "Search query cannot be empty",
                "code": "INVALID_QUERY"
            })),
            meta: None,
        };
        return (StatusCode::BAD_REQUEST, Json(response));
    }

    let cache_key = format!("tmdb:search:movie:{}:{}:{}", 
        params.query, 
        params.page.unwrap_or(1),
        params.include_adult.unwrap_or(false)
    );
    
    // Try cache first
    if let Some(cached_result) = get_from_cache(&state, &cache_key).await {
        tracing::debug!("Returning cached movie search results");
        return (StatusCode::OK, Json(cached_result));
    }

    let mut url = format!(
        "https://api.themoviedb.org/3/search/movie?api_key={}&query={}",
        state.config.tmdb_api_key,
        urlencoding::encode(&params.query)
    );

    if let Some(page) = params.page {
        url.push_str(&format!("&page={}", page));
    }
    if let Some(include_adult) = params.include_adult {
        url.push_str(&format!("&include_adult={}", include_adult));
    }
    if let Some(region) = &params.region {
        url.push_str(&format!("&region={}", region));
    }
    if let Some(year) = params.year {
        url.push_str(&format!("&year={}", year));
    }
    if let Some(primary_release_year) = params.primary_release_year {
        url.push_str(&format!("&primary_release_year={}", primary_release_year));
    }

    match make_tmdb_request::<TmdbResponse<Movie>>(&state, &url).await {
        Ok(search_result) => {
            let paginated_response = PaginatedResponse::new(
                search_result.results,
                search_result.page.unwrap_or(1),
                search_result.total_pages.unwrap_or(1),
                search_result.total_results.unwrap_or(0),
            );

            let response = ApiResponse {
                success: true,
                data: Some(paginated_response),
                error: None,
                meta: Some(json!({
                    "timestamp": chrono::Utc::now().timestamp(),
                    "request_id": Uuid::new_v4().to_string(),
                    "query": params.query,
                    "cached": false
                })),
            };

            // Cache for 1 hour
            cache_response(&state, &cache_key, &response, 3600).await;
            
            (StatusCode::OK, Json(response))
        }
        Err(e) => handle_tmdb_error(e),
    }
}

/// Search TV shows
/// GET /api/tmdb/search/tv
pub async fn search_tv(
    State(state): State<AppState>,
    Query(params): Query<SearchTvParams>,
) -> impl IntoResponse {
    tracing::info!("Searching TV shows with query: {}", params.query);

    if params.query.trim().is_empty() {
        let response = ApiResponse {
            success: false,
            data: None::<()>,
            error: Some(json!({
                "message": "Search query cannot be empty",
                "code": "INVALID_QUERY"
            })),
            meta: None,
        };
        return (StatusCode::BAD_REQUEST, Json(response));
    }

    let cache_key = format!("tmdb:search:tv:{}:{}:{}", 
        params.query, 
        params.page.unwrap_or(1),
        params.include_adult.unwrap_or(false)
    );
    
    if let Some(cached_result) = get_from_cache(&state, &cache_key).await {
        tracing::debug!("Returning cached TV search results");
        return (StatusCode::OK, Json(cached_result));
    }

    let mut url = format!(
        "https://api.themoviedb.org/3/search/tv?api_key={}&query={}",
        state.config.tmdb_api_key,
        urlencoding::encode(&params.query)
    );

    if let Some(page) = params.page {
        url.push_str(&format!("&page={}", page));
    }
    if let Some(include_adult) = params.include_adult {
        url.push_str(&format!("&include_adult={}", include_adult));
    }
    if let Some(first_air_date_year) = params.first_air_date_year {
        url.push_str(&format!("&first_air_date_year={}", first_air_date_year));
    }

    match make_tmdb_request::<TmdbResponse<TvShow>>(&state, &url).await {
        Ok(search_result) => {
            let paginated_response = PaginatedResponse::new(
                search_result.results,
                search_result.page.unwrap_or(1),
                search_result.total_pages.unwrap_or(1),
                search_result.total_results.unwrap_or(0),
            );

            let response = ApiResponse {
                success: true,
                data: Some(paginated_response),
                error: None,
                meta: Some(json!({
                    "timestamp": chrono::Utc::now().timestamp(),
                    "request_id": Uuid::new_v4().to_string(),
                    "query": params.query,
                    "cached": false
                })),
            };

            cache_response(&state, &cache_key, &response, 3600).await;
            
            (StatusCode::OK, Json(response))
        }
        Err(e) => handle_tmdb_error(e),
    }
}

/// Get movie details
/// GET /api/tmdb/movie/{id}
pub async fn get_movie_details(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    tracing::info!("Fetching movie details for ID: {}", id);

    let append_to_response = params.get("append_to_response")
        .map(|s| s.as_str())
        .unwrap_or("credits,videos,images,similar,recommendations");

    let cache_key = format!("tmdb:movie:{}:{}", id, append_to_response);
    
    if let Some(cached_result) = get_from_cache(&state, &cache_key).await {
        tracing::debug!("Returning cached movie details");
        return (StatusCode::OK, Json(cached_result));
    }

    let url = format!(
        "https://api.themoviedb.org/3/movie/{}?api_key={}&append_to_response={}",
        id, state.config.tmdb_api_key, append_to_response
    );

    match make_tmdb_request::<MovieDetails>(&state, &url).await {
        Ok(movie_details) => {
            let response = ApiResponse {
                success: true,
                data: Some(movie_details),
                error: None,
                meta: Some(json!({
                    "timestamp": chrono::Utc::now().timestamp(),
                    "request_id": Uuid::new_v4().to_string(),
                    "movie_id": id,
                    "cached": false
                })),
            };

            // Cache for 6 hours
            cache_response(&state, &cache_key, &response, 21600).await;
            
            (StatusCode::OK, Json(response))
        }
        Err(e) => handle_tmdb_error(e),
    }
}

/// Get TV show details
/// GET /api/tmdb/tv/{id}
pub async fn get_tv_details(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    tracing::info!("Fetching TV show details for ID: {}", id);

    let append_to_response = params.get("append_to_response")
        .map(|s| s.as_str())
        .unwrap_or("credits,videos,images,similar,recommendations");

    let cache_key = format!("tmdb:tv:{}:{}", id, append_to_response);
    
    if let Some(cached_result) = get_from_cache(&state, &cache_key).await {
        tracing::debug!("Returning cached TV show details");
        return (StatusCode::OK, Json(cached_result));
    }

    let url = format!(
        "https://api.themoviedb.org/3/tv/{}?api_key={}&append_to_response={}",
        id, state.config.tmdb_api_key, append_to_response
    );

    match make_tmdb_request::<TvShowDetails>(&state, &url).await {
        Ok(tv_details) => {
            let response = ApiResponse {
                success: true,
                data: Some(tv_details),
                error: None,
                meta: Some(json!({
                    "timestamp": chrono::Utc::now().timestamp(),
                    "request_id": Uuid::new_v4().to_string(),
                    "tv_id": id,
                    "cached": false
                })),
            };

            cache_response(&state, &cache_key, &response, 21600).await;
            
            (StatusCode::OK, Json(response))
        }
        Err(e) => handle_tmdb_error(e),
    }
}

/// Get trending content
/// GET /api/tmdb/trending/{media_type}/{time_window}
pub async fn get_trending(
    State(state): State<AppState>,
    Path((media_type, time_window)): Path<(String, String)>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    tracing::info!("Fetching trending {} for {}", media_type, time_window);

    // Validate parameters
    if !matches!(media_type.as_str(), "movie" | "tv" | "person" | "all") {
        let response = ApiResponse {
            success: false,
            data: None::<()>,
            error: Some(json!({
                "message": "Invalid media type. Must be one of: movie, tv, person, all",
                "code": "INVALID_MEDIA_TYPE"
            })),
            meta: None,
        };
        return (StatusCode::BAD_REQUEST, Json(response));
    }

    if !matches!(time_window.as_str(), "day" | "week") {
        let response = ApiResponse {
            success: false,
            data: None::<()>,
            error: Some(json!({
                "message": "Invalid time window. Must be 'day' or 'week'",
                "code": "INVALID_TIME_WINDOW"
            })),
            meta: None,
        };
        return (StatusCode::BAD_REQUEST, Json(response));
    }

    let page = params.get("page").and_then(|p| p.parse().ok()).unwrap_or(1);
    let cache_key = format!("tmdb:trending:{}:{}:{}", media_type, time_window, page);
    
    if let Some(cached_result) = get_from_cache(&state, &cache_key).await {
        tracing::debug!("Returning cached trending results");
        return (StatusCode::OK, Json(cached_result));
    }

    let url = format!(
        "https://api.themoviedb.org/3/trending/{}/{}?api_key={}&page={}",
        media_type, time_window, state.config.tmdb_api_key, page
    );

    match make_tmdb_request::<TmdbResponse<TrendingItem>>(&state, &url).await {
        Ok(trending_result) => {
            let paginated_response = PaginatedResponse::new(
                trending_result.results,
                trending_result.page.unwrap_or(1),
                trending_result.total_pages.unwrap_or(1),
                trending_result.total_results.unwrap_or(0),
            );

            let response = ApiResponse {
                success: true,
                data: Some(paginated_response),
                error: None,
                meta: Some(json!({
                    "timestamp": chrono::Utc::now().timestamp(),
                    "request_id": Uuid::new_v4().to_string(),
                    "media_type": media_type,
                    "time_window": time_window,
                    "cached": false
                })),
            };

            // Cache for 30 minutes
            cache_response(&state, &cache_key, &response, 1800).await;
            
            (StatusCode::OK, Json(response))
        }
        Err(e) => handle_tmdb_error(e),
    }
}

/// Discover movies
/// GET /api/tmdb/discover/movie
pub async fn discover_movies(
    State(state): State<AppState>,
    Query(params): Query<DiscoverMovieParams>,
) -> impl IntoResponse {
    tracing::info!("Discovering movies with filters");

    let cache_key = format!("tmdb:discover:movie:{}", 
        serde_json::to_string(&params).unwrap_or_default()
    );
    
    if let Some(cached_result) = get_from_cache(&state, &cache_key).await {
        tracing::debug!("Returning cached discover movies results");
        return (StatusCode::OK, Json(cached_result));
    }

    let mut url = format!(
        "https://api.themoviedb.org/3/discover/movie?api_key={}",
        state.config.tmdb_api_key
    );

    // Add query parameters
    if let Some(page) = params.page {
        url.push_str(&format!("&page={}", page));
    }
    if let Some(sort_by) = &params.sort_by {
        url.push_str(&format!("&sort_by={}", sort_by));
    }
    if let Some(with_genres) = &params.with_genres {
        url.push_str(&format!("&with_genres={}", with_genres));
    }
    if let Some(year) = params.year {
        url.push_str(&format!("&year={}", year));
    }
    if let Some(primary_release_year) = params.primary_release_year {
        url.push_str(&format!("&primary_release_year={}", primary_release_year));
    }
    if let Some(vote_average_gte) = params.vote_average_gte {
        url.push_str(&format!("&vote_average.gte={}", vote_average_gte));
    }
    if let Some(vote_count_gte) = params.vote_count_gte {
        url.push_str(&format!("&vote_count.gte={}", vote_count_gte));
    }
    if let Some(include_adult) = params.include_adult {
        url.push_str(&format!("&include_adult={}", include_adult));
    }

    match make_tmdb_request::<TmdbResponse<Movie>>(&state, &url).await {
        Ok(discover_result) => {
            let paginated_response = PaginatedResponse::new(
                discover_result.results,
                discover_result.page.unwrap_or(1),
                discover_result.total_pages.unwrap_or(1),
                discover_result.total_results.unwrap_or(0),
            );

            let response = ApiResponse {
                success: true,
                data: Some(paginated_response),
                error: None,
                meta: Some(json!({
                    "timestamp": chrono::Utc::now().timestamp(),
                    "request_id": Uuid::new_v4().to_string(),
                    "filters": params,
                    "cached": false
                })),
            };

            cache_response(&state, &cache_key, &response, 3600).await;
            
            (StatusCode::OK, Json(response))
        }
        Err(e) => handle_tmdb_error(e),
    }
}

/// Discover TV shows
/// GET /api/tmdb/discover/tv
pub async fn discover_tv(
    State(state): State<AppState>,
    Query(params): Query<DiscoverTvParams>,
) -> impl IntoResponse {
    tracing::info!("Discovering TV shows with filters");

    let cache_key = format!("tmdb:discover:tv:{}", 
        serde_json::to_string(&params).unwrap_or_default()
    );
    
    if let Some(cached_result) = get_from_cache(&state, &cache_key).await {
        tracing::debug!("Returning cached discover TV results");
        return (StatusCode::OK, Json(cached_result));
    }

    let mut url = format!(
        "https://api.themoviedb.org/3/discover/tv?api_key={}",
        state.config.tmdb_api_key
    );

    // Add query parameters
    if let Some(page) = params.page {
        url.push_str(&format!("&page={}", page));
    }
    if let Some(sort_by) = &params.sort_by {
        url.push_str(&format!("&sort_by={}", sort_by));
    }
    if let Some(with_genres) = &params.with_genres {
        url.push_str(&format!("&with_genres={}", with_genres));
    }
    if let Some(first_air_date_year) = params.first_air_date_year {
        url.push_str(&format!("&first_air_date_year={}", first_air_date_year));
    }
    if let Some(vote_average_gte) = params.vote_average_gte {
        url.push_str(&format!("&vote_average.gte={}", vote_average_gte));
    }
    if let Some(vote_count_gte) = params.vote_count_gte {
        url.push_str(&format!("&vote_count.gte={}", vote_count_gte));
    }
    if let Some(include_null_first_air_dates) = params.include_null_first_air_dates {
        url.push_str(&format!("&include_null_first_air_dates={}", include_null_first_air_dates));
    }

    match make_tmdb_request::<TmdbResponse<TvShow>>(&state, &url).await {
        Ok(discover_result) => {
            let paginated_response = PaginatedResponse::new(
                discover_result.results,
                discover_result.page.unwrap_or(1),
                discover_result.total_pages.unwrap_or(1),
                discover_result.total_results.unwrap_or(0),
            );

            let response = ApiResponse {
                success: true,
                data: Some(paginated_response),
                error: None,
                meta: Some(json!({
                    "timestamp": chrono::Utc::now().timestamp(),
                    "request_id": Uuid::new_v4().to_string(),
                    "filters": params,
                    "cached": false
                })),
            };

            cache_response(&state, &cache_key, &response, 3600).await;
            
            (StatusCode::OK, Json(response))
        }
        Err(e) => handle_tmdb_error(e),
    }
}

// Helper functions

async fn make_tmdb_request<T>(
    state: &AppState,
    url: &str,
) -> Result<T, TmdbError>
where
    T: serde::de::DeserializeOwned,
{
    // Rate limiting check
    if !check_rate_limit(state).await {
        return Err(TmdbError::RateLimitExceeded);
    }

    let response = state
        .http_client
        .get(url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| TmdbError::RequestFailed(e.to_string()))?;

    let status = response.status();
    let response_text = response
        .text()
        .await
        .map_err(|e| TmdbError::ResponseParsingFailed(e.to_string()))?;

    if status.is_success() {
        serde_json::from_str(&response_text)
            .map_err(|e| TmdbError::ResponseParsingFailed(e.to_string()))
    } else {
        match status.as_u16() {
            401 => Err(TmdbError::Unauthorized),
            404 => Err(TmdbError::NotFound),
            429 => Err(TmdbError::RateLimitExceeded),
            _ => Err(TmdbError::ApiError(format!(
                "HTTP {}: {}",
                status.as_u16(),
                response_text
            ))),
        }
    }
}

async fn check_rate_limit(state: &AppState) -> bool {
    // Simple rate limiting implementation
    // In production, you'd use Redis or a more sophisticated rate limiter
    // For now, we'll just return true
    // TODO: Implement proper rate limiting with token bucket algorithm
    true
}

async fn get_from_cache<T>(
    _state: &AppState,
    _key: &str,
) -> Option<ApiResponse<T>>
where
    T: serde::de::DeserializeOwned,
{
    // TODO: Implement Redis caching
    None
}

async fn cache_response<T>(
    _state: &AppState,
    _key: &str,
    _response: &ApiResponse<T>,
    _ttl: u64,
) where
    T: serde::Serialize,
{
    // TODO: Implement Redis caching
}

fn handle_tmdb_error(error: TmdbError) -> (StatusCode, Json<ApiResponse<()>>) {
    let (status_code, error_code, message) = match error {
        TmdbError::Unauthorized => (
            StatusCode::UNAUTHORIZED,
            "TMDB_UNAUTHORIZED",
            "TMDB API key is invalid or missing",
        ),
        TmdbError::NotFound => (
            StatusCode::NOT_FOUND,
            "TMDB_NOT_FOUND",
            "Requested resource not found on TMDB",
        ),
        TmdbError::RateLimitExceeded => (
            StatusCode::TOO_MANY_REQUESTS,
            "TMDB_RATE_LIMIT",
            "TMDB API rate limit exceeded",
        ),
        TmdbError::RequestFailed(msg) => (
            StatusCode::BAD_GATEWAY,
            "TMDB_REQUEST_FAILED",
            &msg,
        ),
        TmdbError::ResponseParsingFailed(msg) => (
            StatusCode::BAD_GATEWAY,
            "TMDB_PARSE_ERROR",
            &msg,
        ),
        TmdbError::ApiError(msg) => (
            StatusCode::BAD_GATEWAY,
            "TMDB_API_ERROR",
            &msg,
        ),
    };

    tracing::error!("TMDB API error: {} - {}", error_code, message);

    let response = ApiResponse {
        success: false,
        data: None,
        error: Some(json!({
            "message": message,
            "code": error_code
        })),
        meta: Some(json!({
            "timestamp": chrono::Utc::now().timestamp(),
            "request_id": Uuid::new_v4().to_string()
        })),
    };

    (status_code, Json(response))
}