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
        "🔵 Incoming: {} {} | Request ID: {}",
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
                info!("📦 Request Body: {}", body_str);
                
                // Pretty print JSON if valid
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(body_str) {
                    if let Ok(pretty) = serde_json::to_string_pretty(&json_value) {
                        info!("📋 Parsed Body:\n{}", pretty);
                    }
                }
            }
        }

        // Reconstruct request with body
        let new_body = Body::from(bytes);
        let mut req = Request::from_parts(parts, new_body);
        
        // Add request ID to extensions
        req.extensions_mut().insert(request_id);
        
        // Process request
        let response = next.run(req).await;
        
        log_response(&method, &path, request_id, start, &response);
        
        response
    } else {
        // For GET/DELETE, just pass through
        let mut req = Request::from_parts(parts, body);
        req.extensions_mut().insert(request_id);
        
        let response = next.run(req).await;
        
        log_response(&method, &path, request_id, start, &response);
        
        response
    }
}

/// Log response with appropriate emoji based on status code
fn log_response(
    method: &axum::http::Method,
    path: &str,
    request_id: Uuid,
    start: Instant,
    response: &Response,
) {
    let duration = start.elapsed();
    let status = response.status();

    if status.is_success() {
        info!(
            "✅ Response: {} {} | Status: {} | Duration: {:?} | Request ID: {}",
            method, path, status.as_u16(), duration, request_id
        );
    } else if status.is_client_error() {
        warn!(
            "❌ Response: {} {} | Status: {} | Duration: {:?} | Request ID: {}",
            method, path, status.as_u16(), duration, request_id
        );
    } else if status.is_server_error() {
        error!(
            "🔥 Response: {} {} | Status: {} | Duration: {:?} | Request ID: {}",
            method, path, status.as_u16(), duration, request_id
        );
    } else {
        info!(
            "ℹ️  Response: {} {} | Status: {} | Duration: {:?} | Request ID: {}",
            method, path, status.as_u16(), duration, request_id
        );
    }
}

/// Extract request ID from request extensions
pub fn get_request_id(req: &Request) -> Option<Uuid> {
    req.extensions().get::<Uuid>().copied()
}
