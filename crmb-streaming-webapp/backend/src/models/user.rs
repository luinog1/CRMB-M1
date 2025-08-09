use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub preferences: Option<String>, // JSON string
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UserCreate {
    #[validate(length(min = 3, max = 50, message = "Username must be between 3 and 50 characters"))]
    #[validate(regex(
        path = "USERNAME_REGEX",
        message = "Username can only contain letters, numbers, and underscores"
    ))]
    pub username: String,
    
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    
    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    pub password: String,
    
    #[serde(skip_deserializing)]
    pub password_hash: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UserLogin {
    #[validate(length(min = 1, message = "Email or username is required"))]
    pub email_or_username: String,
    
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UserUpdate {
    #[validate(length(min = 3, max = 50, message = "Username must be between 3 and 50 characters"))]
    #[validate(regex(
        path = "USERNAME_REGEX",
        message = "Username can only contain letters, numbers, and underscores"
    ))]
    pub username: Option<String>,
    
    #[validate(email(message = "Invalid email format"))]
    pub email: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: Option<String>,
    
    #[validate(length(min = 3, max = 50, message = "Username must be between 3 and 50 characters"))]
    #[validate(regex(
        path = "USERNAME_REGEX",
        message = "Username can only contain letters, numbers, and underscores"
    ))]
    pub username: Option<String>,
    
    #[validate(length(max = 100, message = "Full name cannot exceed 100 characters"))]
    pub full_name: Option<String>,
    
    pub avatar_url: Option<String>,
    pub preferences: Option<UserPreferences>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteAccountRequest {
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserPreferences {
    pub theme: Option<String>,
    pub language: Option<String>,
    pub auto_play: Option<bool>,
    pub quality: Option<String>,
    pub subtitles: Option<bool>,
    pub notifications: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct UserProfile {
    pub id: String,
    pub username: String,
    pub email: String,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub preferences: Option<UserPreferences>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub watchlist_count: usize,
}

impl From<User> for UserProfile {
    fn from(user: User) -> Self {
        let preferences = user.preferences
            .as_ref()
            .and_then(|p| serde_json::from_str(p).ok())
            .unwrap_or_default();
            
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            full_name: user.full_name,
            avatar_url: user.avatar_url,
            preferences: Some(preferences),
            created_at: user.created_at,
            updated_at: user.updated_at,
            watchlist_count: 0, // Will be populated separately
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchlistItem {
    pub id: Option<String>,
    pub tmdb_id: i32,
    pub media_type: String, // "movie" or "tv"
    pub title: String,
    pub poster_path: Option<String>,
    pub overview: Option<String>,
    pub release_date: Option<String>,
    pub vote_average: Option<f64>,
    pub added_at: Option<DateTime<Utc>>,
    pub watched: bool,
    pub watch_progress: f64, // 0.0 to 1.0
}

#[derive(Debug, Deserialize, Validate)]
pub struct AddToWatchlistRequest {
    pub tmdb_id: i32,
    
    #[validate(regex(
        path = "MEDIA_TYPE_REGEX",
        message = "Media type must be 'movie' or 'tv'"
    ))]
    pub media_type: String,
    
    #[validate(length(min = 1, max = 500, message = "Title must be between 1 and 500 characters"))]
    pub title: String,
    
    pub poster_path: Option<String>,
    pub overview: Option<String>,
    pub release_date: Option<String>,
    pub vote_average: Option<f64>,
}

impl From<AddToWatchlistRequest> for WatchlistItem {
    fn from(req: AddToWatchlistRequest) -> Self {
        Self {
            id: None,
            tmdb_id: req.tmdb_id,
            media_type: req.media_type,
            title: req.title,
            poster_path: req.poster_path,
            overview: req.overview,
            release_date: req.release_date,
            vote_average: req.vote_average,
            added_at: None,
            watched: false,
            watch_progress: 0.0,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateWatchProgressRequest {
    pub progress: f64, // 0.0 to 1.0
    pub watched: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct WatchlistStats {
    pub total_items: usize,
    pub movies: usize,
    pub tv_shows: usize,
    pub watched: usize,
    pub in_progress: usize,
    pub average_rating: f64,
}

// Regex patterns for validation
lazy_static::lazy_static! {
    static ref USERNAME_REGEX: regex::Regex = regex::Regex::new(r"^[a-zA-Z0-9_]+$").unwrap();
    static ref MEDIA_TYPE_REGEX: regex::Regex = regex::Regex::new(r"^(movie|tv)$").unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn test_user_create_validation() {
        let valid_user = UserCreate {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            password_hash: String::new(),
        };
        assert!(valid_user.validate().is_ok());

        let invalid_user = UserCreate {
            username: "ab".to_string(), // Too short
            email: "invalid-email".to_string(), // Invalid email
            password: "123".to_string(), // Too short
            password_hash: String::new(),
        };
        assert!(invalid_user.validate().is_err());
    }

    #[test]
    fn test_watchlist_item_from_request() {
        let request = AddToWatchlistRequest {
            tmdb_id: 123,
            media_type: "movie".to_string(),
            title: "Test Movie".to_string(),
            poster_path: Some("/poster.jpg".to_string()),
            overview: Some("A test movie".to_string()),
            release_date: Some("2023-01-01".to_string()),
            vote_average: Some(8.5),
        };

        let item: WatchlistItem = request.into();
        assert_eq!(item.tmdb_id, 123);
        assert_eq!(item.media_type, "movie");
        assert_eq!(item.title, "Test Movie");
        assert!(!item.watched);
        assert_eq!(item.watch_progress, 0.0);
    }
}