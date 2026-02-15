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
    // 1. Try Authorization header first
    let token = if let Some(auth_header) = request.headers().get(header::AUTHORIZATION) {
        auth_header.to_str()
            .ok()
            .and_then(|h| h.strip_prefix("Bearer "))
            .map(|t| t.to_string())
    } else {
        None
    };

    // 2. Fallback to Cookie header
    let token = token.or_else(|| {
        request.headers()
            .get(header::COOKIE)
            .and_then(|c| c.to_str().ok())
            .and_then(|cookie_str| {
                cookie_str
                    .split(';')
                    .find_map(|s| {
                        let parts: Vec<&str> = s.trim().split('=').collect();
                        if parts.len() == 2 && parts[0] == "auth_token" {
                            Some(parts[1].to_string())
                        } else {
                            None
                        }
                    })
            })
    });

    // 3. Validate token
    let token = token.ok_or_else(|| {
        error!("❌ Auth Failed: No token found in Authorization header or Cookie");
        AppError::Auth("Missing authentication token".to_string())
    })?;

    // Get JWT secret from environment
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "default-secret-key-change-in-production".to_string());

    // Validate token
    let auth_service = AuthService::new(jwt_secret);
    
    match auth_service.validate_token(&token) {
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
