use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Movie {
    pub id: u32,
    pub title: String,
    pub overview: Option<String>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub release_date: Option<String>,
    pub vote_average: f64,
    pub vote_count: u32,
    pub popularity: f64,
    pub adult: bool,
    pub video: bool,
    pub original_language: String,
    pub original_title: String,
    pub genre_ids: Vec<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MovieResponse {
    pub page: u32,
    pub results: Vec<Movie>,
    pub total_pages: u32,
    pub total_results: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub page: u32,
    pub results: Vec<Movie>,
    pub total_pages: u32,
    pub total_results: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovieDetails {
    pub id: u32,
    pub title: String,
    pub overview: Option<String>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub release_date: Option<String>,
    pub vote_average: f64,
    pub vote_count: u32,
    pub popularity: f64,
    pub adult: bool,
    pub video: bool,
    pub original_language: String,
    pub original_title: String,
    pub genres: Vec<Genre>,
    pub runtime: Option<u32>,
    pub budget: u64,
    pub revenue: u64,
    pub status: String,
    pub tagline: Option<String>,
    pub homepage: Option<String>,
    pub imdb_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Genre {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionCompany {
    pub id: u32,
    pub name: String,
    pub logo_path: Option<String>,
    pub origin_country: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionCountry {
    pub iso_3166_1: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpokenLanguage {
    pub english_name: String,
    pub iso_639_1: String,
    pub name: String,
}

// Enhanced movie data combining TMDB with additional metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedMovie {
    #[serde(flatten)]
    pub movie: Movie,
    pub image_urls: ImageUrls,
    pub streaming_info: Option<StreamingInfo>,
    pub cached_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrls {
    pub poster_small: Option<String>,
    pub poster_medium: Option<String>,
    pub poster_large: Option<String>,
    pub backdrop_small: Option<String>,
    pub backdrop_medium: Option<String>,
    pub backdrop_large: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingInfo {
    pub available_streams: Vec<StreamSource>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamSource {
    pub name: String,
    pub url: String,
    pub quality: String,
    pub source_type: String, // "torrent", "direct", "addon"
    pub seeds: Option<u32>,
    pub peers: Option<u32>,
    pub size: Option<String>,
}

impl Movie {
    pub fn get_year(&self) -> Option<u32> {
        self.release_date
            .as_ref()
            .and_then(|date| date.split('-').next())
            .and_then(|year| year.parse().ok())
    }
    
    pub fn has_poster(&self) -> bool {
        self.poster_path.is_some()
    }
    
    pub fn has_backdrop(&self) -> bool {
        self.backdrop_path.is_some()
    }
}

impl EnhancedMovie {
    pub fn from_movie(movie: Movie) -> Self {
        Self {
            image_urls: ImageUrls {
                poster_small: None,
                poster_medium: None,
                poster_large: None,
                backdrop_small: None,
                backdrop_medium: None,
                backdrop_large: None,
            },
            streaming_info: None,
            cached_at: Utc::now(),
            movie,
        }
    }
    
    pub fn with_image_urls(mut self, base_url: &str) -> Self {
        if let Some(poster_path) = &self.movie.poster_path {
            self.image_urls.poster_small = Some(format!("{}w185{}", base_url, poster_path));
            self.image_urls.poster_medium = Some(format!("{}w342{}", base_url, poster_path));
            self.image_urls.poster_large = Some(format!("{}w500{}", base_url, poster_path));
        }
        
        if let Some(backdrop_path) = &self.movie.backdrop_path {
            self.image_urls.backdrop_small = Some(format!("{}w300{}", base_url, backdrop_path));
            self.image_urls.backdrop_medium = Some(format!("{}w780{}", base_url, backdrop_path));
            self.image_urls.backdrop_large = Some(format!("{}w1280{}", base_url, backdrop_path));
        }
        
        self
    }
}