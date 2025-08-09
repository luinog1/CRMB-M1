use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Manifest {
    pub id: String,
    pub version: String,
    pub name: String,
    pub description: String,
    pub logo: Option<String>,
    pub background: Option<String>,
    pub types: Vec<String>,
    pub catalogs: Vec<CatalogDefinition>,
    pub resources: Vec<String>,
    #[serde(rename = "idPrefixes")]
    pub id_prefixes: Option<Vec<String>>,
    #[serde(rename = "behaviorHints")]
    pub behavior_hints: Option<BehaviorHints>,
    #[serde(rename = "contactEmail")]
    pub contact_email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CatalogDefinition {
    #[serde(rename = "type")]
    pub catalog_type: String,
    pub id: String,
    pub name: String,
    pub extra: Option<Vec<ExtraProperty>>,
    #[serde(rename = "pageSize")]
    pub page_size: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExtraProperty {
    pub name: String,
    #[serde(rename = "isRequired")]
    pub is_required: Option<bool>,
    pub options: Option<Vec<String>>,
    #[serde(rename = "optionsLimit")]
    pub options_limit: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BehaviorHints {
    pub adult: Option<bool>,
    pub p2p: Option<bool>,
    #[serde(rename = "configurable")]
    pub configurable: Option<bool>,
    #[serde(rename = "configurationRequired")]
    pub configuration_required: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CatalogResponse {
    pub metas: Vec<MetaPreview>,
    #[serde(rename = "cacheMaxAge")]
    pub cache_max_age: Option<u32>,
    #[serde(rename = "staleRevalidate")]
    pub stale_revalidate: Option<u32>,
    #[serde(rename = "staleError")]
    pub stale_error: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MetaPreview {
    pub id: String,
    #[serde(rename = "type")]
    pub meta_type: String,
    pub name: String,
    pub poster: Option<String>,
    pub background: Option<String>,
    pub logo: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "releaseInfo")]
    pub release_info: Option<String>,
    #[serde(rename = "imdbRating")]
    pub imdb_rating: Option<f64>,
    pub genres: Option<Vec<String>>,
    pub year: Option<u32>,
    #[serde(rename = "posterShape")]
    pub poster_shape: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MetaResponse {
    pub meta: MetaDetail,
    #[serde(rename = "cacheMaxAge")]
    pub cache_max_age: Option<u32>,
    #[serde(rename = "staleRevalidate")]
    pub stale_revalidate: Option<u32>,
    #[serde(rename = "staleError")]
    pub stale_error: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MetaDetail {
    pub id: String,
    #[serde(rename = "type")]
    pub meta_type: String,
    pub name: String,
    pub poster: Option<String>,
    pub background: Option<String>,
    pub logo: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "releaseInfo")]
    pub release_info: Option<String>,
    #[serde(rename = "imdbRating")]
    pub imdb_rating: Option<f64>,
    pub genres: Option<Vec<String>>,
    pub year: Option<u32>,
    pub cast: Option<Vec<String>>,
    pub director: Option<Vec<String>>,
    pub writer: Option<Vec<String>>,
    pub country: Option<String>,
    pub language: Option<String>,
    pub runtime: Option<String>,
    pub website: Option<String>,
    #[serde(rename = "behaviorHints")]
    pub behavior_hints: Option<MetaBehaviorHints>,
    pub videos: Option<Vec<Video>>,
    pub links: Option<Vec<MetaLink>>,
    #[serde(rename = "trailerStreams")]
    pub trailer_streams: Option<Vec<Stream>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MetaBehaviorHints {
    #[serde(rename = "defaultVideoId")]
    pub default_video_id: Option<String>,
    #[serde(rename = "hasScheduledVideos")]
    pub has_scheduled_videos: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Video {
    pub id: String,
    pub title: String,
    #[serde(rename = "releaseInfo")]
    pub release_info: Option<String>,
    pub season: Option<u32>,
    pub episode: Option<u32>,
    pub year: Option<u32>,
    pub overview: Option<String>,
    pub thumbnail: Option<String>,
    pub streams: Option<Vec<Stream>>,
    pub available: Option<bool>,
    pub watched: Option<bool>,
    pub released: Option<String>,
    #[serde(rename = "trailerStreams")]
    pub trailer_streams: Option<Vec<Stream>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MetaLink {
    pub name: String,
    pub category: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StreamResponse {
    pub streams: Vec<Stream>,
    #[serde(rename = "cacheMaxAge")]
    pub cache_max_age: Option<u32>,
    #[serde(rename = "staleRevalidate")]
    pub stale_revalidate: Option<u32>,
    #[serde(rename = "staleError")]
    pub stale_error: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Stream {
    pub url: Option<String>,
    #[serde(rename = "ytId")]
    pub yt_id: Option<String>,
    #[serde(rename = "infoHash")]
    pub info_hash: Option<String>,
    #[serde(rename = "fileIdx")]
    pub file_idx: Option<u32>,
    pub name: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "behaviorHints")]
    pub behavior_hints: Option<StreamBehaviorHints>,
    #[serde(rename = "externalUrl")]
    pub external_url: Option<String>,
    #[serde(rename = "androidTvUrl")]
    pub android_tv_url: Option<String>,
    #[serde(rename = "tizen_url")]
    pub tizen_url: Option<String>,
    #[serde(rename = "webos_url")]
    pub webos_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StreamBehaviorHints {
    #[serde(rename = "notWebReady")]
    pub not_web_ready: Option<bool>,
    #[serde(rename = "bingeGroup")]
    pub binge_group: Option<String>,
    #[serde(rename = "countryWhitelist")]
    pub country_whitelist: Option<Vec<String>>,
    #[serde(rename = "proxyHeaders")]
    pub proxy_headers: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddonRequest {
    #[serde(rename = "type")]
    pub request_type: String,
    pub id: String,
    pub extra: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddonConfiguration {
    pub manifest_url: String,
    pub transport_url: Option<String>,
    pub flags: Option<HashMap<String, bool>>,
}

// Error types for Stremio addon
#[derive(Debug, thiserror::Error)]
pub enum StremioError {
    #[error("Invalid addon request: {0}")]
    InvalidRequest(String),
    
    #[error("Resource not found: {0}")]
    NotFound(String),
    
    #[error("Unsupported media type: {0}")]
    UnsupportedMediaType(String),
    
    #[error("Invalid ID format: {0}")]
    InvalidIdFormat(String),
    
    #[error("External service error: {0}")]
    ExternalServiceError(String),
    
    #[error("Cache error: {0}")]
    CacheError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

// Helper functions and implementations
impl Manifest {
    pub fn new_crmb_addon() -> Self {
        Self {
            id: "com.crmb.streaming".to_string(),
            version: "2.0.0".to_string(),
            name: "CRMB Streaming".to_string(),
            description: "High-performance streaming media center with TMDB integration".to_string(),
            logo: Some("https://crmb.app/logo.png".to_string()),
            background: Some("https://crmb.app/background.jpg".to_string()),
            types: vec!["movie".to_string(), "series".to_string()],
            catalogs: vec![
                CatalogDefinition {
                    catalog_type: "movie".to_string(),
                    id: "tmdb_popular_movies".to_string(),
                    name: "Popular Movies".to_string(),
                    extra: Some(vec![
                        ExtraProperty {
                            name: "genre".to_string(),
                            is_required: Some(false),
                            options: Some(vec![
                                "Action".to_string(),
                                "Comedy".to_string(),
                                "Drama".to_string(),
                                "Horror".to_string(),
                                "Sci-Fi".to_string(),
                            ]),
                            options_limit: None,
                        },
                        ExtraProperty {
                            name: "skip".to_string(),
                            is_required: Some(false),
                            options: None,
                            options_limit: None,
                        },
                    ]),
                    page_size: Some(20),
                },
                CatalogDefinition {
                    catalog_type: "series".to_string(),
                    id: "tmdb_popular_series".to_string(),
                    name: "Popular TV Series".to_string(),
                    extra: Some(vec![
                        ExtraProperty {
                            name: "genre".to_string(),
                            is_required: Some(false),
                            options: Some(vec![
                                "Action".to_string(),
                                "Comedy".to_string(),
                                "Drama".to_string(),
                                "Sci-Fi".to_string(),
                            ]),
                            options_limit: None,
                        },
                        ExtraProperty {
                            name: "skip".to_string(),
                            is_required: Some(false),
                            options: None,
                            options_limit: None,
                        },
                    ]),
                    page_size: Some(20),
                },
            ],
            resources: vec![
                "catalog".to_string(),
                "meta".to_string(),
                "stream".to_string(),
            ],
            id_prefixes: Some(vec!["tmdb:".to_string()]),
            behavior_hints: Some(BehaviorHints {
                adult: Some(false),
                p2p: Some(false),
                configurable: Some(false),
                configuration_required: Some(false),
            }),
            contact_email: Some("support@crmb.app".to_string()),
        }
    }
}

impl MetaPreview {
    pub fn from_tmdb_movie(movie: &crate::models::tmdb::Movie) -> Self {
        Self {
            id: format!("tmdb:{}", movie.id),
            meta_type: "movie".to_string(),
            name: movie.title.clone(),
            poster: movie.poster_path.as_ref().map(|p| {
                format!("https://image.tmdb.org/t/p/w342{}", p)
            }),
            background: movie.backdrop_path.as_ref().map(|p| {
                format!("https://image.tmdb.org/t/p/w1280{}", p)
            }),
            logo: None,
            description: movie.overview.clone(),
            release_info: movie.release_date.clone(),
            imdb_rating: Some(movie.vote_average),
            genres: None, // Will be populated from genre_ids if needed
            year: movie.release_date.as_ref().and_then(|date| {
                date.split('-').next()?.parse().ok()
            }),
            poster_shape: Some("poster".to_string()),
        }
    }

    pub fn from_tmdb_tv(tv: &crate::models::tmdb::TvShow) -> Self {
        Self {
            id: format!("tmdb:{}", tv.id),
            meta_type: "series".to_string(),
            name: tv.name.clone(),
            poster: tv.poster_path.as_ref().map(|p| {
                format!("https://image.tmdb.org/t/p/w342{}", p)
            }),
            background: tv.backdrop_path.as_ref().map(|p| {
                format!("https://image.tmdb.org/t/p/w1280{}", p)
            }),
            logo: None,
            description: tv.overview.clone(),
            release_info: tv.first_air_date.clone(),
            imdb_rating: Some(tv.vote_average),
            genres: None, // Will be populated from genre_ids if needed
            year: tv.first_air_date.as_ref().and_then(|date| {
                date.split('-').next()?.parse().ok()
            }),
            poster_shape: Some("poster".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_creation() {
        let manifest = Manifest::new_crmb_addon();
        assert_eq!(manifest.id, "com.crmb.streaming");
        assert_eq!(manifest.name, "CRMB Streaming");
        assert_eq!(manifest.types.len(), 2);
        assert_eq!(manifest.catalogs.len(), 2);
        assert_eq!(manifest.resources.len(), 3);
    }

    #[test]
    fn test_meta_preview_from_tmdb() {
        let movie = crate::models::tmdb::Movie {
            id: 123,
            title: "Test Movie".to_string(),
            original_title: "Test Movie".to_string(),
            overview: Some("A test movie".to_string()),
            poster_path: Some("/poster.jpg".to_string()),
            backdrop_path: Some("/backdrop.jpg".to_string()),
            release_date: Some("2023-01-01".to_string()),
            vote_average: 8.5,
            vote_count: 1000,
            popularity: 100.0,
            genre_ids: vec![28, 12],
            adult: false,
            video: false,
            original_language: "en".to_string(),
        };

        let meta = MetaPreview::from_tmdb_movie(&movie);
        assert_eq!(meta.id, "tmdb:123");
        assert_eq!(meta.meta_type, "movie");
        assert_eq!(meta.name, "Test Movie");
        assert_eq!(meta.year, Some(2023));
        assert!(meta.poster.is_some());
        assert!(meta.background.is_some());
    }
}