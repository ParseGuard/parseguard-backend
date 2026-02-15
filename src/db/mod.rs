use sqlx::{postgres::PgPoolOptions, PgPool};

pub mod repository;

use crate::error::AppResult;

/// Create a PostgreSQL connection pool
///
/// # Arguments
///
/// * `database_url` - PostgreSQL connection string
///
/// # Returns
///
/// Returns a connection pool ready for use
///
/// # Errors
///
/// Returns an error if the database connection fails
pub async fn create_pool(database_url: &str) -> AppResult<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    Ok(pool)
}

/// Run database migrations
///
/// # Arguments
///
/// * `pool` - Database connection pool
///
/// # Returns
///
/// Returns Ok(()) if migrations succeed
///
/// # Errors
///
/// Returns an error if migrations fail
pub async fn run_migrations(pool: &PgPool) -> AppResult<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await?;

    Ok(())
}
