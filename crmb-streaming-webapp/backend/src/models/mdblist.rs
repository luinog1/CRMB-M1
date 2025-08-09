//! MDBList API models and types
//!
//! This module provides data structures for MDBList API responses
//! and domain models for ratings, lists, and metadata.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MDBList media types
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MdbListMediaType {
    Movie,
    Show,
}

/// MDBList provider types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MdbListProvider {
    Imdb,
    Tmdb,
    Tvdb,
    Trakt,
}

/// Search parameters for MDBList
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MdbListSearchParams {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<MdbListMediaType>,
}

/// MDBList search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MdbListSearchResponse {
    pub search: Vec<MdbListSearchItem>,
    pub total: u32,
}

/// MDBList search item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MdbListSearchItem {
    pub id: String,
    pub title: String,
    pub year: u32,
    pub score: f32,
    #[serde(rename = "type")]
    pub media_type: MdbListMediaType,
    pub imdbid: Option<String>,
    pub traktid: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tmdbid: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tvdbid: Option<u64>,
}

/// MDBList detailed item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MdbListItem {
    pub released: String,
    pub description: String,
    pub runtime: u32,
    pub tmdbid: u64,
    pub language: String,
    pub country: String,
    pub certification: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commonsense: Option<String>,
    pub status: String,
    pub trailer: String,
    pub poster: String,
    pub backdrop: String,
    pub apiused: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<MdbListRating>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genres: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cast: Option<Vec<MdbListPerson>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crew: Option<Vec<MdbListPerson>>,
}

/// MDBList rating information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MdbListRating {
    pub imdb: Option<f32>,
    pub tmdb: Option<f32>,
    pub rotten_tomatoes: Option<f32>,
    pub metacritic: Option<f32>,
    pub average: Option<f32>,
}

/// MDBList person (cast/crew)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MdbListPerson {
    pub name: String,
    pub role: Option<String>,
    pub image: Option<String>,
}

/// MDBList user lists
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MdbListUserLists {
    pub lists: Vec<MdbListList>,
}

/// MDBList list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MdbListList {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub items: Vec<MdbListListItem>,
    pub item_count: u32,
    pub public: bool,
}

/// MDBList list item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MdbListListItem {
    pub title: String,
    pub year: u32,
    #[serde(rename = "type")]
    pub media_type: MdbListMediaType,
    pub imdbid: Option<String>,
    pub tmdbid: Option<u64>,
    pub poster: Option<String>,
    pub backdrop: Option<String>,
}

/// MDBList trending response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MdbListTrendingResponse {
    pub movies: Vec<MdbListTrendingItem>,
    pub shows: Vec<MdbListTrendingItem>,
}

/// MDBList trending item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MdbListTrendingItem {
    pub title: String,
    pub year: u32,
    #[serde(rename = "type")]
    pub media_type: MdbListMediaType,
    pub imdbid: String,
    pub tmdbid: u64,
    pub poster: String,
    pub backdrop: String,
    pub rating: Option<f32>,
}

/// Enhanced metadata combining TMDB and MDBList data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedMediaMetadata {
    pub tmdb_data: crate::models::tmdb::Movie,
    pub mdblist_data: Option<MdbListItem>,
    pub ratings: Option<MdbListRating>,
    pub user_lists: Option<Vec<String>>,
}

/// MDBList API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MdbListConfig {
    pub api_key: String,
    pub base_url: String,
    pub timeout: u64,
    pub max_retries: u32,
    pub cache_ttl: u64,
}