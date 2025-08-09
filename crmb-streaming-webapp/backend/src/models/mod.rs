pub mod auth;
pub mod api;
pub mod user;
pub mod tmdb;
pub mod stremio;
pub mod mdblist;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
            error: None,
        }
    }

    pub fn success_with_message(data: T, message: String) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: Some(message),
            error: None,
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            message: None,
            error: Some(error),
        }
    }

    pub fn error_with_message(error: String, message: String) -> Self {
        Self {
            success: false,
            data: None,
            message: Some(message),
            error: Some(error),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub page: u32,
    pub total_pages: u32,
    pub total_results: u32,
    pub has_next: bool,
    pub has_previous: bool,
}

impl<T> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, page: u32, total_pages: u32, total_results: u32) -> Self {
        Self {
            items,
            page,
            total_pages,
            total_results,
            has_next: page < total_pages,
            has_previous: page > 1,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub version: String,
    pub timestamp: String,
    pub database: String,
    pub tmdb_api: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MediaItem {
    pub id: i32,
    pub title: Option<String>,
    pub name: Option<String>, // For TV shows
    pub overview: Option<String>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub release_date: Option<String>,
    pub first_air_date: Option<String>, // For TV shows
    pub vote_average: f64,
    pub vote_count: i32,
    pub popularity: f64,
    pub genre_ids: Vec<i32>,
    pub adult: bool,
    pub video: bool,
    pub original_language: String,
    pub original_title: Option<String>,
    pub original_name: Option<String>, // For TV shows
    pub media_type: Option<String>,
}

impl MediaItem {
    pub fn get_title(&self) -> String {
        self.title
            .clone()
            .or_else(|| self.name.clone())
            .unwrap_or_else(|| "Unknown Title".to_string())
    }

    pub fn get_release_date(&self) -> Option<String> {
        self.release_date
            .clone()
            .or_else(|| self.first_air_date.clone())
    }

    pub fn get_original_title(&self) -> String {
        self.original_title
            .clone()
            .or_else(|| self.original_name.clone())
            .unwrap_or_else(|| self.get_title())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub status_code: u16,
    pub timestamp: String,
}

impl ErrorResponse {
    pub fn new(error: String, message: String, status_code: u16) -> Self {
        Self {
            error,
            message,
            status_code,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}