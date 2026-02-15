use common::spawn_app;
use uuid::Uuid;

mod common;

#[tokio::test]
async fn register_works_with_valid_data() {
    let app = spawn_app().await;
    
    let email = format!("test-{}@example.com", Uuid::new_v4());
    let body = serde_json::json!({
        "full_name": "Test User",
        "email": email,
        "password": "password123",
        "confirm_password": "password123"
    });

    let response = app.post_register(&body).await;

    assert_eq!(201, response.status().as_u16());
}

#[tokio::test]
async fn login_works_with_valid_credentials_and_sets_cookie() {
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

    // 3. Check Cookie
    // Note: reqwest cookie store handles cookies automatically, but we can verify access to protected route
    // to confirm cookie is working
    let stats_response = app.get_dashboard_stats().await;
    assert_eq!(200, stats_response.status().as_u16());
}

#[tokio::test]
async fn login_fails_with_invalid_password() {
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

    // 2. Login with wrong password
    let login_body = serde_json::json!({
        "email": email,
        "password": "wrongpassword"
    });
    let response = app.post_login(&login_body).await;

    assert_eq!(401, response.status().as_u16());
}
