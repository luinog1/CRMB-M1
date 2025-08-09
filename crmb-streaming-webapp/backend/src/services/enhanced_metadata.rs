//! Enhanced metadata service combining TMDB and MDBList data
//!
//! This service provides:
//! - Dual API integration (TMDB + MDBList)
//! - Fallback mechanisms when one API is unavailable
//! - Enhanced metadata with high-quality images
//! - User-specific content from MDBList lists
//! - Optimized caching and performance

use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};

use crate::models::tmdb::{TmdbMovie, TmdbShow, TmdbSearchResponse};
use crate::models::mdblist::{MdbListItem, MdbListSearchParams, MdbListMediaType};
use crate::services::tmdb::TmdbService;
use crate::services::mdblist::MdbListService;
use crate::services::cache::{CacheService, CacheKeys};

/// Enhanced metadata combining TMDB and MDBList data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedMediaItem {
    /// TMDB ID
    pub tmdb_id: u64,
    /// IMDB ID (from MDBList)
    pub imdb_id: Option<String>,
    /// Media type
    pub media_type: MediaType,
    /// Title
    pub title: String,
    /// Original title
    pub original_title: String,
    /// Overview/synopsis
    pub overview: String,
    /// Release date
    pub release_date: Option<String>,
    /// Runtime in minutes
    pub runtime: Option<u32>,
    /// Genres
    pub genres: Vec<String>,
    /// High-quality poster URLs
    pub posters: PosterUrls,
    /// High-quality backdrop URLs
    pub backdrops: BackdropUrls,
    /// Clean logo URLs
    pub logos: Vec<String>,
    /// Cast and crew information
    pub credits: Credits,
    /// Ratings from various sources
    pub ratings: Ratings,
    /// Trailers and videos
    pub videos: Vec<Video>,
    /// MDBList specific data
    pub mdblist_data: Option<MdbListEnhancedData>,
    /// User-specific data (if available)
    pub user_data: Option<UserData>,
}

/// Media type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    Movie,
    Tv,
}

/// Poster URL collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PosterUrls {
    pub w500: Option<String>,
    pub w780: Option<String>,
    pub original: Option<String>,
}

/// Backdrop URL collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackdropUrls {
    pub w780: Option<String>,
    pub w1280: Option<String>,
    pub original: Option<String>,
}

/// Credits information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credits {
    pub cast: Vec<Person>,
    pub crew: Vec<Person>,
}

/// Person information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    pub id: u64,
    pub name: String,
    pub character: Option<String>,
    pub job: Option<String>,
    pub profile_path: Option<String>,
}

/// Ratings from different sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ratings {
    pub tmdb: Option<f64>,
    pub imdb: Option<f64>,
    pub rotten_tomatoes: Option<u32>,
    pub metacritic: Option<u32>,
}

/// Video information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Video {
    pub id: String,
    pub key: String,
    pub name: String,
    pub site: String,
    pub size: u32,
    pub video_type: String,
}

/// MDBList enhanced data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MdbListEnhancedData {
    pub score: Option<f64>,
    pub popularity: Option<f64>,
    pub language: Option<String>,
    pub country: Option<String>,
    pub certification: Option<String>,
}

/// User-specific data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserData {
    pub in_watchlist: bool,
    pub user_rating: Option<u32>,
    pub watched: bool,
    pub user_lists: Vec<String>,
}

/// Enhanced metadata service
#[derive(Clone)]
pub struct EnhancedMetadataService {
    tmdb_service: Arc<TmdbService>,
    mdblist_service: Arc<MdbListService>,
    cache: Arc<CacheService>,
}

impl EnhancedMetadataService {
    /// Create new enhanced metadata service
    pub fn new(
        tmdb_service: Arc<TmdbService>,
        mdblist_service: Arc<MdbListService>,
        cache: Arc<CacheService>,
    ) -> Self {
        Self {
            tmdb_service,
            mdblist_service,
            cache,
        }
    }

    /// Search for enhanced media items
    pub async fn search_enhanced(
        &self,
        query: &str,
        year: Option<u32>,
        media_type: Option<MediaType>,
        user_id: Option<&str>,
    ) -> Result<Vec<EnhancedMediaItem>, EnhancedMetadataError> {
        let cache_key = CacheKeys::enhanced_search(query, year, media_type);
        
        // Try cache first
        if let Some(cached) = self.cache.get::<Vec<EnhancedMediaItem>>(&cache_key).await {
            return Ok(self.apply_user_data(cached, user_id).await);
        }

        // Search TMDB as primary source
        let tmdb_results = self.search_tmdb(query, year, media_type.clone()).await;
        
        // Enhance with MDBList data
        let enhanced_items = match tmdb_results {
            Ok(results) => {
                self.enhance_tmdb_results(results, user_id).await
            },
            Err(e) => {
                warn!("TMDB search failed, trying MDBList fallback: {}", e);
                self.search_mdblist_fallback(query, year, media_type.clone()).await?
            }
        };

        // Cache the results
        let cache_duration = std::time::Duration::from_secs(900); // 15 minutes
        let _ = self.cache.set(&cache_key, &enhanced_items, cache_duration).await;

        Ok(enhanced_items)
    }

    /// Get enhanced media item by TMDB ID
    pub async fn get_by_tmdb_id(
        &self,
        tmdb_id: u64,
        media_type: MediaType,
        user_id: Option<&str>,
    ) -> Result<Option<EnhancedMediaItem>, EnhancedMetadataError> {
        let cache_key = CacheKeys::enhanced_by_tmdb_id(tmdb_id, media_type.clone());
        
        // Try cache first
        if let Some(cached) = self.cache.get::<EnhancedMediaItem>(&cache_key).await {
            return Ok(Some(self.apply_user_data_single(cached, user_id).await));
        }

        // Get from TMDB
        let tmdb_item = match media_type {
            MediaType::Movie => {
                self.tmdb_service.get_movie(tmdb_id).await
            },
            MediaType::Tv => {
                self.tmdb_service.get_tv_show(tmdb_id).await
            },
        };

        let enhanced = match tmdb_item {
            Ok(Some(item)) => {
                let enhanced = self.enhance_tmdb_item(item).await?;
                
                // Cache the result
                let cache_duration = std::time::Duration::from_secs(1800); // 30 minutes
                let _ = self.cache.set(&cache_key, &enhanced, cache_duration).await;
                
                Some(self.apply_user_data_single(enhanced, user_id).await)
            },
            Ok(None) => None,
            Err(e) => {
                error!("Failed to get TMDB item {}: {}", tmdb_id, e);
                None
            }
        };

        Ok(enhanced)
    }

    /// Search TMDB
    async fn search_tmdb(
        &self,
        query: &str,
        year: Option<u32>,
        media_type: Option<MediaType>,
    ) -> Result<TmdbSearchResponse, EnhancedMetadataError> {
        let tmdb_type = match media_type {
            Some(MediaType::Movie) => crate::models::tmdb::MediaType::Movie,
            Some(MediaType::Tv) => crate::models::tmdb::MediaType::Tv,
            None => crate::models::tmdb::MediaType::All,
        };

        self.tmdb_service
            .search(query, year, tmdb_type)
            .await
            .map_err(EnhancedMetadataError::TmdbError)
    }

    /// Enhance TMDB search results
    async fn enhance_tmdb_results(
        &self,
        results: TmdbSearchResponse,
        user_id: Option<&str>,
    ) -> Result<Vec<EnhancedMediaItem>, EnhancedMetadataError> {
        let mut enhanced_items = Vec::new();

        for result in results.results {
            if let Some(enhanced) = self.enhance_tmdb_search_result(result).await? {
                enhanced_items.push(enhanced);
            }
        }

        Ok(enhanced_items)
    }

    /// Enhance individual TMDB search result
    async fn enhance_tmdb_search_result(
        &self,
        result: crate::models::tmdb::TmdbSearchResult,
    ) -> Result<Option<EnhancedMediaItem>, EnhancedMetadataError> {
        let media_type = match result.media_type.as_str() {
            "movie" => MediaType::Movie,
            "tv" => MediaType::Tv,
            _ => return Ok(None),
        };

        self.get_by_tmdb_id(result.id, media_type, None).await
    }

    /// Enhance TMDB item with MDBList data
    async fn enhance_tmdb_item(
        &self,
        tmdb_item: crate::models::tmdb::TmdbItem,
    ) -> Result<EnhancedMediaItem, EnhancedMetadataError> {
        let media_type = match tmdb_item {
            crate::models::tmdb::TmdbItem::Movie(ref movie) => MediaType::Movie,
            crate::models::tmdb::TmdbItem::Show(ref show) => MediaType::Tv,
        };

        let tmdb_id = match tmdb_item {
            crate::models::tmdb::TmdbItem::Movie(ref movie) => movie.id,
            crate::models::tmdb::TmdbItem::Show(ref show) => show.id,
        };

        // Get MDBList data using IMDB ID
        let imdb_id = match tmdb_item {
            crate::models::tmdb::TmdbItem::Movie(ref movie) => movie.imdb_id.clone(),
            crate::models::tmdb::TmdbItem::Show(ref show) => show.external_ids.imdb_id.clone(),
        };

        let mdblist_data = if let Some(imdb) = &imdb_id {
            match self.mdblist_service.get_by_imdb_id(imdb).await {
                Ok(item) => Some(self.convert_mdblist_item(item)),
                Err(e) => {
                    warn!("Failed to get MDBList data for IMDB {}: {}", imdb, e);
                    None
                }
            }
        } else {
            None
        };

        // Build enhanced item
        let enhanced = match tmdb_item {
            crate::models::tmdb::TmdbItem::Movie(movie) => self.build_enhanced_movie(movie, mdblist_data),
            crate::models::tmdb::TmdbItem::Show(show) => self.build_enhanced_show(show, mdblist_data),
        };

        Ok(enhanced)
    }

    /// MDBList fallback search
    async fn search_mdblist_fallback(
        &self,
        query: &str,
        year: Option<u32>,
        media_type: Option<MediaType>,
    ) -> Result<Vec<EnhancedMediaItem>, EnhancedMetadataError> {
        let mdblist_type = match media_type {
            Some(MediaType::Movie) => MdbListMediaType::Movie,
            Some(MediaType::Tv) => MdbListMediaType::Show,
            None => MdbListMediaType::Movie, // Default
        };

        let params = MdbListSearchParams {
            query: query.to_string(),
            year,
            media_type: Some(mdblist_type),
        };

        let results = self.mdblist_service.search(params).await
            .map_err(EnhancedMetadataError::MdbListError)?;

        let mut enhanced_items = Vec::new();
        for item in results.search {
            enhanced_items.push(self.convert_mdblist_to_enhanced(item));
        }

        Ok(enhanced_items)
    }

    /// Convert MDBList item to enhanced format
    fn convert_mdblist_item(&self, item: MdbListItem) -> MdbListEnhancedData {
        MdbListEnhancedData {
            score: None,
            popularity: None,
            language: Some(item.language),
            country: Some(item.country),
            certification: Some(item.certification),
        }
    }

    /// Convert MDBList item to enhanced format
    fn convert_mdblist_to_enhanced(&self, item: MdbListItem) -> EnhancedMediaItem {
        EnhancedMediaItem {
            tmdb_id: item.tmdbid,
            imdb_id: None,
            media_type: MediaType::Movie, // Default for MDBList items
            title: item.description.clone(), // MDBList uses description as title
            original_title: item.description.clone(),
            overview: item.description.clone(),
            release_date: Some(item.released),
            runtime: Some(item.runtime),
            posters: PosterUrls {
                w500: Some(item.poster.clone()),
                w780: Some(item.poster.clone()),
                original: Some(item.poster.clone()),
            },
            backdrops: BackdropUrls {
                w780: Some(item.backdrop.clone()),
                w1280: Some(item.backdrop.clone()),
                original: Some(item.backdrop.clone()),
            },
            logos: Vec::new(),
            credits: Credits { cast: Vec::new(), crew: Vec::new() },
            ratings: Ratings {
                tmdb: item.rating.as_ref().and_then(|r| r.tmdb.map(|f| f as f64)),
                imdb: item.rating.as_ref().and_then(|r| r.imdb.map(|f| f as f64)),
                rotten_tomatoes: item.rating.as_ref().and_then(|r| r.rotten_tomatoes.map(|f| f as u32)),
                metacritic: item.rating.as_ref().and_then(|r| r.metacritic.map(|f| f as u32)),
            },
            videos: Vec::new(),
            mdblist_data: Some(MdbListEnhancedData {
                score: None,
                popularity: None,
                language: Some(item.language),
                country: Some(item.country),
                certification: Some(item.certification),
            }),
            genres: item.genres.unwrap_or_default(),
            user_data: None,
        }
    }

    /// Build enhanced movie
    fn build_enhanced_movie(
        &self,
        movie: TmdbMovie,
        mdblist_data: Option<MdbListEnhancedData>,
    ) -> EnhancedMediaItem {
        EnhancedMediaItem {
            tmdb_id: movie.id,
            imdb_id: movie.imdb_id.clone(),
            media_type: MediaType::Movie,
            title: movie.title.clone(),
            original_title: movie.original_title.clone(),
            overview: movie.overview.unwrap_or_default(),
            release_date: movie.release_date.clone(),
            runtime: movie.runtime,
            genres: movie.genres.iter().map(|g| g.name.clone()).collect(),
            posters: PosterUrls {
                w500: movie.poster_path.as_ref().map(|p| format!("https://image.tmdb.org/t/p/w500{}", p)),
                w780: movie.poster_path.as_ref().map(|p| format!("https://image.tmdb.org/t/p/w780{}", p)),
                original: movie.poster_path.as_ref().map(|p| format!("https://image.tmdb.org/t/p/original{}", p)),
            },
            backdrops: BackdropUrls {
                w780: movie.backdrop_path.as_ref().map(|p| format!("https://image.tmdb.org/t/p/w780{}", p)),
                w1280: movie.backdrop_path.as_ref().map(|p| format!("https://image.tmdb.org/t/p/w1280{}", p)),
                original: movie.backdrop_path.as_ref().map(|p| format!("https://image.tmdb.org/t/p/original{}", p)),
            },
            logos: Vec::new(), // Would need TMDB images endpoint
            credits: Credits {
                cast: movie.credits.cast.iter().take(10).map(|c| Person {
                    id: c.id,
                    name: c.name.clone(),
                    character: c.character.clone(),
                    job: None,
                    profile_path: c.profile_path.clone(),
                }).collect(),
                crew: movie.credits.crew.iter().take(5).map(|c| Person {
                    id: c.id,
                    name: c.name.clone(),
                    character: None,
                    job: Some(c.job.clone()),
                    profile_path: c.profile_path.clone(),
                }).collect(),
            },
            ratings: Ratings {
                tmdb: movie.vote_average,
                imdb: None,
                rotten_tomatoes: None,
                metacritic: None,
            },
            videos: movie.videos.results.iter().map(|v| Video {
                id: v.id.clone(),
                key: v.key.clone(),
                name: v.name.clone(),
                site: v.site.clone(),
                size: v.size,
                video_type: v.video_type.clone(),
            }).collect(),
            mdblist_data,
            user_data: None,
        }
    }

    /// Build enhanced TV show
    fn build_enhanced_show(
        &self,
        show: TmdbShow,
        mdblist_data: Option<MdbListEnhancedData>,
    ) -> EnhancedMediaItem {
        EnhancedMediaItem {
            tmdb_id: show.id,
            imdb_id: show.external_ids.imdb_id.clone(),
            media_type: MediaType::Tv,
            title: show.name.clone(),
            original_title: show.original_name.clone(),
            overview: show.overview.unwrap_or_default(),
            release_date: show.first_air_date.clone(),
            runtime: show.episode_run_time.first().copied(),
            genres: show.genres.iter().map(|g| g.name.clone()).collect(),
            posters: PosterUrls {
                w500: show.poster_path.as_ref().map(|p| format!("https://image.tmdb.org/t/p/w500{}", p)),
                w780: show.poster_path.as_ref().map(|p| format!("https://image.tmdb.org/t/p/w780{}", p)),
                original: show.poster_path.as_ref().map(|p| format!("https://image.tmdb.org/t/p/original{}", p)),
            },
            backdrops: BackdropUrls {
                w780: show.backdrop_path.as_ref().map(|p| format!("https://image.tmdb.org/t/p/w780{}", p)),
                w1280: show.backdrop_path.as_ref().map(|p| format!("https://image.tmdb.org/t/p/w1280{}", p)),
                original: show.backdrop_path.as_ref().map(|p| format!("https://image.tmdb.org/t/p/original{}", p)),
            },
            logos: Vec::new(),
            credits: Credits {
                cast: show.credits.cast.iter().take(10).map(|c| Person {
                    id: c.id,
                    name: c.name.clone(),
                    character: c.character.clone(),
                    job: None,
                    profile_path: c.profile_path.clone(),
                }).collect(),
                crew: show.credits.crew.iter().take(5).map(|c| Person {
                    id: c.id,
                    name: c.name.clone(),
                    character: None,
                    job: Some(c.job.clone()),
                    profile_path: c.profile_path.clone(),
                }).collect(),
            },
            ratings: Ratings {
                tmdb: show.vote_average,
                imdb: None,
                rotten_tomatoes: None,
                metacritic: None,
            },
            videos: show.videos.results.iter().map(|v| Video {
                id: v.id.clone(),
                key: v.key.clone(),
                name: v.name.clone(),
                site: v.site.clone(),
                size: v.size,
                video_type: v.video_type.clone(),
            }).collect(),
            mdblist_data,
            user_data: None,
        }
    }

    /// Apply user-specific data to enhanced items
    async fn apply_user_data(
        &self,
        items: Vec<EnhancedMediaItem>,
        user_id: Option<&str>,
    ) -> Vec<EnhancedMediaItem> {
        if let Some(user_id) = user_id {
            // In a real implementation, this would fetch user-specific data
            // For now, we'll return items as-is
        }
        items
    }

    /// Apply user-specific data to single item
    async fn apply_user_data_single(
        &self,
        mut item: EnhancedMediaItem,
        user_id: Option<&str>,
    ) -> EnhancedMediaItem {
        if let Some(user_id) = user_id {
            // In a real implementation, this would fetch user-specific data
            // For now, we'll return the item as-is
        }
        item
    }
}

/// Enhanced metadata service error
#[derive(Debug, thiserror::Error)]
pub enum EnhancedMetadataError {
    #[error("TMDB service error: {0}")]
    TmdbError(#[from] crate::services::tmdb::TmdbError),
    #[error("MDBList service error: {0}")]
    MdbListError(#[from] crate::services::mdblist::MdbListError),
    #[error("Cache error: {0}")]
    CacheError(String),
}

/// Cache key utilities for enhanced metadata
pub mod CacheKeys {
    use super::MediaType;

    pub fn enhanced_search(query: &str, year: Option<u32>, media_type: Option<MediaType>) -> String {
        let type_str = match media_type {
            Some(MediaType::Movie) => "movie",
            Some(MediaType::Tv) => "tv",
            None => "all",
        };
        format!("enhanced:search:{}:{}:{}", query, year.map(|y| y.to_string()).unwrap_or_else(|| "none".to_string()), type_str)
    }

    pub fn enhanced_by_tmdb_id(tmdb_id: u64, media_type: MediaType) -> String {
        let type_str = match media_type {
            MediaType::Movie => "movie",
            MediaType::Tv => "tv",
        };
        format!("enhanced:tmdb:{}:{}", tmdb_id, type_str)
    }
}