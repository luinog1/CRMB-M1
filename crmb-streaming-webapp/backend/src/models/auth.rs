use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String, // User ID
    pub username: String,
    pub email: String,
    pub exp: i64, // Expiration timestamp
    pub iat: i64, // Issued at timestamp
    pub jti: String, // JWT ID for token tracking
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserInfo,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 50, message = "Username must be between 3 and 50 characters"))]
    #[validate(regex(
        path = "USERNAME_REGEX",
        message = "Username can only contain letters, numbers, and underscores"
    ))]
    pub username: String,
    
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    
    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    #[validate(custom(function = "validate_password_strength", message = "Password must contain at least one uppercase letter, one lowercase letter, and one number"))]
    pub password: String,
    
    #[validate(must_match(other = "password", message = "Passwords do not match"))]
    pub confirm_password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 1, message = "Email or username is required"))]
    pub email_or_username: String,
    
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
    
    pub remember_me: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct RefreshTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LogoutResponse {
    pub message: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ChangePasswordRequest {
    #[validate(length(min = 1, message = "Current password is required"))]
    pub current_password: String,
    
    #[validate(length(min = 8, message = "New password must be at least 8 characters long"))]
    #[validate(custom(function = "validate_password_strength", message = "New password must contain at least one uppercase letter, one lowercase letter, and one number"))]
    pub new_password: String,
    
    #[validate(must_match(other = "new_password", message = "Password confirmation does not match"))]
    pub confirm_new_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenClaims {
    pub sub: String, // User ID
    pub exp: i64, // Expiration timestamp
    pub iat: i64, // Issued at timestamp
    pub jti: String, // JWT ID for token tracking
    pub token_type: String, // "refresh"
    pub iss: Option<String>, // Issuer
}

#[derive(Debug, Serialize)]
pub struct TokenValidationResponse {
    pub valid: bool,
    pub user_id: Option<String>,
    pub username: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    #[error("User not found")]
    UserNotFound,
    
    #[error("User already exists")]
    UserAlreadyExists,
    
    #[error("Invalid token")]
    InvalidToken,
    
    #[error("Token expired")]
    TokenExpired,
    
    #[error("Token revoked")]
    TokenRevoked,
    
    #[error("Insufficient permissions")]
    InsufficientPermissions,
    
    #[error("Account disabled")]
    AccountDisabled,
    
    #[error("Password hash error: {0}")]
    PasswordHashError(String),
    
    #[error("JWT error: {0}")]
    JwtError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Weak password: {0}")]
    WeakPassword(String),
    
    #[error("Token generation failed")]
    TokenGenerationFailed,
    
    #[error("Password hashing failed")]
    PasswordHashingFailed,
    
    #[error("Password verification failed")]
    PasswordVerificationFailed,
}

impl From<bcrypt::BcryptError> for AuthError {
    fn from(err: bcrypt::BcryptError) -> Self {
        AuthError::PasswordHashError(err.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for AuthError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        match err.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
            jsonwebtoken::errors::ErrorKind::InvalidToken => AuthError::InvalidToken,
            _ => AuthError::JwtError(err.to_string()),
        }
    }
}

impl From<sqlx::Error> for AuthError {
    fn from(err: sqlx::Error) -> Self {
        AuthError::DatabaseError(err.to_string())
    }
}

impl From<validator::ValidationErrors> for AuthError {
    fn from(err: validator::ValidationErrors) -> Self {
        let messages: Vec<String> = err
            .field_errors()
            .iter()
            .flat_map(|(_, errors)| {
                errors.iter().map(|e| {
                    e.message
                        .as_ref()
                        .map(|m| m.to_string())
                        .unwrap_or_else(|| "Validation error".to_string())
                })
            })
            .collect();
        AuthError::ValidationError(messages.join(", "))
    }
}

// Custom validation functions
fn validate_password_strength(password: &str) -> Result<(), validator::ValidationError> {
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    
    if has_uppercase && has_lowercase && has_digit {
        Ok(())
    } else {
        Err(validator::ValidationError::new("password_strength"))
    }
}

// Regex patterns for validation
lazy_static::lazy_static! {
    static ref USERNAME_REGEX: regex::Regex = regex::Regex::new(r"^[a-zA-Z0-9_]+$").unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn test_register_request_validation() {
        let valid_request = RegisterRequest {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "Password123".to_string(),
            confirm_password: "Password123".to_string(),
        };
        assert!(valid_request.validate().is_ok());

        let invalid_request = RegisterRequest {
            username: "ab".to_string(), // Too short
            email: "invalid-email".to_string(), // Invalid email
            password: "weak".to_string(), // Too short and weak
            confirm_password: "different".to_string(), // Doesn't match
        };
        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_password_strength_validation() {
        assert!(validate_password_strength("Password123").is_ok());
        assert!(validate_password_strength("password123").is_err()); // No uppercase
        assert!(validate_password_strength("PASSWORD123").is_err()); // No lowercase
        assert!(validate_password_strength("Password").is_err()); // No digit
    }

    #[test]
    fn test_login_request_validation() {
        let valid_request = LoginRequest {
            email_or_username: "test@example.com".to_string(),
            password: "password".to_string(),
            remember_me: Some(true),
        };
        assert!(valid_request.validate().is_ok());

        let invalid_request = LoginRequest {
            email_or_username: "".to_string(), // Empty
            password: "".to_string(), // Empty
            remember_me: None,
        };
        assert!(invalid_request.validate().is_err());
    }
}