use axum::{
    extract::{Extension, Query, State},
    http::{header, HeaderMap, StatusCode},
    Json,
    response::IntoResponse,
};
use serde::Deserialize;
use validator::{Validate, ValidationErrors}; // Added ValidationErrors for type inference

use crate::{
    db::repository::UserRepository,
    error::{AppError, AppResult},
    models::{AuthResponse, Claims, CreateUserDto, LoginDto, UserResponse},
    services::AuthService,
    AppState,
};

/// Login query parameters
#[derive(Deserialize)]
pub struct LoginParams {
    #[serde(default)]
    pub return_token: bool,
}

// Helper to create cookie header
fn create_auth_cookie(token: &str) -> String {
    format!(
        "auth_token={}; HttpOnly; Path=/; SameSite=Lax; Max-Age=604800", // 7 days
        token
    )
}

/// Register a new user
pub async fn register(
    State(state): State<AppState>,
    Query(params): Query<LoginParams>,
    Json(dto): Json<CreateUserDto>,
) -> AppResult<impl IntoResponse> {
    // Log incoming registration attempt
    tracing::info!("🔐 Registration attempt | Email: {}", dto.email);
    tracing::debug!("📝 Registration data | Name: {}, Email: {}", dto.full_name, dto.email);
    
    // Validate input
    dto.validate()
        .map_err(|e: ValidationErrors| {
            tracing::error!("❌ Validation failed: {}", e);
            AppError::Validation(e.to_string())
        })?;

    let user_repo = UserRepository::new(state.pool.clone());
    let auth_service = AuthService::new(state.config.jwt_secret.clone());

    // Check if email already exists
    if user_repo.email_exists(&dto.email).await? {
        tracing::warn!("⚠️  Email already registered: {}", dto.email);
        return Err(AppError::Validation("Email already registered".to_string()));
    }

    // Hash password
    let password_hash = auth_service.hash_password(&dto.password)?;

    // Create user
    let user = user_repo.create(&dto, password_hash).await?;

    // Generate JWT token
    let token = auth_service.generate_token(user.id, &user.email)?;

    // Create headers
    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        create_auth_cookie(&token).parse().unwrap(),
    );

    let access_token = if params.return_token { Some(token) } else { None };

    Ok((
        StatusCode::CREATED,
        headers,
        Json(AuthResponse {
            user: UserResponse::from(user),
            access_token,
        }),
    ))
}

/// Login user
pub async fn login(
    State(state): State<AppState>,
    Query(params): Query<LoginParams>,
    Json(dto): Json<LoginDto>,
) -> AppResult<impl IntoResponse> {
    // Validate input
    dto.validate()
        .map_err(|e: ValidationErrors| AppError::Validation(e.to_string()))?;

    let user_repo = UserRepository::new(state.pool.clone());
    let auth_service = AuthService::new(state.config.jwt_secret.clone());

    // Find user by email
    let user = user_repo
        .find_by_email(&dto.email)
        .await?
        .ok_or_else(|| AppError::Auth("Invalid email or password".to_string()))?;

    // Verify password
    let valid = auth_service.verify_password(&dto.password, &user.password_hash)?;
    if !valid {
        return Err(AppError::Auth("Invalid email or password".to_string()));
    }

    // Generate JWT token
    let token = auth_service.generate_token(user.id, &user.email)?;

    // Create headers
    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        create_auth_cookie(&token).parse().unwrap(),
    );

    let access_token = if params.return_token { Some(token) } else { None };

    Ok((
        headers,
        Json(AuthResponse {
            user: UserResponse::from(user),
            access_token,
        }),
    ))
}

/// Refresh JWT token
pub async fn refresh(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> AppResult<impl IntoResponse> {
    let user_id = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Internal("Invalid user ID in token".to_string()))?;

    let user_repo = UserRepository::new(state.pool.clone());
    let auth_service = AuthService::new(state.config.jwt_secret.clone());

    // Verify user still exists
    let user = user_repo
        .find_by_id(user_id)
        .await?
        .ok_or_else(|| AppError::Auth("User not found".to_string()))?;

    // Generate new JWT token
    let token = auth_service.generate_token(user.id, &user.email)?;

    // Create headers
    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        create_auth_cookie(&token).parse().unwrap(),
    );

    Ok((
        headers,
        Json(AuthResponse {
            user: UserResponse::from(user),
            access_token: None,
        }),
    ))
}
