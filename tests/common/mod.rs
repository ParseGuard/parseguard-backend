use std::net::TcpListener;

pub struct TestApp {
    pub address: String,
    pub pool: sqlx::PgPool,
    pub api_client: reqwest::Client,
}

pub async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    listener.set_nonblocking(true).expect("Failed to set non-blocking");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let config = parseguard_backend::config::Config::from_env();
    let pool = parseguard_backend::db::create_pool(&config.database_url).await.unwrap();
    
    // Create AppState
    let state = parseguard_backend::AppState {
        pool: pool.clone(),
        config: config.clone(),
    };

    // Build application router
    let app = axum::Router::new()
        .route("/health", axum::routing::get(parseguard_backend::health_check))
        .nest("/api", parseguard_backend::api::create_router(state));
    
    // Spawn the server
    let server = axum::serve(tokio::net::TcpListener::from_std(listener).unwrap(), app);
    let _ = tokio::spawn(async move {
        server.await.unwrap();
    });

    // Create client with cookie store
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();

    TestApp {
        address,
        pool,
        api_client: client,
    }
}

impl TestApp {
    pub async fn post_register(&self, body: &serde_json::Value) -> reqwest::Response {
        self.api_client
            .post(&format!("{}/api/auth/register", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_login(&self, body: &serde_json::Value) -> reqwest::Response {
        self.api_client
            .post(&format!("{}/api/auth/login", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_dashboard_stats(&self) -> reqwest::Response {
        self.api_client
            .get(&format!("{}/api/dashboard/stats", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }
}
