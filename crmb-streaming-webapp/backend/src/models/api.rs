//! Common API models and response types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Generic paginated response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub page: u32,
    pub results: Vec<T>,
    pub total_results: u32,
    pub total_pages: u32,
}

/// Generic API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub message: Option<String>,
}

/// Search query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub page: Option<u32>,
    pub include_adult: Option<bool>,
    pub region: Option<String>,
    pub year: Option<u32>,
    pub primary_release_year: Option<u32>,
}

/// Discover query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverQuery {
    pub page: Option<u32>,
    pub sort_by: Option<String>,
    pub with_genres: Option<String>,
    pub without_genres: Option<String>,
    pub with_companies: Option<String>,
    pub with_keywords: Option<String>,
    pub without_keywords: Option<String>,
    pub with_people: Option<String>,
    pub with_cast: Option<String>,
    pub with_crew: Option<String>,
    pub year: Option<u32>,
    pub primary_release_year: Option<u32>,
    pub primary_release_date_gte: Option<String>,
    pub primary_release_date_lte: Option<String>,
    pub release_date_gte: Option<String>,
    pub release_date_lte: Option<String>,
    pub vote_count_gte: Option<u32>,
    pub vote_count_lte: Option<u32>,
    pub vote_average_gte: Option<f32>,
    pub vote_average_lte: Option<f32>,
    pub with_runtime_gte: Option<u32>,
    pub with_runtime_lte: Option<u32>,
    pub region: Option<String>,
    pub include_adult: Option<bool>,
    pub include_video: Option<bool>,
    pub with_watch_monetization_types: Option<String>,
}

/// Health status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub timestamp: String,
    pub version: String,
    pub database: String,
    pub services: HashMap<String, String>,
}

/// Error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
    pub code: Option<String>,
    pub details: Option<serde_json::Value>,
}

/// Success response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessResponse<T> {
    pub success: bool,
    pub data: T,
    pub message: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            message: None,
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            message: None,
        }
    }

    pub fn with_message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }
}

impl<T> PaginatedResponse<T> {
    pub fn new(page: u32, results: Vec<T>, total_results: u32, total_pages: u32) -> Self {
        Self {
            page,
            results,
            total_results,
            total_pages,
        }
    }

    pub fn empty(page: u32) -> Self {
        Self {
            page,
            results: Vec::new(),
            total_results: 0,
            total_pages: 0,
        }
    }
}