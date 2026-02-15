use axum::{
    extract::Request,
    http::header,
    middleware::Next,
    response::Response,
};

use tracing::error;

use crate::{
    error::{AppError, AppResult},
    models::Claims,
    services::AuthService,
};

/// Auth middleware for protected routes
///
/// Extracts and validates JWT token from Authorization header
///
/// # Arguments
///
/// * `request` - HTTP request
/// * `next` - Next middleware/handler
///
/// # Returns
///
/// HTTP response or error
///
/// # Errors
///
/// Returns 401 if token is missing or invalid
pub async fn auth_middleware(
    mut request: Request,
    next: Next,
) -> AppResult<Response> {
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            error!("❌ Auth Failed: Missing authorization header");
            AppError::Auth("Missing authorization header".to_string())
        })?;

    // Check Bearer scheme
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| {
            error!("❌ Auth Failed: Invalid authorization header format (expected 'Bearer <token>')");
            AppError::Auth("Invalid authorization header format".to_string())
        })?;

    // Get JWT secret from environment
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "default-secret-key-change-in-production".to_string());

    // Validate token
    let auth_service = AuthService::new(jwt_secret);
    
    match auth_service.validate_token(token) {
        Ok(claims) => {
            // Add claims to request extensions for handlers to access
            request.extensions_mut().insert(claims);
            Ok(next.run(request).await)
        }
        Err(e) => {
            error!("❌ Auth Failed: Token validation failed: {}", e);
            Err(e)
        }
    }
}

/// Extension trait to get authenticated user from request
pub trait AuthUser {
    /// Get claims from request extensions
    ///
    /// # Returns
    ///
    /// Claims if authenticated
    ///
    /// # Errors
    ///
    /// Returns error if not authenticated
    fn claims(&self) -> AppResult<&Claims>;
}

impl AuthUser for Request {
    fn claims(&self) -> AppResult<&Claims> {
        self.extensions()
            .get::<Claims>()
            .ok_or_else(|| AppError::Auth("Unauthorized".to_string()))
    }
}
