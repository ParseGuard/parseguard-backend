use axum::async_trait;
use sqlx::PgPool;

/// Base trait for all services
#[async_trait]
pub trait BaseService: Send + Sync {
    /// Create a new instance of the service
    fn new(pool: PgPool) -> Self;
}

/// Helper macro to implement common service traits
#[macro_export]
macro_rules! impl_service {
    ($service:ident) => {
        impl Clone for $service {
            fn clone(&self) -> Self {
                Self {
                    pool: self.pool.clone(),
                    // Clone other fields if present, assuming they are Arc or Clone
                    ..self.clone_fields()
                }
            }
        }

        #[axum::async_trait]
        impl<S> axum::extract::FromRequestParts<S> for $service
        where
            S: Send + Sync,
            AppState: axum::extract::FromRef<S>,
        {
            type Rejection = std::convert::Infallible;

            async fn from_request_parts(_parts: &mut axum::http::request::Parts, state: &S) -> Result<Self, Self::Rejection> {
                let app_state = AppState::from_ref(state);
                Ok(Self::new(app_state.pool))
            }
        }
    };
}
