use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

use crate::{error::AppResult, models::{CreateUserDto, User}};

/// User repository for database operations
///
/// Handles all user-related database queries using the repository pattern
pub struct UserRepository {
    /// Database connection pool
    pool: PgPool,
}

impl UserRepository {
    /// Create a new UserRepository
    ///
    /// # Arguments
    ///
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    ///
    /// New UserRepository instance
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find user by email
    ///
    /// # Arguments
    ///
    /// * `email` - User email address
    ///
    /// # Returns
    ///
    /// Optional User if found
    ///
    /// # Errors
    ///
    /// Returns database error if query fails
    #[instrument(skip(self))]
    pub async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, full_name, created_at, updated_at
             FROM users
             WHERE email = $1"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    /// Find user by ID
    ///
    /// # Arguments
    ///
    /// * `id` - User UUID
    ///
    /// # Returns
    ///
    /// Optional User if found
    ///
    /// # Errors
    ///
    /// Returns database error if query fails
    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: Uuid) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, full_name, created_at, updated_at
             FROM users
             WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    /// Create a new user
    ///
    /// # Arguments
    ///
    /// * `dto` - User creation data
    /// * `password_hash` - Bcrypt hashed password
    ///
    /// # Returns
    ///
    /// Created User
    ///
    /// # Errors
    ///
    /// Returns database error if insertion fails (e.g., duplicate email)
    #[instrument(skip(self, password_hash))]
    pub async fn create(&self, dto: &CreateUserDto, password_hash: String) -> AppResult<User> {
        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users (email, password_hash, full_name)
             VALUES ($1, $2, $3)
             RETURNING id, email, password_hash, full_name, created_at, updated_at"
        )
        .bind(&dto.email)
        .bind(password_hash)
        .bind(&dto.full_name)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    /// Check if email already exists
    ///
    /// # Arguments
    ///
    /// * `email` - Email address to check
    ///
    /// # Returns
    ///
    /// true if email exists, false otherwise
    ///
    /// # Errors
    ///
    /// Returns database error if query fails
    #[instrument(skip(self))]
    pub async fn email_exists(&self, email: &str) -> AppResult<bool> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)"
        )
        .bind(email)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }
}
