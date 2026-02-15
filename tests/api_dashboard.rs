use common::spawn_app;
use uuid::Uuid;

mod common;

#[tokio::test]
async fn dashboard_stats_returns_401_without_auth() {
    let app = spawn_app().await;

    let response = app.get_dashboard_stats().await;

    assert_eq!(401, response.status().as_u16());
}

#[tokio::test]
async fn dashboard_stats_returns_200_with_valid_auth() {
    let app = spawn_app().await;

    // 1. Register
    let email = format!("test-{}@example.com", Uuid::new_v4());
    let register_body = serde_json::json!({
        "full_name": "Test User",
        "email": email,
        "password": "password123",
        "confirm_password": "password123"
    });
    app.post_register(&register_body).await;

    // 2. Login
    let login_body = serde_json::json!({
        "email": email,
        "password": "password123"
    });
    let response = app.post_login(&login_body).await;
    assert_eq!(200, response.status().as_u16());

    // 3. Get Stats
    let response = app.get_dashboard_stats().await;
    assert_eq!(200, response.status().as_u16());
}
