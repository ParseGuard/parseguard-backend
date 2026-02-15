use axum::{
    body::Body,
    extract::Request,
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use tracing::{info, warn, error};
use uuid::Uuid;

/// Custom header for request ID
pub const X_REQUEST_ID: &str = "x-request-id";

/// Logger middleware
/// Logs incoming requests with method, path, request body, and request ID
pub async fn logger_middleware(req: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = req.method().clone();
    let uri = req.uri().clone();
    let path = uri.path().to_string();
    let request_id = Uuid::new_v4();

    // Log request start with emoji
    info!(
        "\n🚀 INCOMING REQUEST 🚀\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\nMethod: {}\nPath:   {}\nID:     {}\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
        method,
        path,
        request_id
    );

    // Extract and log body for POST/PUT/PATCH requests
    let (parts, body) = req.into_parts();
    
    if method == "POST" || method == "PUT" || method == "PATCH" {
        // Read the body bytes
        let bytes = match axum::body::to_bytes(body, usize::MAX).await {
            Ok(bytes) => bytes,
            Err(err) => {
                error!("❌ Failed to read request body: {}", err);
                axum::body::Bytes::new()
            }
        };

        // Try to log as JSON
        if !bytes.is_empty() {
            if let Ok(body_str) = std::str::from_utf8(&bytes) {
                // Pretty print JSON if valid
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(body_str) {
                    if let Ok(pretty) = serde_json::to_string_pretty(&json_value) {
                        info!("📦 REQUEST BODY:\n{}", pretty);
                    } else {
                        info!("📦 REQUEST BODY: {}", body_str);
                    }
                } else {
                    info!("📦 REQUEST BODY: {}", body_str);
                }
            }
        }

        // Reconstruct request with body
        let new_body = Body::from(bytes);
        let req = Request::from_parts(parts, new_body);
        
        // Process request
        let response = next.run(req).await;
        
        // Log response for this path
        log_response_internal(&method, &path, request_id, start, &response);
        
        response
    } else {
        // For other methods, process request directly without reading body
        let req = Request::from_parts(parts, body);
        let response = next.run(req).await;
        
        log_response_internal(&method, &path, request_id, start, &response);
        
        response
    }
}

/// Internal helper to log responses (to handle both main path and body logging path)
fn log_response_internal(
    method: &axum::http::Method,
    path: &str,
    request_id: Uuid,
    start: Instant,
    response: &Response,
) {
    let duration = start.elapsed();
    let status = response.status();

    let icon = if status.is_success() { "✅" } else { "❌" };
    let color_line = if status.is_success() { "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" } else { "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx" };

    info!(
        "\n{} RESPONSE SENDING {}\n{}\nMethod:   {}\nPath:     {}\nStatus:   {} ({})\nDuration: {:?}\nID:       {}\n{}",
        icon, icon,
        color_line,
        method,
        path,
        status.as_u16(),
        status,
        duration,
        request_id,
        color_line
    );
}

/// Extract request ID from request extensions
pub fn get_request_id(req: &Request) -> Option<Uuid> {
    req.extensions().get::<Uuid>().copied()
}
