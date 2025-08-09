use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TmdbResponse<T> {
    pub page: Option<u32>,
    pub results: Vec<T>,
    pub total_pages: Option<u32>,
    pub total_results: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Movie {
    pub id: i32,
    pub title: String,
    pub original_title: String,
    pub overview: Option<String>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub release_date: Option<String>,
    pub vote_average: f64,
    pub vote_count: i32,
    pub popularity: f64,
    pub genre_ids: Vec<i32>,
    pub adult: bool,
    pub video: bool,
    pub original_language: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TvShow {
    pub id: i32,
    pub name: String,
    pub original_name: String,
    pub overview: Option<String>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub first_air_date: Option<String>,
    pub vote_average: f64,
    pub vote_count: i32,
    pub popularity: f64,
    pub genre_ids: Vec<i32>,
    pub origin_country: Vec<String>,
    pub original_language: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Person {
    pub id: i32,
    pub name: String,
    pub profile_path: Option<String>,
    pub adult: bool,
    pub popularity: f64,
    pub known_for_department: String,
    pub known_for: Vec<KnownForItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum KnownForItem {
    Movie(Movie),
    TvShow(TvShow),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum SearchResult {
    Movie(Movie),
    TvShow(TvShow),
    Person(Person),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MovieDetails {
    pub id: i32,
    pub title: String,
    pub original_title: String,
    pub overview: Option<String>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub release_date: Option<String>,
    pub vote_average: f64,
    pub vote_count: i32,
    pub popularity: f64,
    pub genres: Vec<Genre>,
    pub adult: bool,
    pub video: bool,
    pub original_language: String,
    pub runtime: Option<i32>,
    pub budget: i64,
    pub revenue: i64,
    pub status: String,
    pub tagline: Option<String>,
    pub homepage: Option<String>,
    pub imdb_id: Option<String>,
    pub production_companies: Vec<ProductionCompany>,
    pub production_countries: Vec<ProductionCountry>,
    pub spoken_languages: Vec<SpokenLanguage>,
    pub belongs_to_collection: Option<Collection>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TvShowDetails {
    pub id: i32,
    pub name: String,
    pub original_name: String,
    pub overview: Option<String>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub first_air_date: Option<String>,
    pub last_air_date: Option<String>,
    pub vote_average: f64,
    pub vote_count: i32,
    pub popularity: f64,
    pub genres: Vec<Genre>,
    pub origin_country: Vec<String>,
    pub original_language: String,
    pub number_of_episodes: i32,
    pub number_of_seasons: i32,
    pub status: String,
    pub tagline: Option<String>,
    pub homepage: Option<String>,
    pub in_production: bool,
    pub languages: Vec<String>,
    pub last_episode_to_air: Option<Episode>,
    pub next_episode_to_air: Option<Episode>,
    pub networks: Vec<Network>,
    pub production_companies: Vec<ProductionCompany>,
    pub production_countries: Vec<ProductionCountry>,
    pub seasons: Vec<Season>,
    pub spoken_languages: Vec<SpokenLanguage>,
    pub episode_run_time: Vec<i32>,
    pub created_by: Vec<Creator>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Genre {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProductionCompany {
    pub id: i32,
    pub name: String,
    pub logo_path: Option<String>,
    pub origin_country: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProductionCountry {
    pub iso_3166_1: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpokenLanguage {
    pub english_name: String,
    pub iso_639_1: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Collection {
    pub id: i32,
    pub name: String,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Episode {
    pub id: i32,
    pub name: String,
    pub overview: Option<String>,
    pub vote_average: f64,
    pub vote_count: i32,
    pub air_date: Option<String>,
    pub episode_number: i32,
    pub production_code: Option<String>,
    pub runtime: Option<i32>,
    pub season_number: i32,
    pub show_id: i32,
    pub still_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Network {
    pub id: i32,
    pub name: String,
    pub logo_path: Option<String>,
    pub origin_country: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Season {
    pub id: i32,
    pub name: String,
    pub overview: Option<String>,
    pub poster_path: Option<String>,
    pub season_number: i32,
    pub episode_count: i32,
    pub air_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Creator {
    pub id: i32,
    pub name: String,
    pub profile_path: Option<String>,
    pub credit_id: String,
    pub gender: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Configuration {
    pub images: ImageConfiguration,
    pub change_keys: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageConfiguration {
    pub base_url: String,
    pub secure_base_url: String,
    pub backdrop_sizes: Vec<String>,
    pub logo_sizes: Vec<String>,
    pub poster_sizes: Vec<String>,
    pub profile_sizes: Vec<String>,
    pub still_sizes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Credits {
    pub id: i32,
    pub cast: Vec<CastMember>,
    pub crew: Vec<CrewMember>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CastMember {
    pub id: i32,
    pub name: String,
    pub character: String,
    pub credit_id: String,
    pub gender: Option<i32>,
    pub order: i32,
    pub profile_path: Option<String>,
    pub cast_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CrewMember {
    pub id: i32,
    pub name: String,
    pub job: String,
    pub department: String,
    pub credit_id: String,
    pub gender: Option<i32>,
    pub profile_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Videos {
    pub id: i32,
    pub results: Vec<Video>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Video {
    pub id: String,
    pub iso_639_1: String,
    pub iso_3166_1: String,
    pub key: String,
    pub name: String,
    pub site: String,
    pub size: i32,
    #[serde(rename = "type")]
    pub video_type: String,
    pub official: bool,
    pub published_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Images {
    pub id: i32,
    pub backdrops: Vec<ImageItem>,
    pub logos: Vec<ImageItem>,
    pub posters: Vec<ImageItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageItem {
    pub aspect_ratio: f64,
    pub file_path: String,
    pub height: i32,
    pub iso_639_1: Option<String>,
    pub vote_average: f64,
    pub vote_count: i32,
    pub width: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrendingResponse {
    pub page: u32,
    pub results: Vec<TrendingItem>,
    pub total_pages: u32,
    pub total_results: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum TrendingItem {
    Movie(Movie),
    TvShow(TvShow),
    Person(Person),
}

// Query parameters for TMDB API requests
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub page: Option<u32>,
    pub include_adult: Option<bool>,
    pub region: Option<String>,
    pub year: Option<u32>,
    pub primary_release_year: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscoverQuery {
    pub page: Option<u32>,
    pub region: Option<String>,
    pub sort_by: Option<String>,
    pub with_genres: Option<String>,
    pub without_genres: Option<String>,
    pub year: Option<u32>,
    pub primary_release_year: Option<u32>,
    pub vote_average_gte: Option<f64>,
    pub vote_average_lte: Option<f64>,
    pub vote_count_gte: Option<u32>,
    pub with_runtime_gte: Option<u32>,
    pub with_runtime_lte: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchMovieParams {
    pub query: String,
    pub page: Option<u32>,
    pub include_adult: Option<bool>,
    pub region: Option<String>,
    pub year: Option<u32>,
    pub primary_release_year: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchTvParams {
    pub query: String,
    pub page: Option<u32>,
    pub include_adult: Option<bool>,
    pub first_air_date_year: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscoverMovieParams {
    pub page: Option<u32>,
    pub region: Option<String>,
    pub sort_by: Option<String>,
    pub with_genres: Option<String>,
    pub without_genres: Option<String>,
    pub year: Option<u32>,
    pub primary_release_year: Option<u32>,
    pub vote_average_gte: Option<f64>,
    pub vote_average_lte: Option<f64>,
    pub vote_count_gte: Option<u32>,
    pub with_runtime_gte: Option<u32>,
    pub with_runtime_lte: Option<u32>,
    pub with_cast: Option<String>,
    pub with_crew: Option<String>,
    pub with_companies: Option<String>,
    pub with_keywords: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscoverTvParams {
    pub page: Option<u32>,
    pub sort_by: Option<String>,
    pub air_date_gte: Option<String>,
    pub air_date_lte: Option<String>,
    pub first_air_date_gte: Option<String>,
    pub first_air_date_lte: Option<String>,
    pub first_air_date_year: Option<u32>,
    pub timezone: Option<String>,
    pub vote_average_gte: Option<f64>,
    pub vote_count_gte: Option<u32>,
    pub with_genres: Option<String>,
    pub with_networks: Option<String>,
    pub without_genres: Option<String>,
    pub with_runtime_gte: Option<u32>,
    pub with_runtime_lte: Option<u32>,
    pub include_null_first_air_dates: Option<bool>,
    pub with_original_language: Option<String>,
    pub without_keywords: Option<String>,
    pub screened_theatrically: Option<bool>,
    pub with_companies: Option<String>,
    pub with_keywords: Option<String>,
}

// Error types for TMDB API
#[derive(Debug, thiserror::Error)]
pub enum TmdbError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    
    #[error("JSON parsing failed: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("API error: {status_code} - {message}")]
    ApiError { status_code: u16, message: String },
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Invalid API key")]
    InvalidApiKey,
    
    #[error("Resource not found")]
    NotFound,
    
    #[error("Cache error: {0}")]
    CacheError(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TmdbErrorResponse {
    pub success: bool,
    pub status_code: u32,
    pub status_message: String,
}

// Helper functions for image URLs
impl ImageConfiguration {
    pub fn get_poster_url(&self, path: &str, size: &str) -> String {
        format!("{}{}{}", self.secure_base_url, size, path)
    }
    
    pub fn get_backdrop_url(&self, path: &str, size: &str) -> String {
        format!("{}{}{}", self.secure_base_url, size, path)
    }
    
    pub fn get_profile_url(&self, path: &str, size: &str) -> String {
        format!("{}{}{}", self.secure_base_url, size, path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_url_generation() {
        let config = ImageConfiguration {
            base_url: "http://image.tmdb.org/t/p/".to_string(),
            secure_base_url: "https://image.tmdb.org/t/p/".to_string(),
            backdrop_sizes: vec!["w300".to_string(), "w780".to_string()],
            logo_sizes: vec!["w45".to_string(), "w92".to_string()],
            poster_sizes: vec!["w154".to_string(), "w342".to_string()],
            profile_sizes: vec!["w45".to_string(), "w185".to_string()],
            still_sizes: vec!["w92".to_string(), "w185".to_string()],
        };

        let poster_url = config.get_poster_url("/poster.jpg", "w342");
        assert_eq!(poster_url, "https://image.tmdb.org/t/p/w342/poster.jpg");

        let backdrop_url = config.get_backdrop_url("/backdrop.jpg", "w780");
        assert_eq!(backdrop_url, "https://image.tmdb.org/t/p/w780/backdrop.jpg");
    }
}