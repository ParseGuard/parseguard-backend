use axum::{extract::{Extension, State}, http::StatusCode, Json};
use validator::Validate;

use crate::{
    db::repository::UserRepository,
    error::{AppError, AppResult},
    models::{AuthResponse, Claims, CreateUserDto, LoginDto, UserResponse},
    services::AuthService,
    AppState,
};

/// Register a new user
///
/// # Arguments
///
/// * `state` - Application state
/// * `dto` - User registration data
///
/// # Returns
///
/// AuthResponse with user info and JWT token
///
/// # Errors
///
/// Returns validation error or database error
pub async fn register(
    State(state): State<AppState>,
    Json(dto): Json<CreateUserDto>,
) -> AppResult<(StatusCode, Json<AuthResponse>)> {
    // Log incoming registration attempt
    tracing::info!("🔐 Registration attempt | Email: {}", dto.email);
    tracing::debug!("📝 Registration data | Name: {}, Email: {}", dto.full_name, dto.email);
    
    // Validate input
    dto.validate()
        .map_err(|e| {
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

    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            user: UserResponse::from(user),
            access_token: token,
        }),
    ))
}

/// Login user
///
/// # Arguments
///
/// * `state` - Application state
/// * `dto` - Login credentials
///
/// # Returns
///
/// AuthResponse with user info and JWT token
///
/// # Errors
///
/// Returns authentication error if credentials are invalid
pub async fn login(
    State(state): State<AppState>,
    Json(dto): Json<LoginDto>,
) -> AppResult<Json<AuthResponse>> {
    // Validate input
    dto.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

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

    Ok(Json(AuthResponse {
        user: UserResponse::from(user),
        access_token: token,
    }))
}

/// Refresh JWT token
///
/// # Arguments
///
/// * `state` - Application state
/// * `claims` - Authenticated user claims from middleware
///
/// # Returns
///
/// New JWT token
///
/// # Errors
///
/// Returns authentication error if user not found
pub async fn refresh(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> AppResult<Json<AuthResponse>> {
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

    Ok(Json(AuthResponse {
        user: UserResponse::from(user),
        access_token: token,
    }))
}
