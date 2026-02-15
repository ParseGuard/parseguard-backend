use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// User model from database
///
/// Represents a user in the system with all database fields
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct User {
    /// Unique user identifier
    pub id: Uuid,
    
    /// User email address (unique)
    pub email: String,
    
    /// Bcrypt hashed password
    #[serde(skip_serializing)]
    pub password_hash: String,
    
    /// User's full name
    pub full_name: String,
    
    /// Account creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// JWT Claims structure
///
/// Contains the data embedded in JWT tokens for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// DTO for user registration
///
/// Used when creating a new user account
#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserDto {
    /// Email address
    #[validate(email(message = "Invalid email address"))]
    pub email: String,
    
    /// Password (minimum 8 characters)
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
    
    /// Full name
    #[validate(length(min = 2, message = "Full name must be at least 2 characters"))]
    pub full_name: String,
}

/// DTO for user login
///
/// Used for authentication requests
#[derive(Debug, Deserialize, Validate)]
pub struct LoginDto {
    /// Email address
    #[validate(email(message = "Invalid email address"))]
    pub email: String,
    
    /// Password
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

/// User response (excludes sensitive data)
///
/// Used in API responses to avoid exposing password hash
#[derive(Debug, Serialize)]
pub struct UserResponse {
    /// User ID
    pub id: Uuid,
    
    /// Email address
    pub email: String,
    
    /// Full name
    pub full_name: String,
    
    /// Account creation timestamp
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    /// Convert User to UserResponse
    ///
    /// # Arguments
    ///
    /// * `user` - User model from database
    ///
    /// # Returns
    ///
    /// Safe user representation without password hash
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            full_name: user.full_name,
            created_at: user.created_at,
        }
    }
}

/// JWT authentication response
///
/// Returned after successful login or registration
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    /// User information
    pub user: UserResponse,
    
    /// JWT access token
    pub access_token: String,
}
