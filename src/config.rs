use serde::Deserialize;

/// Application configuration loaded from environment variables
///
/// This struct holds all configuration values needed to run the application.
/// Values are loaded from environment variables using dotenvy.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// PostgreSQL database connection string
    pub database_url: String,
    
    /// Secret key for JWT token signing
    pub jwt_secret: String,
    
    /// OLLAMA API base URL for AI requests
    pub ollama_api_url: String,
    
    /// Server port to bind to
    pub port: u16,
}

impl Config {
    /// Load configuration from environment variables
    ///
    /// # Returns
    ///
    /// Returns a `Config` instance with all required configuration values
    ///
    /// # Panics
    ///
    /// Panics if any required environment variable is missing
    ///
    /// # Example
    ///
    /// ```no_run
    /// let config = Config::from_env();
    /// println!("Server will run on port {}", config.port);
    /// ```
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        
        Self {
            database_url: std::env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set"),
            jwt_secret: std::env::var("JWT_SECRET")
                .expect("JWT_SECRET must be set"),
            ollama_api_url: std::env::var("OLLAMA_API_URL")
                .unwrap_or_else(|_| "http://localhost:11434".to_string()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "8000".to_string())
                .parse()
                .expect("PORT must be a valid number"),
        }
    }
}
