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
        stremio::*,
        tmdb::{Movie, TvShow},
        ApiResponse,
    },
};

/// Get addon manifest
/// GET /stremio/manifest.json
pub async fn get_manifest() -> impl IntoResponse {
    tracing::info!("Serving Stremio addon manifest");

    let manifest = Manifest::new_crmb_addon();
    
    let response = ApiResponse {
        success: true,
        data: Some(manifest),
        error: None,
        meta: Some(json!({
            "timestamp": chrono::Utc::now().timestamp(),
            "request_id": Uuid::new_v4().to_string(),
            "addon_version": "2.0.0"
        })),
    };

    (StatusCode::OK, Json(response))
}

/// Get catalog
/// GET /stremio/catalog/{type}/{id}.json
/// GET /stremio/catalog/{type}/{id}/{extra}.json
pub async fn get_catalog(
    State(state): State<AppState>,
    Path(params): Path<HashMap<String, String>>,
    Query(query_params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let catalog_type = params.get("type").cloned().unwrap_or_default();
    let catalog_id = params.get("id").cloned().unwrap_or_default();
    let extra = params.get("extra").cloned();

    tracing::info!(
        "Serving catalog: type={}, id={}, extra={:?}",
        catalog_type, catalog_id, extra
    );

    // Validate catalog type
    if !matches!(catalog_type.as_str(), "movie" | "series") {
        return handle_stremio_error(StremioError::UnsupportedMediaType(catalog_type));
    }

    // Parse extra parameters
    let mut filters = HashMap::new();
    if let Some(extra_str) = &extra {
        for param in extra_str.split('&') {
            if let Some((key, value)) = param.split_once('=') {
                filters.insert(key.to_string(), value.to_string());
            }
        }
    }

    // Add query parameters to filters
    for (key, value) in query_params {
        filters.insert(key, value);
    }

    let page: u32 = filters
        .get("skip")
        .and_then(|s| s.parse().ok())
        .map(|skip: u32| (skip / 20) + 1)
        .unwrap_or(1);

    let genre = filters.get("genre").cloned();

    match catalog_type.as_str() {
        "movie" => get_movie_catalog(&state, &catalog_id, page, genre).await,
        "series" => get_tv_catalog(&state, &catalog_id, page, genre).await,
        _ => handle_stremio_error(StremioError::UnsupportedMediaType(catalog_type)),
    }
}

/// Get metadata
/// GET /stremio/meta/{type}/{id}.json
pub async fn get_meta(
    State(state): State<AppState>,
    Path((media_type, id)): Path<(String, String)>,
) -> impl IntoResponse {
    tracing::info!("Serving metadata: type={}, id={}", media_type, id);

    // Validate media type
    if !matches!(media_type.as_str(), "movie" | "series") {
        return handle_stremio_error(StremioError::UnsupportedMediaType(media_type));
    }

    // Parse TMDB ID from Stremio ID format (tmdb:123)
    let tmdb_id = match parse_tmdb_id(&id) {
        Ok(id) => id,
        Err(e) => return handle_stremio_error(e),
    };

    match media_type.as_str() {
        "movie" => get_movie_meta(&state, tmdb_id).await,
        "series" => get_tv_meta(&state, tmdb_id).await,
        _ => handle_stremio_error(StremioError::UnsupportedMediaType(media_type)),
    }
}

/// Get streams
/// GET /stremio/stream/{type}/{id}.json
/// GET /stremio/stream/{type}/{id}/{extra}.json
pub async fn get_streams(
    State(state): State<AppState>,
    Path(params): Path<HashMap<String, String>>,
) -> impl IntoResponse {
    let media_type = params.get("type").cloned().unwrap_or_default();
    let id = params.get("id").cloned().unwrap_or_default();
    let extra = params.get("extra").cloned();

    tracing::info!(
        "Serving streams: type={}, id={}, extra={:?}",
        media_type, id, extra
    );

    // Validate media type
    if !matches!(media_type.as_str(), "movie" | "series") {
        return handle_stremio_error(StremioError::UnsupportedMediaType(media_type));
    }

    // Parse TMDB ID
    let tmdb_id = match parse_tmdb_id(&id) {
        Ok(id) => id,
        Err(e) => return handle_stremio_error(e),
    };

    // For now, return empty streams as we don't have actual stream sources
    // In a real implementation, this would query torrent trackers, streaming services, etc.
    let stream_response = StreamResponse {
        streams: vec![
            // Example placeholder stream
            Stream {
                url: Some("https://example.com/placeholder.mp4".to_string()),
                yt_id: None,
                info_hash: None,
                file_idx: None,
                name: Some("Placeholder Stream".to_string()),
                title: Some("Demo Stream - Not Functional".to_string()),
                description: Some("This is a placeholder stream for demonstration purposes".to_string()),
                behavior_hints: Some(StreamBehaviorHints {
                    not_web_ready: Some(true),
                    binge_group: None,
                    country_whitelist: None,
                    proxy_headers: None,
                }),
                external_url: None,
                android_tv_url: None,
                tizen_url: None,
                webos_url: None,
            },
        ],
        cache_max_age: Some(300), // 5 minutes
        stale_revalidate: Some(600), // 10 minutes
        stale_error: Some(86400), // 24 hours
    };

    let response = ApiResponse {
        success: true,
        data: Some(stream_response),
        error: None,
        meta: Some(json!({
            "timestamp": chrono::Utc::now().timestamp(),
            "request_id": Uuid::new_v4().to_string(),
            "media_type": media_type,
            "tmdb_id": tmdb_id,
            "note": "Placeholder streams - integrate with actual stream sources"
        })),
    };

    (StatusCode::OK, Json(response))
}

// Helper functions

async fn get_movie_catalog(
    state: &AppState,
    catalog_id: &str,
    page: u32,
    genre: Option<String>,
) -> (StatusCode, Json<ApiResponse<CatalogResponse>>) {
    let cache_key = format!("stremio:catalog:movie:{}:{}:{:?}", catalog_id, page, genre);
    
    // Try cache first
    if let Some(cached_result) = get_from_cache(state, &cache_key).await {
        tracing::debug!("Returning cached movie catalog");
        return (StatusCode::OK, Json(cached_result));
    }

    // Determine TMDB endpoint based on catalog ID
    let tmdb_endpoint = match catalog_id {
        "tmdb_popular_movies" => "popular",
        "tmdb_top_rated_movies" => "top_rated",
        "tmdb_upcoming_movies" => "upcoming",
        "tmdb_now_playing_movies" => "now_playing",
        _ => "popular", // Default fallback
    };

    let mut url = format!(
        "https://api.themoviedb.org/3/movie/{}?api_key={}&page={}",
        tmdb_endpoint, state.config.tmdb_api_key, page
    );

    // Add genre filter if specified
    if let Some(genre_name) = &genre {
        // In a real implementation, you'd map genre names to TMDB genre IDs
        // For now, we'll use the discover endpoint with genre filtering
        url = format!(
            "https://api.themoviedb.org/3/discover/movie?api_key={}&page={}&with_genres={}",
            state.config.tmdb_api_key, page, map_genre_to_id(genre_name)
        );
    }

    match make_tmdb_request::<crate::models::tmdb::TmdbResponse<Movie>>(state, &url).await {
        Ok(tmdb_response) => {
            let metas: Vec<MetaPreview> = tmdb_response
                .results
                .iter()
                .map(MetaPreview::from_tmdb_movie)
                .collect();

            let catalog_response = CatalogResponse {
                metas,
                cache_max_age: Some(3600), // 1 hour
                stale_revalidate: Some(7200), // 2 hours
                stale_error: Some(86400), // 24 hours
            };

            let response = ApiResponse {
                success: true,
                data: Some(catalog_response),
                error: None,
                meta: Some(json!({
                    "timestamp": chrono::Utc::now().timestamp(),
                    "request_id": Uuid::new_v4().to_string(),
                    "catalog_id": catalog_id,
                    "page": page,
                    "genre": genre,
                    "cached": false
                })),
            };

            // Cache for 1 hour
            cache_response(state, &cache_key, &response, 3600).await;
            
            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            tracing::error!("Failed to fetch movie catalog: {}", e);
            handle_stremio_error(StremioError::ExternalServiceError(e.to_string()))
        }
    }
}

async fn get_tv_catalog(
    state: &AppState,
    catalog_id: &str,
    page: u32,
    genre: Option<String>,
) -> (StatusCode, Json<ApiResponse<CatalogResponse>>) {
    let cache_key = format!("stremio:catalog:tv:{}:{}:{:?}", catalog_id, page, genre);
    
    if let Some(cached_result) = get_from_cache(state, &cache_key).await {
        tracing::debug!("Returning cached TV catalog");
        return (StatusCode::OK, Json(cached_result));
    }

    let tmdb_endpoint = match catalog_id {
        "tmdb_popular_series" => "popular",
        "tmdb_top_rated_series" => "top_rated",
        "tmdb_on_the_air_series" => "on_the_air",
        "tmdb_airing_today_series" => "airing_today",
        _ => "popular",
    };

    let mut url = format!(
        "https://api.themoviedb.org/3/tv/{}?api_key={}&page={}",
        tmdb_endpoint, state.config.tmdb_api_key, page
    );

    if let Some(genre_name) = &genre {
        url = format!(
            "https://api.themoviedb.org/3/discover/tv?api_key={}&page={}&with_genres={}",
            state.config.tmdb_api_key, page, map_genre_to_id(genre_name)
        );
    }

    match make_tmdb_request::<crate::models::tmdb::TmdbResponse<TvShow>>(state, &url).await {
        Ok(tmdb_response) => {
            let metas: Vec<MetaPreview> = tmdb_response
                .results
                .iter()
                .map(MetaPreview::from_tmdb_tv)
                .collect();

            let catalog_response = CatalogResponse {
                metas,
                cache_max_age: Some(3600),
                stale_revalidate: Some(7200),
                stale_error: Some(86400),
            };

            let response = ApiResponse {
                success: true,
                data: Some(catalog_response),
                error: None,
                meta: Some(json!({
                    "timestamp": chrono::Utc::now().timestamp(),
                    "request_id": Uuid::new_v4().to_string(),
                    "catalog_id": catalog_id,
                    "page": page,
                    "genre": genre,
                    "cached": false
                })),
            };

            cache_response(state, &cache_key, &response, 3600).await;
            
            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            tracing::error!("Failed to fetch TV catalog: {}", e);
            handle_stremio_error(StremioError::ExternalServiceError(e.to_string()))
        }
    }
}

async fn get_movie_meta(
    state: &AppState,
    tmdb_id: u32,
) -> (StatusCode, Json<ApiResponse<MetaResponse>>) {
    let cache_key = format!("stremio:meta:movie:{}", tmdb_id);
    
    if let Some(cached_result) = get_from_cache(state, &cache_key).await {
        tracing::debug!("Returning cached movie metadata");
        return (StatusCode::OK, Json(cached_result));
    }

    let url = format!(
        "https://api.themoviedb.org/3/movie/{}?api_key={}&append_to_response=credits,videos,images",
        tmdb_id, state.config.tmdb_api_key
    );

    match make_tmdb_request::<crate::models::tmdb::MovieDetails>(state, &url).await {
        Ok(movie_details) => {
            let meta_detail = convert_movie_to_meta_detail(&movie_details);
            
            let meta_response = MetaResponse {
                meta: meta_detail,
                cache_max_age: Some(21600), // 6 hours
                stale_revalidate: Some(43200), // 12 hours
                stale_error: Some(86400), // 24 hours
            };

            let response = ApiResponse {
                success: true,
                data: Some(meta_response),
                error: None,
                meta: Some(json!({
                    "timestamp": chrono::Utc::now().timestamp(),
                    "request_id": Uuid::new_v4().to_string(),
                    "tmdb_id": tmdb_id,
                    "cached": false
                })),
            };

            cache_response(state, &cache_key, &response, 21600).await;
            
            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            tracing::error!("Failed to fetch movie metadata: {}", e);
            handle_stremio_error(StremioError::ExternalServiceError(e.to_string()))
        }
    }
}

async fn get_tv_meta(
    state: &AppState,
    tmdb_id: u32,
) -> (StatusCode, Json<ApiResponse<MetaResponse>>) {
    let cache_key = format!("stremio:meta:tv:{}", tmdb_id);
    
    if let Some(cached_result) = get_from_cache(state, &cache_key).await {
        tracing::debug!("Returning cached TV metadata");
        return (StatusCode::OK, Json(cached_result));
    }

    let url = format!(
        "https://api.themoviedb.org/3/tv/{}?api_key={}&append_to_response=credits,videos,images",
        tmdb_id, state.config.tmdb_api_key
    );

    match make_tmdb_request::<crate::models::tmdb::TvShowDetails>(state, &url).await {
        Ok(tv_details) => {
            let meta_detail = convert_tv_to_meta_detail(&tv_details);
            
            let meta_response = MetaResponse {
                meta: meta_detail,
                cache_max_age: Some(21600),
                stale_revalidate: Some(43200),
                stale_error: Some(86400),
            };

            let response = ApiResponse {
                success: true,
                data: Some(meta_response),
                error: None,
                meta: Some(json!({
                    "timestamp": chrono::Utc::now().timestamp(),
                    "request_id": Uuid::new_v4().to_string(),
                    "tmdb_id": tmdb_id,
                    "cached": false
                })),
            };

            cache_response(state, &cache_key, &response, 21600).await;
            
            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            tracing::error!("Failed to fetch TV metadata: {}", e);
            handle_stremio_error(StremioError::ExternalServiceError(e.to_string()))
        }
    }
}

// Utility functions

fn parse_tmdb_id(stremio_id: &str) -> Result<u32, StremioError> {
    if let Some(id_str) = stremio_id.strip_prefix("tmdb:") {
        id_str
            .parse()
            .map_err(|_| StremioError::InvalidIdFormat(stremio_id.to_string()))
    } else {
        Err(StremioError::InvalidIdFormat(stremio_id.to_string()))
    }
}

fn map_genre_to_id(genre_name: &str) -> u32 {
    // Map genre names to TMDB genre IDs
    match genre_name.to_lowercase().as_str() {
        "action" => 28,
        "adventure" => 12,
        "animation" => 16,
        "comedy" => 35,
        "crime" => 80,
        "documentary" => 99,
        "drama" => 18,
        "family" => 10751,
        "fantasy" => 14,
        "history" => 36,
        "horror" => 27,
        "music" => 10402,
        "mystery" => 9648,
        "romance" => 10749,
        "science fiction" | "sci-fi" => 878,
        "thriller" => 53,
        "war" => 10752,
        "western" => 37,
        _ => 28, // Default to Action
    }
}

fn convert_movie_to_meta_detail(movie: &crate::models::tmdb::MovieDetails) -> MetaDetail {
    MetaDetail {
        id: format!("tmdb:{}", movie.id),
        meta_type: "movie".to_string(),
        name: movie.title.clone(),
        poster: movie.poster_path.as_ref().map(|p| {
            format!("https://image.tmdb.org/t/p/w500{}", p)
        }),
        background: movie.backdrop_path.as_ref().map(|p| {
            format!("https://image.tmdb.org/t/p/w1280{}", p)
        }),
        logo: None,
        description: movie.overview.clone(),
        release_info: movie.release_date.clone(),
        imdb_rating: Some(movie.vote_average),
        genres: Some(movie.genres.iter().map(|g| g.name.clone()).collect()),
        year: movie.release_date.as_ref().and_then(|date| {
            date.split('-').next()?.parse().ok()
        }),
        cast: movie.credits.as_ref().map(|credits| {
            credits.cast.iter().take(10).map(|c| c.name.clone()).collect()
        }),
        director: movie.credits.as_ref().map(|credits| {
            credits.crew.iter()
                .filter(|c| c.job == "Director")
                .map(|c| c.name.clone())
                .collect()
        }),
        writer: movie.credits.as_ref().map(|credits| {
            credits.crew.iter()
                .filter(|c| c.job == "Writer" || c.job == "Screenplay")
                .map(|c| c.name.clone())
                .collect()
        }),
        country: movie.production_countries.first().map(|c| c.name.clone()),
        language: Some(movie.original_language.clone()),
        runtime: movie.runtime.map(|r| format!("{} min", r)),
        website: movie.homepage.clone(),
        behavior_hints: None,
        videos: None, // TODO: Convert TMDB videos to Stremio videos
        links: None,
        trailer_streams: None,
    }
}

fn convert_tv_to_meta_detail(tv: &crate::models::tmdb::TvShowDetails) -> MetaDetail {
    MetaDetail {
        id: format!("tmdb:{}", tv.id),
        meta_type: "series".to_string(),
        name: tv.name.clone(),
        poster: tv.poster_path.as_ref().map(|p| {
            format!("https://image.tmdb.org/t/p/w500{}", p)
        }),
        background: tv.backdrop_path.as_ref().map(|p| {
            format!("https://image.tmdb.org/t/p/w1280{}", p)
        }),
        logo: None,
        description: tv.overview.clone(),
        release_info: tv.first_air_date.clone(),
        imdb_rating: Some(tv.vote_average),
        genres: Some(tv.genres.iter().map(|g| g.name.clone()).collect()),
        year: tv.first_air_date.as_ref().and_then(|date| {
            date.split('-').next()?.parse().ok()
        }),
        cast: tv.credits.as_ref().map(|credits| {
            credits.cast.iter().take(10).map(|c| c.name.clone()).collect()
        }),
        director: None, // TV shows don't typically have a single director
        writer: tv.credits.as_ref().map(|credits| {
            credits.crew.iter()
                .filter(|c| c.job == "Writer" || c.job == "Creator")
                .map(|c| c.name.clone())
                .collect()
        }),
        country: tv.production_countries.first().map(|c| c.name.clone()),
        language: Some(tv.original_language.clone()),
        runtime: tv.episode_run_time.first().map(|r| format!("{} min", r)),
        website: tv.homepage.clone(),
        behavior_hints: None,
        videos: None, // TODO: Convert episodes to Stremio videos
        links: None,
        trailer_streams: None,
    }
}

async fn make_tmdb_request<T>(
    state: &AppState,
    url: &str,
) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
where
    T: serde::de::DeserializeOwned,
{
    let response = state
        .http_client
        .get(url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    
    Ok(response)
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

fn handle_stremio_error(error: StremioError) -> (StatusCode, Json<ApiResponse<()>>) {
    let (status_code, error_code, message) = match error {
        StremioError::InvalidRequest(msg) => (
            StatusCode::BAD_REQUEST,
            "INVALID_REQUEST",
            msg,
        ),
        StremioError::NotFound(msg) => (
            StatusCode::NOT_FOUND,
            "NOT_FOUND",
            msg,
        ),
        StremioError::UnsupportedMediaType(media_type) => (
            StatusCode::BAD_REQUEST,
            "UNSUPPORTED_MEDIA_TYPE",
            format!("Unsupported media type: {}", media_type),
        ),
        StremioError::InvalidIdFormat(id) => (
            StatusCode::BAD_REQUEST,
            "INVALID_ID_FORMAT",
            format!("Invalid ID format: {}", id),
        ),
        StremioError::ExternalServiceError(msg) => (
            StatusCode::BAD_GATEWAY,
            "EXTERNAL_SERVICE_ERROR",
            msg,
        ),
        StremioError::CacheError(msg) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "CACHE_ERROR",
            msg,
        ),
        StremioError::SerializationError(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "SERIALIZATION_ERROR",
            e.to_string(),
        ),
    };

    tracing::error!("Stremio addon error: {} - {}", error_code, message);

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