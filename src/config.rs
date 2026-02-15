/// Application configuration
#[derive(Clone)]
pub struct Config {
    /// Database connection URL
    pub database_url: String,
    
    /// Server port
    pub port: u16,
    
    /// JWT secret key
    pub jwt_secret: String,
    
    /// JWT expiration time in seconds (default: 24 hours)
    pub jwt_expiration: i64,
    
    /// Upload directory for documents
    pub upload_dir: String,
    
    /// Maximum file upload size in bytes (default: 50MB)
    pub max_file_size: usize,
    
    /// OLLAMA API URL for AI processing
    pub ollama_url: String,
}

impl Config {
    /// Load configuration from environment variables
    ///
    /// # Returns
    ///
    /// Configuration instance
    ///
    /// # Panics
    ///
    /// Panics if required environment variables are missing
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        Self {
            database_url: std::env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set"),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "8000".to_string())
                .parse()
                .expect("PORT must be a valid number"),
            jwt_secret: std::env::var("JWT_SECRET")
                .expect("JWT_SECRET must be set"),
            jwt_expiration: std::env::var("JWT_EXPIRATION")
                .unwrap_or_else(|_| "86400".to_string()) // 24 hours
                .parse()
                .expect("JWT_EXPIRATION must be a valid number"),
            upload_dir: std::env::var("UPLOAD_DIR")
                .unwrap_or_else(|_| "./uploads".to_string()),
            max_file_size: std::env::var("MAX_FILE_SIZE")
                .unwrap_or_else(|_| "52428800".to_string()) // 50MB
                .parse()
                .expect("MAX_FILE_SIZE must be a valid number"),
            ollama_url: std::env::var("OLLAMA_URL")
                .unwrap_or_else(|_| "http://localhost:11434".to_string()),
        }
    }
}
