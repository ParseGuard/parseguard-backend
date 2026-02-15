use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use tracing::{info, instrument};
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    models::Claims,
};

/// Auth service for JWT operations
///
/// Handles token generation, validation, and password hashing
pub struct AuthService {
    /// JWT secret key for signing tokens
    secret: String,
}

impl AuthService {
    /// Create a new AuthService
    ///
    /// # Arguments
    ///
    /// * `secret` - JWT secret key for signing tokens
    ///
    /// # Returns
    ///
    /// New AuthService instance
    pub fn new(secret: String) -> Self {
        info!("🚀 AuthService started");
        Self { secret }
    }

    /// Generate JWT access token
    ///
    /// # Arguments
    ///
    /// * `user_id` - User UUID
    /// * `email` - User email address
    ///
    /// # Returns
    ///
    /// JWT token string
    ///
    /// # Errors
    ///
    /// Returns error if token encoding fails
    #[instrument(skip(self))]
    pub fn generate_token(&self, user_id: Uuid, email: &str) -> AppResult<String> {
        info!("Generating token for user: {}", email);
        let now = Utc::now();
        let expires_at = now + Duration::hours(24); // 24 hour expiry

        let claims = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
            exp: expires_at.timestamp() as usize,
            iat: now.timestamp() as usize,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )?;

        Ok(token)
    }

    /// Validate and decode JWT token
    ///
    /// # Arguments
    ///
    /// * `token` - JWT token string
    ///
    /// # Returns
    ///
    /// Decoded Claims
    ///
    /// # Errors
    ///
    /// Returns error if token is invalid or expired
    #[instrument(skip(self, token))]
    pub fn validate_token(&self, token: &str) -> AppResult<Claims> {
        // Don't log the full token for security, just that we are validating
        info!("Validating token");
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )?;

        Ok(token_data.claims)
    }

    /// Hash password using bcrypt
    ///
    /// # Arguments
    ///
    /// * `password` - Plain text password
    ///
    /// # Returns
    ///
    /// Bcrypt hashed password
    ///
    /// # Errors
    ///
    /// Returns error if hashing fails
    #[instrument(skip(self, password))]
    pub fn hash_password(&self, password: &str) -> AppResult<String> {
        info!("Hashing password");
        let hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)
            .map_err(|e| AppError::Internal(format!("Password hashing failed: {}", e)))?;
        Ok(hash)
    }

    /// Verify password against hash
    ///
    /// # Arguments
    ///
    /// * `password` - Plain text password
    /// * `hash` - Bcrypt password hash
    ///
    /// # Returns
    ///
    /// true if password matches, false otherwise
    ///
    /// # Errors
    ///
    /// Returns error if verification process fails
    #[instrument(skip(self, password, hash))]
    pub fn verify_password(&self, password: &str, hash: &str) -> AppResult<bool> {
        info!("Verifying password");
        let valid = bcrypt::verify(password, hash)
            .map_err(|e| AppError::Internal(format!("Password verification failed: {}", e)))?;
        Ok(valid)
    }

    /// Create authentication cookie header value
    ///
    /// # Arguments
    ///
    /// * `token` - JWT token string
    ///
    /// # Returns
    ///
    /// Cookie header string
    pub fn create_auth_cookie(&self, token: &str) -> String {
        format!(
            "auth_token={}; HttpOnly; Path=/; SameSite=Lax; Max-Age=604800", // 7 days
            token
        )
    }
}
