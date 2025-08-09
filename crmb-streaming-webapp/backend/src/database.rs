use anyhow::Result;
use sqlx::{sqlite::SqlitePool, Row, SqlitePool as Pool};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::models::user::{User, UserCreate, WatchlistItem};

#[derive(Clone)]
pub struct Database {
    pool: Pool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePool::connect(database_url).await?;
        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<()> {
        // Create users table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT UNIQUE NOT NULL,
                email TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                is_active BOOLEAN NOT NULL DEFAULT 1
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create watchlist table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS watchlist (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                tmdb_id INTEGER NOT NULL,
                media_type TEXT NOT NULL, -- 'movie' or 'tv'
                title TEXT NOT NULL,
                poster_path TEXT,
                overview TEXT,
                release_date TEXT,
                vote_average REAL,
                added_at TEXT NOT NULL,
                watched BOOLEAN NOT NULL DEFAULT 0,
                watch_progress REAL DEFAULT 0.0,
                FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
                UNIQUE(user_id, tmdb_id, media_type)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create user sessions table for JWT token management
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS user_sessions (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                token_hash TEXT NOT NULL,
                expires_at TEXT NOT NULL,
                created_at TEXT NOT NULL,
                is_active BOOLEAN NOT NULL DEFAULT 1,
                FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes for better performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_email ON users (email)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_username ON users (username)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_watchlist_user_id ON watchlist (user_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_watchlist_tmdb_id ON watchlist (tmdb_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON user_sessions (user_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_sessions_token_hash ON user_sessions (token_hash)")
            .execute(&self.pool)
            .await?;

        tracing::info!("Database migration completed successfully");
        Ok(())
    }

    // User operations
    pub async fn create_user(&self, user_create: &UserCreate) -> Result<User> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO users (id, username, email, password_hash, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(&user_create.username)
        .bind(&user_create.email)
        .bind(&user_create.password_hash)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        self.get_user_by_id(&id).await
    }

    pub async fn get_user_by_id(&self, id: &str) -> Result<User> {
        let row = sqlx::query(
            "SELECT id, username, email, password_hash, created_at, updated_at, is_active FROM users WHERE id = ?"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(User {
            id: row.get("id"),
            username: row.get("username"),
            email: row.get("email"),
            password_hash: row.get("password_hash"),
            created_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))?
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("updated_at"))?
                .with_timezone(&Utc),
            is_active: row.get("is_active"),
        })
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        let row = sqlx::query(
            "SELECT id, username, email, password_hash, created_at, updated_at, is_active FROM users WHERE email = ?"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(User {
                id: row.get("id"),
                username: row.get("username"),
                email: row.get("email"),
                password_hash: row.get("password_hash"),
                created_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))?
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("updated_at"))?
                    .with_timezone(&Utc),
                is_active: row.get("is_active"),
            })),
            None => Ok(None),
        }
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let row = sqlx::query(
            "SELECT id, username, email, password_hash, created_at, updated_at, is_active FROM users WHERE username = ?"
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(User {
                id: row.get("id"),
                username: row.get("username"),
                email: row.get("email"),
                password_hash: row.get("password_hash"),
                created_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))?
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("updated_at"))?
                    .with_timezone(&Utc),
                is_active: row.get("is_active"),
            })),
            None => Ok(None),
        }
    }

    // Watchlist operations
    pub async fn add_to_watchlist(&self, user_id: &str, item: &WatchlistItem) -> Result<()> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO watchlist 
            (id, user_id, tmdb_id, media_type, title, poster_path, overview, release_date, vote_average, added_at, watched, watch_progress)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(user_id)
        .bind(item.tmdb_id)
        .bind(&item.media_type)
        .bind(&item.title)
        .bind(&item.poster_path)
        .bind(&item.overview)
        .bind(&item.release_date)
        .bind(item.vote_average)
        .bind(&now)
        .bind(item.watched)
        .bind(item.watch_progress)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_watchlist(&self, user_id: &str) -> Result<Vec<WatchlistItem>> {
        let rows = sqlx::query(
            r#"
            SELECT id, tmdb_id, media_type, title, poster_path, overview, release_date, 
                   vote_average, added_at, watched, watch_progress
            FROM watchlist 
            WHERE user_id = ? 
            ORDER BY added_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let mut items = Vec::new();
        for row in rows {
            items.push(WatchlistItem {
                id: Some(row.get("id")),
                tmdb_id: row.get("tmdb_id"),
                media_type: row.get("media_type"),
                title: row.get("title"),
                poster_path: row.get("poster_path"),
                overview: row.get("overview"),
                release_date: row.get("release_date"),
                vote_average: row.get("vote_average"),
                added_at: Some(
                    DateTime::parse_from_rfc3339(&row.get::<String, _>("added_at"))?
                        .with_timezone(&Utc),
                ),
                watched: row.get("watched"),
                watch_progress: row.get("watch_progress"),
            });
        }

        Ok(items)
    }

    pub async fn remove_from_watchlist(&self, user_id: &str, tmdb_id: i32, media_type: &str) -> Result<()> {
        sqlx::query(
            "DELETE FROM watchlist WHERE user_id = ? AND tmdb_id = ? AND media_type = ?"
        )
        .bind(user_id)
        .bind(tmdb_id)
        .bind(media_type)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Session management
    pub async fn create_session(&self, user_id: &str, token_hash: &str, expires_at: DateTime<Utc>) -> Result<String> {
        let session_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let expires_at_str = expires_at.to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO user_sessions (id, user_id, token_hash, expires_at, created_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&session_id)
        .bind(user_id)
        .bind(token_hash)
        .bind(&expires_at_str)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(session_id)
    }

    pub async fn invalidate_session(&self, token_hash: &str) -> Result<()> {
        sqlx::query(
            "UPDATE user_sessions SET is_active = 0 WHERE token_hash = ?"
        )
        .bind(token_hash)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn is_session_valid(&self, token_hash: &str) -> Result<bool> {
        let row = sqlx::query(
            "SELECT COUNT(*) as count FROM user_sessions WHERE token_hash = ? AND is_active = 1 AND expires_at > ?"
        )
        .bind(token_hash)
        .bind(Utc::now().to_rfc3339())
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get::<i64, _>("count") > 0)
    }

    pub fn get_pool(&self) -> &Pool {
        &self.pool
    }

    #[cfg(test)]
    pub async fn new_test() -> Result<Self> {
        use sqlx::sqlite::SqlitePoolOptions;
        
        // Create an in-memory database for testing
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await?;

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await?;

        Ok(Self { pool })
    }
}