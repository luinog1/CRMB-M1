//! Authentication service with JWT token management
//!
//! This module provides comprehensive authentication services including:
//! - JWT token generation and validation
//! - Password hashing and verification
//! - Session management
//! - Token refresh logic
//! - Security utilities

use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::database::Database;
use crate::models::auth::*;
use crate::models::user::User;

/// Authentication service
#[derive(Clone)]
pub struct AuthService {
    /// Database connection
    database: Arc<Database>,
    /// JWT configuration
    jwt_config: JwtConfig,
    /// Service metrics
    metrics: Arc<RwLock<AuthMetrics>>,
    /// Active sessions (in-memory store)
    active_sessions: Arc<RwLock<std::collections::HashMap<String, SessionInfo>>>,
}

/// JWT configuration
#[derive(Debug, Clone)]
pub struct JwtConfig {
    /// Secret key for signing tokens
    pub secret: String,
    /// Access token expiration time
    pub access_token_expiry: Duration,
    /// Refresh token expiration time
    pub refresh_token_expiry: Duration,
    /// Token issuer
    pub issuer: String,
    /// Token audience
    pub audience: String,
    /// Algorithm for signing
    pub algorithm: Algorithm,
}

/// Session information
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SessionInfo {
    user_id: u32,
    username: String,
    created_at: SystemTime,
    last_accessed: SystemTime,
    ip_address: Option<String>,
    user_agent: Option<String>,
}

/// Authentication metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthMetrics {
    pub total_logins: u64,
    pub successful_logins: u64,
    pub failed_logins: u64,
    pub token_validations: u64,
    pub token_refreshes: u64,
    pub active_sessions: u64,
    pub password_resets: u64,
    pub account_lockouts: u64,
}

/// Password strength requirements
#[derive(Debug, Clone)]
pub struct PasswordPolicy {
    pub min_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_special_chars: bool,
    pub max_length: usize,
}

/// Authentication result
#[derive(Debug, Clone)]
pub struct AuthResult {
    pub user: User,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
}

/// Token validation result
#[derive(Debug, Clone)]
pub struct TokenValidation {
    pub valid: bool,
    pub user_id: Option<u32>,
    pub username: Option<String>,
    pub expires_at: Option<SystemTime>,
    pub error: Option<String>,
}

impl AuthService {
    /// Create a new authentication service from AppConfig
    pub fn new(database: Arc<Database>, config: &crate::config::AppConfig) -> Self {
        let metrics = Arc::new(RwLock::new(AuthMetrics::default()));
        let active_sessions = Arc::new(RwLock::new(std::collections::HashMap::new()));

        let jwt_config = JwtConfig {
            secret: config.jwt_secret.clone(),
            access_token_expiry: Duration::from_secs(config.jwt_expiration as u64),
            refresh_token_expiry: Duration::from_secs(config.jwt_expiration as u64 * 7), // 7x longer than access token
            issuer: "crmb-streaming-webapp".to_string(),
            audience: "crmb-users".to_string(),
            algorithm: Algorithm::HS256,
        };

        Self {
            database,
            jwt_config,
            metrics,
            active_sessions,
        }
    }

    /// Register a new user
    pub async fn register(&self, request: RegisterRequest) -> Result<AuthResult, AuthError> {
        // Validate input
        self.validate_registration(&request).await?;

        // Check if user already exists
        if self.database.get_user_by_email(&request.email).await.is_ok() {
            return Err(AuthError::UserAlreadyExists);
        }

        if self.database.get_user_by_username(&request.username).await.is_ok() {
            return Err(AuthError::UserAlreadyExists);
        }

        // Hash password
        let password_hash = self.hash_password(&request.password)?;

        // Create user
        let user = self.database.create_user(
            &request.username,
            &request.email,
            &password_hash,
        ).await.map_err(AuthError::DatabaseError)?;

        // Generate tokens
        let (access_token, refresh_token) = self.generate_tokens(&user).await?;

        // Create session
        self.create_session(&user, None, None).await?;

        self.update_metrics(|m| {
            m.total_logins += 1;
            m.successful_logins += 1;
        }).await;

        Ok(AuthResult {
            user,
            access_token,
            refresh_token,
            expires_in: self.jwt_config.access_token_expiry.as_secs(),
        })
    }

    /// Authenticate user login
    pub async fn login(&self, request: LoginRequest) -> Result<AuthResult, AuthError> {
        self.update_metrics(|m| m.total_logins += 1).await;

        // Get user by email or username
        let user = if request.email_or_username.contains('@') {
            self.database.get_user_by_email(&request.email_or_username).await
        } else {
            self.database.get_user_by_username(&request.email_or_username).await
        }.map_err(|_| AuthError::InvalidCredentials)?;

        // Verify password
        if !self.verify_password(&request.password, &user.password_hash)? {
            self.update_metrics(|m| m.failed_logins += 1).await;
            return Err(AuthError::InvalidCredentials);
        }

        // Generate tokens
        let (access_token, refresh_token) = self.generate_tokens(&user).await?;

        // Create session
        self.create_session(&user, request.ip_address, request.user_agent).await?;

        self.update_metrics(|m| m.successful_logins += 1).await;

        Ok(AuthResult {
            user,
            access_token,
            refresh_token,
            expires_in: self.jwt_config.access_token_expiry.as_secs(),
        })
    }

    /// Refresh access token
    pub async fn refresh_token(&self, request: RefreshTokenRequest) -> Result<RefreshTokenResponse, AuthError> {
        self.update_metrics(|m| m.token_refreshes += 1).await;

        // Validate refresh token
        let claims = self.validate_refresh_token(&request.refresh_token)?;

        // Get user
        let user = self.database.get_user_by_id(claims.user_id).await
            .map_err(|_| AuthError::InvalidToken)?;

        // Generate new access token
        let access_token = self.generate_access_token(&user)?;

        Ok(RefreshTokenResponse {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in: self.jwt_config.access_token_expiry.as_secs(),
        })
    }

    /// Logout user
    pub async fn logout(&self, request: LogoutRequest) -> Result<LogoutResponse, AuthError> {
        // Validate token to get user info
        let validation = self.validate_token(&request.token).await;
        
        if let Some(user_id) = validation.user_id {
            // Remove session from active sessions
            let mut sessions = self.active_sessions.write().await;
            sessions.retain(|_, session| session.user_id != user_id);
            
            // Invalidate session in database
            let _ = self.database.invalidate_user_sessions(user_id).await;
            
            self.update_metrics(|m| m.active_sessions = sessions.len() as u64).await;
        }

        Ok(LogoutResponse {
            message: "Successfully logged out".to_string(),
        })
    }

    /// Validate access token
    pub async fn validate_token(&self, token: &str) -> TokenValidation {
        self.update_metrics(|m| m.token_validations += 1).await;

        match self.decode_access_token(token) {
            Ok(claims) => {
                // Check if session is still active
                let sessions = self.active_sessions.read().await;
                let session_active = sessions.values()
                    .any(|session| session.user_id == claims.user_id);

                if session_active {
                    TokenValidation {
                        valid: true,
                        user_id: Some(claims.user_id),
                        username: Some(claims.username),
                        expires_at: Some(UNIX_EPOCH + Duration::from_secs(claims.exp)),
                        error: None,
                    }
                } else {
                    TokenValidation {
                        valid: false,
                        user_id: None,
                        username: None,
                        expires_at: None,
                        error: Some("Session not active".to_string()),
                    }
                }
            }
            Err(e) => TokenValidation {
                valid: false,
                user_id: None,
                username: None,
                expires_at: None,
                error: Some(e.to_string()),
            },
        }
    }

    /// Get user from token
    pub async fn get_user_from_token(&self, token: &str) -> Result<User, AuthError> {
        let claims = self.decode_access_token(token)?;
        self.database.get_user_by_id(claims.user_id).await
            .map_err(AuthError::DatabaseError)
    }

    /// Change user password
    pub async fn change_password(
        &self,
        user_id: u32,
        current_password: &str,
        new_password: &str,
    ) -> Result<(), AuthError> {
        // Get user
        let user = self.database.get_user_by_id(user_id).await
            .map_err(AuthError::DatabaseError)?;

        // Verify current password
        if !self.verify_password(current_password, &user.password_hash)? {
            return Err(AuthError::InvalidCredentials);
        }

        // Validate new password
        self.validate_password(new_password)?;

        // Hash new password
        let new_password_hash = self.hash_password(new_password)?;

        // Update password in database
        // TODO: Implement update_user_password in database
        // self.database.update_user_password(user_id, &new_password_hash).await
        //     .map_err(AuthError::DatabaseError)?;

        // Invalidate all user sessions
        let _ = self.database.invalidate_user_sessions(user_id).await;
        
        // Remove from active sessions
        let mut sessions = self.active_sessions.write().await;
        sessions.retain(|_, session| session.user_id != user_id);

        Ok(())
    }

    /// Get authentication metrics
    pub async fn get_metrics(&self) -> AuthMetrics {
        self.metrics.read().await.clone()
    }

    /// Get active sessions count
    pub async fn get_active_sessions_count(&self) -> usize {
        self.active_sessions.read().await.len()
    }

    /// Cleanup expired sessions
    pub async fn cleanup_expired_sessions(&self) {
        let now = SystemTime::now();
        let session_timeout = Duration::from_secs(24 * 60 * 60); // 24 hours

        let mut sessions = self.active_sessions.write().await;
        let initial_count = sessions.len();
        
        sessions.retain(|_, session| {
            now.duration_since(session.last_accessed)
                .map(|duration| duration < session_timeout)
                .unwrap_or(false)
        });

        let removed_count = initial_count - sessions.len();
        if removed_count > 0 {
            tracing::info!("Cleaned up {} expired sessions", removed_count);
        }

        self.update_metrics(|m| m.active_sessions = sessions.len() as u64).await;
    }

    // Private helper methods

    /// Generate access and refresh tokens
    async fn generate_tokens(&self, user: &User) -> Result<(String, String), AuthError> {
        let access_token = self.generate_access_token(user)?;
        let refresh_token = self.generate_refresh_token(user)?;
        Ok((access_token, refresh_token))
    }

    /// Generate access token
    fn generate_access_token(&self, user: &User) -> Result<String, AuthError> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)
            .map_err(|_| AuthError::TokenGenerationFailed)?;
        
        let exp = now + self.jwt_config.access_token_expiry;
        
        let claims = TokenClaims {
            sub: user.id.to_string(),
            username: user.username.clone(),
            email: user.email.clone(),
            iat: now.as_secs() as i64,
            exp: exp.as_secs() as i64,
            jti: Uuid::new_v4().to_string(),
        };

        let header = Header::new(self.jwt_config.algorithm);
        let encoding_key = EncodingKey::from_secret(self.jwt_config.secret.as_bytes());
        
        encode(&header, &claims, &encoding_key)
            .map_err(|_| AuthError::TokenGenerationFailed)
    }

    /// Generate refresh token
    fn generate_refresh_token(&self, user: &User) -> Result<String, AuthError> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)
            .map_err(|_| AuthError::TokenGenerationFailed)?;
        
        let exp = now + self.jwt_config.refresh_token_expiry;
        
        let claims = RefreshTokenClaims {
            sub: user.id.to_string(),
            jti: Uuid::new_v4().to_string(),
            iat: now.as_secs() as i64,
            exp: exp.as_secs() as i64,
            iss: Some(self.jwt_config.issuer.clone()),
            token_type: "refresh".to_string(),
        };

        let header = Header::new(self.jwt_config.algorithm);
        let encoding_key = EncodingKey::from_secret(self.jwt_config.secret.as_bytes());
        
        encode(&header, &claims, &encoding_key)
            .map_err(|_| AuthError::TokenGenerationFailed)
    }

    /// Decode and validate access token
    fn decode_access_token(&self, token: &str) -> Result<TokenClaims, AuthError> {
        let decoding_key = DecodingKey::from_secret(self.jwt_config.secret.as_bytes());
        let mut validation = Validation::new(self.jwt_config.algorithm);
        validation.set_issuer(&[&self.jwt_config.issuer]);
        validation.set_audience(&[&self.jwt_config.audience]);
        
        decode::<TokenClaims>(token, &decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|_| AuthError::InvalidToken)
    }

    /// Decode and validate refresh token
    fn validate_refresh_token(&self, token: &str) -> Result<RefreshTokenClaims, AuthError> {
        let decoding_key = DecodingKey::from_secret(self.jwt_config.secret.as_bytes());
        let mut validation = Validation::new(self.jwt_config.algorithm);
        validation.set_issuer(&[&self.jwt_config.issuer]);
        
        decode::<RefreshTokenClaims>(token, &decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|_| AuthError::InvalidToken)
    }

    /// Hash password using bcrypt
    fn hash_password(&self, password: &str) -> Result<String, AuthError> {
        bcrypt::hash(password, bcrypt::DEFAULT_COST)
            .map_err(|_| AuthError::PasswordHashingFailed)
    }

    /// Verify password against hash
    fn verify_password(&self, password: &str, hash: &str) -> Result<bool, AuthError> {
        bcrypt::verify(password, hash)
            .map_err(|_| AuthError::PasswordVerificationFailed)
    }

    /// Validate registration request
    async fn validate_registration(&self, request: &RegisterRequest) -> Result<(), AuthError> {
        // Validate email format
        if !self.is_valid_email(&request.email) {
            return Err(AuthError::InvalidEmail);
        }

        // Validate username
        if !self.is_valid_username(&request.username) {
            return Err(AuthError::InvalidUsername);
        }

        // Validate password strength
        self.validate_password(&request.password)?;

        Ok(())
    }

    /// Validate email format
    fn is_valid_email(&self, email: &str) -> bool {
        email.contains('@') && email.contains('.') && email.len() > 5
    }

    /// Validate username format
    fn is_valid_username(&self, username: &str) -> bool {
        username.len() >= 3 && username.len() <= 30 && 
        username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    }

    /// Validate password strength
    fn validate_password(&self, password: &str) -> Result<(), AuthError> {
        let policy = PasswordPolicy::default();
        
        if password.len() < policy.min_length {
            return Err(AuthError::WeakPassword(format!(
                "Password must be at least {} characters long", 
                policy.min_length
            )));
        }

        if password.len() > policy.max_length {
            return Err(AuthError::WeakPassword(format!(
                "Password must be no more than {} characters long", 
                policy.max_length
            )));
        }

        if policy.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            return Err(AuthError::WeakPassword(
                "Password must contain at least one uppercase letter".to_string()
            ));
        }

        if policy.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            return Err(AuthError::WeakPassword(
                "Password must contain at least one lowercase letter".to_string()
            ));
        }

        if policy.require_numbers && !password.chars().any(|c| c.is_numeric()) {
            return Err(AuthError::WeakPassword(
                "Password must contain at least one number".to_string()
            ));
        }

        if policy.require_special_chars && !password.chars().any(|c| !c.is_alphanumeric()) {
            return Err(AuthError::WeakPassword(
                "Password must contain at least one special character".to_string()
            ));
        }

        Ok(())
    }

    /// Create user session
    async fn create_session(
        &self,
        user: &User,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(), AuthError> {
        let session_id = Uuid::new_v4().to_string();
        let now = SystemTime::now();
        
        let session_info = SessionInfo {
            user_id: user.id,
            username: user.username.clone(),
            created_at: now,
            last_accessed: now,
            ip_address,
            user_agent,
        };

        // Store in memory
        let mut sessions = self.active_sessions.write().await;
        sessions.insert(session_id.clone(), session_info);
        
        // Store in database
        let _ = self.database.create_session(
            user.id,
            &session_id,
            now.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64 + 
            self.jwt_config.access_token_expiry.as_secs() as i64,
        ).await;

        self.update_metrics(|m| m.active_sessions = sessions.len() as u64).await;

        Ok(())
    }

    /// Update service metrics
    async fn update_metrics<F>(&self, update_fn: F)
    where
        F: FnOnce(&mut AuthMetrics),
    {
        let mut metrics = self.metrics.write().await;
        update_fn(&mut *metrics);
    }
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: "your-secret-key".to_string(),
            access_token_expiry: Duration::from_secs(15 * 60), // 15 minutes
            refresh_token_expiry: Duration::from_secs(7 * 24 * 60 * 60), // 7 days
            issuer: "crmb-streaming-webapp".to_string(),
            audience: "crmb-users".to_string(),
            algorithm: Algorithm::HS256,
        }
    }
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special_chars: false,
            max_length: 128,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Database;

    async fn create_test_auth_service() -> AuthService {
        let database = Arc::new(Database::new(":memory:").await.unwrap());
        let jwt_config = JwtConfig {
            secret: "test-secret-key".to_string(),
            ..Default::default()
        };
        
        AuthService::new(database, jwt_config)
    }

    #[tokio::test]
    async fn test_password_hashing() {
        let auth_service = create_test_auth_service().await;
        let password = "test_password123";
        
        let hash = auth_service.hash_password(password).unwrap();
        assert!(auth_service.verify_password(password, &hash).unwrap());
        assert!(!auth_service.verify_password("wrong_password", &hash).unwrap());
    }

    #[tokio::test]
    async fn test_token_generation_and_validation() {
        let auth_service = create_test_auth_service().await;
        let user = User {
            id: 1,
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "hash".to_string(),
            created_at: "2023-01-01T00:00:00Z".to_string(),
            updated_at: "2023-01-01T00:00:00Z".to_string(),
        };

        let access_token = auth_service.generate_access_token(&user).unwrap();
        let claims = auth_service.decode_access_token(&access_token).unwrap();
        
        assert_eq!(claims.sub, user.id.to_string());
        assert_eq!(claims.username, user.username);
        assert_eq!(claims.email, user.email);
    }

    #[test]
    fn test_email_validation() {
        let auth_service = tokio_test::block_on(create_test_auth_service());
        
        assert!(auth_service.is_valid_email("test@example.com"));
        assert!(auth_service.is_valid_email("user.name+tag@domain.co.uk"));
        assert!(!auth_service.is_valid_email("invalid-email"));
        assert!(!auth_service.is_valid_email("@domain.com"));
        assert!(!auth_service.is_valid_email("user@"));
    }

    #[test]
    fn test_username_validation() {
        let auth_service = tokio_test::block_on(create_test_auth_service());
        
        assert!(auth_service.is_valid_username("testuser"));
        assert!(auth_service.is_valid_username("test_user"));
        assert!(auth_service.is_valid_username("test-user"));
        assert!(auth_service.is_valid_username("user123"));
        assert!(!auth_service.is_valid_username("ab")); // too short
        assert!(!auth_service.is_valid_username("test user")); // contains space
        assert!(!auth_service.is_valid_username("test@user")); // contains @
    }

    #[test]
    fn test_password_validation() {
        let auth_service = tokio_test::block_on(create_test_auth_service());
        
        assert!(auth_service.validate_password("Password123").is_ok());
        assert!(auth_service.validate_password("MySecurePass1").is_ok());
        assert!(auth_service.validate_password("short").is_err()); // too short
        assert!(auth_service.validate_password("password123").is_err()); // no uppercase
        assert!(auth_service.validate_password("PASSWORD123").is_err()); // no lowercase
        assert!(auth_service.validate_password("Password").is_err()); // no numbers
    }
}