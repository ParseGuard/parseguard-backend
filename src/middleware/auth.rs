use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};

use tracing::error;

use crate::{
    error::{AppError, AppResult},
    models::Claims,
    services::AuthService,
    AppState,
};

/// Auth middleware for protected routes
///
/// Extracts and validates JWT token from Authorization header or Cookie
///
/// # Arguments
///
/// * `state` - Application state
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
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> AppResult<Response> {
    // 1. Try Authorization header first
    let token = if let Some(auth_header) = request.headers().get(header::AUTHORIZATION) {
        println!("🔍 [Auth Middleware] Found Authorization Header: {:?}", auth_header);
        auth_header.to_str()
            .ok()
            .and_then(|h| h.strip_prefix("Bearer "))
            .map(|t| t.to_string())
    } else {
        println!("⚠️ [Auth Middleware] No Authorization Header");
        None
    };

    // Always log cookies for debugging
    if let Some(cookies) = request.headers().get(header::COOKIE) {
         println!("🍪 [Auth Middleware] All Cookies: {:?}", cookies);
    } else {
         println!("⚠️ [Auth Middleware] No Cookie header found");
    }

    // 2. Fallback to Cookie header
    let token = token.or_else(|| {
        let cookies = request.headers().get(header::COOKIE);
        println!("🔍 [Auth Middleware] Checking Cookies: {:?}", cookies);
        
        cookies.and_then(|c| c.to_str().ok())
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

    if let Some(ref t) = token {
        println!("✅ [Auth Middleware] Token extracted (len={})", t.len());
    } else {
        println!("❌ [Auth Middleware] Token extraction failed");
    }

    // 3. Validate token
    let token = token.ok_or_else(|| {
        error!("❌ Auth Failed: No token found in Authorization header or Cookie");
        AppError::Auth("Missing authentication token".to_string())
    })?;

    // Validate token using secret from state
    let auth_service = AuthService::new(state.config.jwt_secret.clone());
    
    match auth_service.validate_token(&token) {
        Ok(claims) => {
            println!("✅ [Auth Middleware] Token validated for user: {}", claims.sub);
            // Add claims to request extensions for handlers to access
            request.extensions_mut().insert(claims);
            Ok(next.run(request).await)
        }
        Err(e) => {
            println!("❌ [Auth Middleware] Validation Error: {}", e);
            error!("❌ Auth Failed: Token validation failed: {}", e);
            // DEBUG: Log the token that failed (careful with logs in prod)
            tracing::debug!("Failed token: {}", token); 
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
