use axum::{extract::State, http::StatusCode, Json};
use validator::Validate;

use crate::{
    db::repository::UserRepository,
    error::{AppError, AppResult},
    models::{AuthResponse, CreateUserDto, LoginDto, UserResponse},
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
    // Validate input
    dto.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let user_repo = UserRepository::new(state.pool.clone());
    let auth_service = AuthService::new(state.config.jwt_secret.clone());

    // Check if email already exists
    if user_repo.email_exists(&dto.email).await? {
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
