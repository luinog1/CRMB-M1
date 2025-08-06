use axum::{
    extract::{Query, State},
    response::Json,
    http::StatusCode,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::services::tmdb::TMDBService;
use crate::models::movie::{MovieResponse, SearchResponse};

#[derive(Deserialize)]
pub struct MovieQuery {
    pub page: Option<u32>,
    pub query: Option<String>,
}

#[derive(Debug)]
pub enum AppError {
    TmdbError(String),
    MissingQuery,
    InternalError,
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::TmdbError(msg) => (StatusCode::BAD_GATEWAY, format!("TMDB API error: {}", msg)),
            AppError::MissingQuery => (StatusCode::BAD_REQUEST, "Missing query parameter".to_string()),
            AppError::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
        };
        
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

pub async fn get_popular_movies(
    Query(params): Query<MovieQuery>,
    State(tmdb): State<Arc<TMDBService>>,
) -> Result<Json<MovieResponse>, AppError> {
    let page = params.page.unwrap_or(1);
    
    match tmdb.get_popular_movies(page).await {
        Ok(movies) => Ok(Json(movies)),
        Err(e) => Err(AppError::TmdbError(e.to_string())),
    }
}

pub async fn get_upcoming_movies(
    Query(params): Query<MovieQuery>,
    State(tmdb): State<Arc<TMDBService>>,
) -> Result<Json<MovieResponse>, AppError> {
    let page = params.page.unwrap_or(1);
    
    match tmdb.get_upcoming_movies(page).await {
        Ok(movies) => Ok(Json(movies)),
        Err(e) => Err(AppError::TmdbError(e.to_string())),
    }
}

pub async fn get_trending_movies(
    Query(params): Query<MovieQuery>,
    State(tmdb): State<Arc<TMDBService>>,
) -> Result<Json<MovieResponse>, AppError> {
    let page = params.page.unwrap_or(1);
    
    match tmdb.get_trending_movies(page).await {
        Ok(movies) => Ok(Json(movies)),
        Err(e) => Err(AppError::TmdbError(e.to_string())),
    }
}

pub async fn search_movies(
    Query(params): Query<MovieQuery>,
    State(tmdb): State<Arc<TMDBService>>,
) -> Result<Json<SearchResponse>, AppError> {
    let query = params.query.ok_or(AppError::MissingQuery)?;
    
    match tmdb.search_movies(&query, 1).await {
        Ok(results) => Ok(Json(results)),
        Err(e) => Err(AppError::TmdbError(e.to_string())),
    }
}