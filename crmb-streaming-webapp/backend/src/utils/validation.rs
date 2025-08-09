//! Input validation utilities

use regex::Regex;
use std::sync::OnceLock;

/// Get the username validation regex
pub fn username_regex() -> &'static Regex {
    static USERNAME_REGEX: OnceLock<Regex> = OnceLock::new();
    USERNAME_REGEX.get_or_init(|| {
        Regex::new(r"^[a-zA-Z0-9_]{3,20}$").unwrap()
    })
}

/// Get the media type validation regex
pub fn media_type_regex() -> &'static Regex {
    static MEDIA_TYPE_REGEX: OnceLock<Regex> = OnceLock::new();
    MEDIA_TYPE_REGEX.get_or_init(|| {
        Regex::new(r"^(movie|tv)$").unwrap()
    })
}

/// Validate username format
pub fn validate_username(username: &str) -> bool {
    username_regex().is_match(username)
}

/// Validate media type
pub fn validate_media_type(media_type: &str) -> bool {
    media_type_regex().is_match(media_type)
}

/// Validate email format
pub fn validate_email(email: &str) -> bool {
    email.contains('@') && email.len() > 5 && email.len() < 255
}

/// Validate password strength
pub fn validate_password(password: &str) -> bool {
    password.len() >= 8 && password.len() <= 128
}