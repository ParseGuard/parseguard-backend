use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, AppResult};

/// JWT Claims structure
///
/// Contains the data embedded in the JWT token
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    
    /// Email address
    pub email: String,
    
    /// Expiration time (Unix timestamp)
    pub exp: usize,
    
    /// Issued at (Unix timestamp)
    pub iat: usize,
}

/// Auth service for JWT operations
///
/// Handles token generation and validation
pub struct AuthService {
    /// JWT secret key
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
    pub fn generate_token(&self, user_id: Uuid, email: &str) -> AppResult<String> {
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
    pub fn validate_token(&self, token: &str) -> AppResult<Claims> {
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
    pub fn hash_password(&self, password: &str) -> AppResult<String> {
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
    pub fn verify_password(&self, password: &str, hash: &str) -> AppResult<bool> {
        let valid = bcrypt::verify(password, hash)
            .map_err(|e| AppError::Internal(format!("Password verification failed: {}", e)))?;
        Ok(valid)
    }
}
