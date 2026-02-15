use common::spawn_app;

mod common;

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    
    let body = response.text().await.expect("Failed to read body");
    assert!(body.contains("healthy"));
}
