//! Stremio addon handlers for MDBList integration
//!
//! Provides personalized Stremio catalogs based on MDBList user data
//! including watchlists, custom lists, and trending content

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    error::AppResult,
    models::stremio::{
        StremioCatalog,
        StremioManifest,
        StremioMeta,
        StremioMetaItem,
        StremioStream,
        StremioStreamItem,
    },
    services::{
        enhanced_metadata::{EnhancedMetadataService, MediaType},
        stremio::StremioService,
    },
    AppState,
};

/// Query parameters for Stremio catalog requests
#[derive(Debug, Deserialize)]
pub struct CatalogQuery {
    pub user_id: Option<String>,
    pub list_id: Option<String>,
    pub page: Option<u32>,
}

/// Stremio addon manifest for MDBList integration
pub async fn mdblist_manifest(
    State(state): State<AppState>,
) -> AppResult<Json<StremioManifest>> {
    let manifest = StremioManifest {
        id: "crmb.mdblist".to_string(),
        version: "1.0.0".to_string(),
        name: "CRMB MDBList".to_string(),
        description: "Personalized streaming catalogs from your MDBLists".to_string(),
        resources: vec![
            "catalog".to_string(),
            "meta".to_string(),
        ],
        types: vec![
            "movie".to_string(),
            "series".to_string(),
        ],
        catalogs: vec![
            StremioCatalog {
                id: "mdblist.watchlist".to_string(),
                name: "My MDBList Watchlist".to_string(),
                r#type: vec!["movie".to_string(), "series".to_string()],
                extra: vec![
                    StremioCatalogExtra {
                        name: "user_id".to_string(),
                        is_required: true,
                        options: vec![],
                    },
                ],
            },
            StremioCatalog {
                id: "mdblist.custom".to_string(),
                name: "My MDBList Lists".to_string(),
                r#type: vec!["movie".to_string(), "series".to_string()],
                extra: vec![
                    StremioCatalogExtra {
                        name: "user_id".to_string(),
                        is_required: true,
                        options: vec![],
                    },
                    StremioCatalogExtra {
                        name: "list_id".to_string(),
                        is_required: false,
                        options: vec![],
                    },
                ],
            },
            StremioCatalog {
                id: "mdblist.trending".to_string(),
                name: "MDBList Trending".to_string(),
                r#type: vec!["movie".to_string(), "series".to_string()],
                extra: vec![],
            },
        ],
        id_prefixes: vec!["tt".to_string()],
        logo: "https://raw.githubusercontent.com/fatshotty/mdblist-lib/main/assets/logo.png".to_string(),
        contact_email: "support@crmb-streaming.com".to_string(),
    };

    Ok(Json(manifest))
}

/// Stremio catalog endpoint for MDBList content
pub async fn mdblist_catalog(
    State(state): State<AppState>,
    Path((catalog_type, catalog_id)): Path<(String, String)>,
    Query(query): Query<CatalogQuery>,
) -> AppResult<Json<StremioMeta>> {
    let enhanced_service = EnhancedMetadataService::new(
        state.services.tmdb.clone(),
        state.services.mdblist.clone(),
        state.services.cache.clone(),
    );

    let user_id = query.user_id.clone();
    let page = query.page.unwrap_or(1);
    
    let items = match catalog_id.as_str() {
        "mdblist.watchlist" => {
            if let Some(user_id) = user_id {
                get_user_watchlist_catalog(&enhanced_service, &user_id, page).await?
            } else {
                return Err(crate::error::AppError::BadRequest(
                    "user_id parameter required for watchlist".to_string(),
                ));
            }
        },
        "mdblist.custom" => {
            if let Some(user_id) = user_id {
                let list_id = query.list_id.clone();
                get_custom_list_catalog(&enhanced_service, &user_id, list_id.as_deref(), page).await?
            } else {
                return Err(crate::error::AppError::BadRequest(
                    "user_id parameter required for custom lists".to_string(),
                ));
            }
        },
        "mdblist.trending" => {
            get_trending_catalog(&enhanced_service, page).await?
        },
        _ => {
            return Err(crate::error::AppError::NotFound(
                "Catalog not found".to_string(),
            ));
        },
    };

    let meta = StremioMeta {
        metas: items,
    };

    Ok(Json(meta))
}

/// Get user's MDBList watchlist as Stremio items
async fn get_user_watchlist_catalog(
    enhanced_service: &EnhancedMetadataService,
    user_id: &str,
    page: u32,
) -> AppResult<Vec<StremioMetaItem>> {
    let user_lists = enhanced_service.mdblist_service.get_user_lists(user_id).await
        .map_err(|e| crate::error::AppError::Internal(format!("Failed to get user lists: {}", e)))?;

    let mut all_items = Vec::new();

    // Find the user's watchlist
    if let Some(watchlist) = user_lists.lists.iter().find(|list| list.name.to_lowercase().contains("watchlist")) {
        let list_content = enhanced_service.mdblist_service.get_list_content(&watchlist.id).await
            .map_err(|e| crate::error::AppError::Internal(format!("Failed to get watchlist content: {}", e)))?;

        for item in list_content.items {
            if let Some(tmdb_id) = item.tmdb_id {
                let enhanced_item = enhanced_service
                    .get_by_tmdb_id(
                        tmdb_id,
                        match item.media_type.as_str() {
                            "movie" => MediaType::Movie,
                            "show" => MediaType::Tv,
                            _ => MediaType::Movie,
                        },
                        Some(user_id.to_string()),
                    )
                    .await
                    .unwrap_or_else(|_| None);

                if let Some(enhanced) = enhanced_item {
                    all_items.push(convert_to_stremio_item(enhanced));
                }
            }
        }
    }

    // Paginate results
    let start = ((page - 1) * 20) as usize;
    let end = (start + 20).min(all_items.len());
    
    Ok(all_items[start..end].to_vec())
}

/// Get user's custom MDBList as Stremio items
async fn get_custom_list_catalog(
    enhanced_service: &EnhancedMetadataService,
    user_id: &str,
    list_id: Option<&str>,
    page: u32,
) -> AppResult<Vec<StremioMetaItem>> {
    let user_lists = enhanced_service.mdblist_service.get_user_lists(user_id).await
        .map_err(|e| crate::error::AppError::Internal(format!("Failed to get user lists: {}", e)))?;

    let mut all_items = Vec::new();

    let lists_to_process = if let Some(list_id) = list_id {
        // Get specific list
        vec![list_id.to_string()]
    } else {
        // Get all custom lists
        user_lists.lists.iter()
            .filter(|list| !list.name.to_lowercase().contains("watchlist"))
            .map(|list| list.id.clone())
            .collect()
    };

    for list_id in lists_to_process {
        let list_content = enhanced_service.mdblist_service.get_list_content(&list_id).await
            .map_err(|e| crate::error::AppError::Internal(format!("Failed to get list content: {}", e)))?;

        for item in list_content.items {
            if let Some(tmdb_id) = item.tmdb_id {
                let enhanced_item = enhanced_service
                    .get_by_tmdb_id(
                        tmdb_id,
                        match item.media_type.as_str() {
                            "movie" => MediaType::Movie,
                            "show" => MediaType::Tv,
                            _ => MediaType::Movie,
                        },
                        Some(user_id.to_string()),
                    )
                    .await
                    .unwrap_or_else(|_| None);

                if let Some(enhanced) = enhanced_item {
                    all_items.push(convert_to_stremio_item(enhanced));
                }
            }
        }
    }

    // Paginate results
    let start = ((page - 1) * 20) as usize;
    let end = (start + 20).min(all_items.len());
    
    Ok(all_items[start..end].to_vec())
}

/// Get MDBList trending content as Stremio items
async fn get_trending_catalog(
    enhanced_service: &EnhancedMetadataService,
    page: u32,
) -> AppResult<Vec<StremioMetaItem>> {
    let trending = enhanced_service.mdblist_service.get_trending(None).await
        .map_err(|e| crate::error::AppError::Internal(format!("Failed to get trending: {}", e)))?;

    let mut items = Vec::new();

    for item in trending.trending {
        if let Some(tmdb_id) = item.tmdb_id {
            let enhanced_item = enhanced_service
                .get_by_tmdb_id(
                    tmdb_id,
                    match item.media_type.as_str() {
                        "movie" => MediaType::Movie,
                        "show" => MediaType::Tv,
                        _ => MediaType::Movie,
                    },
                    None,
                )
                .await
                .unwrap_or_else(|_| None);

            if let Some(enhanced) = enhanced_item {
                items.push(convert_to_stremio_item(enhanced));
            }
        }
    }

    // Paginate results
    let start = ((page - 1) * 20) as usize;
    let end = (start + 20).min(items.len());
    
    Ok(items[start..end].to_vec())
}

/// Convert EnhancedMediaItem to StremioMetaItem
fn convert_to_stremio_item(enhanced: crate::services::enhanced_metadata::EnhancedMediaItem) -> StremioMetaItem {
    let (id, r#type) = match enhanced.media_type {
        crate::services::enhanced_metadata::MediaType::Movie => (format!("tt{}" , enhanced.imdb_id.unwrap_or_default()), "movie".to_string()),
        crate::services::enhanced_metadata::MediaType::Tv => (format!("tt{}" , enhanced.imdb_id.unwrap_or_default()), "series".to_string()),
    };

    StremioMetaItem {
        id,
        r#type,
        name: enhanced.title,
        poster: enhanced.posters.w500,
        background: enhanced.backdrops.w780,
        logo: enhanced.posters.original.clone(),
        description: enhanced.overview,
        release_info: enhanced.release_date.map(|d| d.to_string()).unwrap_or_default(),
        imdb_rating: enhanced.ratings.imdb,
        genres: enhanced.genres,
        runtime: enhanced.runtime.map(|r| r.to_string()),
        videos: enhanced.videos.into_iter().map(|v| StremioStream {
            id: v.key,
            title: v.name,
            thumbnail: v.thumbnail,
            url: v.url,
        }).collect(),
    }
}

/// Stremio addon configuration
#[derive(Debug, Serialize)]
pub struct StremioConfig {
    pub addon_name: String,
    pub addon_id: String,
    pub addon_version: String,
    pub base_url: String,
    pub catalogs: Vec<String>,
    pub features: Vec<String>,
}

/// Get Stremio addon configuration
pub async fn get_stremio_config(
    State(state): State<AppState>,
) -> AppResult<Json<StremioConfig>> {
    let config = StremioConfig {
        addon_name: "CRMB MDBList".to_string(),
        addon_id: "crmb.mdblist".to_string(),
        addon_version: "1.0.0".to_string(),
        base_url: "http://localhost:8080".to_string(),
        catalogs: vec![
            "mdblist.watchlist".to_string(),
            "mdblist.custom".to_string(),
            "mdblist.trending".to_string(),
        ],
        features: vec![
            "Personal watchlists".to_string(),
            "Custom MDBList integration".to_string(),
            "Trending content".to_string(),
            "High-quality metadata".to_string(),
            "Real-time updates".to_string(),
        ],
    };

    Ok(Json(config))
}

/// Stremio catalog extra parameters
#[derive(Debug, Serialize)]
pub struct StremioCatalogExtra {
    pub name: String,
    pub is_required: bool,
    pub options: Vec<String>,
}