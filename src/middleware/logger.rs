use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use uuid::Uuid;

/// Request ID header key
pub const X_REQUEST_ID: &str = "x-request-id";

/// Logger middleware for tracking requests
///
/// Adds request ID to each request and logs request/response details
///
/// # Arguments
///
/// * `req` - Incoming HTTP request
/// * `next` - Next middleware in chain
///
/// # Returns
///
/// HTTP response with request ID header
pub async fn logger_middleware(mut req: Request, next: Next) -> Response {
    let start = Instant::now();
    
    // Generate or extract request ID
    let request_id = req
        .headers()
        .get(X_REQUEST_ID)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or_else(Uuid::new_v4);

    let method = req.method().clone();
    let uri = req.uri().clone();
    let path = uri.path().to_string();

    // Log incoming request
    tracing::info!(
        request_id = %request_id,
        method = %method,
        path = %path,
        "Incoming request"
    );

    // Store request ID in extensions for handlers to access
    req.extensions_mut().insert(request_id);

    // Process request
    let response = next.run(req).await;
    
    // Calculate duration
    let duration = start.elapsed();
    let status = response.status();

    // Log response with appropriate level based on status
    if status.is_server_error() {
        tracing::error!(
            request_id = %request_id,
            method = %method,
            path = %path,
            status = %status.as_u16(),
            duration_ms = duration.as_millis(),
            "Request completed with server error"
        );
    } else if status.is_client_error() {
        tracing::warn!(
            request_id = %request_id,
            method = %method,
            path = %path,
            status = %status.as_u16(),
            duration_ms = duration.as_millis(),
            "Request completed with client error"
        );
    } else {
        tracing::info!(
            request_id = %request_id,
            method = %method,
            path = %path,
            status = %status.as_u16(),
            duration_ms = duration.as_millis(),
            "Request completed successfully"
        );
    }

    // Add request ID to response headers
    let (mut parts, body) = response.into_parts();
    parts.headers.insert(
        X_REQUEST_ID,
        request_id.to_string().parse().unwrap(),
    );

    Response::from_parts(parts, body)
}

/// Get request ID from request extensions
///
/// # Arguments
///
/// * `req` - HTTP request
///
/// # Returns
///
/// Request ID if present, None otherwise
pub fn get_request_id(req: &Request) -> Option<Uuid> {
    req.extensions().get::<Uuid>().copied()
}
