use axum::{
    body::Body,
    extract::Request,
    http::{HeaderMap, HeaderValue},
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use tracing::{info, warn, error};
use uuid::Uuid;

/// Custom header for request ID
pub const X_REQUEST_ID: &str = "x-request-id";

/// Logger middleware
/// Logs incoming requests with method, path, headers, and body
pub async fn logger_middleware(req: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = req.method().clone();
    let uri = req.uri().clone();
    let path = uri.path().to_string();
    let query = uri.query().unwrap_or("");
    let request_id = Uuid::new_v4();

    // Log Request
    let mut log_message = String::new();
    log_message.push_str(&format!("\n🚀 INCOMING REQUEST 🚀\n"));
    log_message.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    log_message.push_str(&format!("Method:  {}\n", method));
    log_message.push_str(&format!("Path:    {}\n", path));
    if !query.is_empty() {
        log_message.push_str(&format!("Query:   {}\n", query));
    }
    log_message.push_str(&format!("ID:      {}\n", request_id));
    
    // Log Headers
    log_message.push_str("----------------------------------------------------\n");
    log_message.push_str("HEADERS:\n");
    for (name, value) in req.headers() {
        let val_str = value.to_str().unwrap_or("<binary>");
        // Redact specific sensitive headers if needed, but show Cookie for debugging
        if name == "authorization" {
             log_message.push_str(&format!("  {}: [REDACTED]\n", name));
        } else {
             log_message.push_str(&format!("  {}: {}\n", name, val_str));
        }
    }
    log_message.push_str("----------------------------------------------------\n");

    // Extract body
    let (parts, body) = req.into_parts();
    
    // Only log body for mutation methods or if debug is needed
    // Assuming JSON payloads for API
    let bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(err) => {
            error!("❌ Failed to read request body: {}", err);
            axum::body::Bytes::new()
        }
    };

    if !bytes.is_empty() {
        log_message.push_str("BODY:\n");
        if let Ok(body_str) = std::str::from_utf8(&bytes) {
             if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(body_str) {
                if let Ok(pretty) = serde_json::to_string_pretty(&json_value) {
                    log_message.push_str(&pretty);
                } else {
                    log_message.push_str(body_str);
                }
            } else {
                log_message.push_str(body_str);
            }
        } else {
            log_message.push_str("<binary body>");
        }
        log_message.push_str("\n");
    }
    log_message.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    info!("{}", log_message);

    // Reconstruct request
    let new_body = Body::from(bytes);
    let req = Request::from_parts(parts, new_body);

    // Process Request
    let response = next.run(req).await;

    // Log Response
    let duration = start.elapsed();
    let status = response.status();
    let icon = if status.is_success() { "✅" } else { "❌" };
    let border = if status.is_success() { "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" } else { "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx" };

    let mut res_log = String::new();
    res_log.push_str(&format!("\n{} RESPONSE COMPLETED {}\n", icon, icon));
    res_log.push_str(&format!("{}\n", border));
    res_log.push_str(&format!("Method:   {}\n", method));
    res_log.push_str(&format!("Path:     {}\n", path));
    res_log.push_str(&format!("Status:   {} ({})\n", status.as_u16(), status));
    res_log.push_str(&format!("Duration: {:?}\n", duration));
    res_log.push_str(&format!("ID:       {}\n", request_id));
    res_log.push_str(&format!("{}", border));

    info!("{}", res_log);

    response
}

/// Extract request ID
pub fn get_request_id(req: &Request) -> Option<Uuid> {
    req.extensions().get::<Uuid>().copied()
}
