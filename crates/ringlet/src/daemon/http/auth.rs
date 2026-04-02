//! HTTP authentication middleware using bearer tokens.

use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::Arc;
use subtle::ConstantTimeEq;
use tracing::{debug, warn};

/// Authenticated user's token hash (injected into request extensions).
/// Used for session ownership verification.
#[derive(Clone)]
pub struct AuthenticatedTokenHash(pub String);

/// Hash a token for ownership tracking (not for authentication).
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Length of the generated auth token in bytes (32 bytes = 256 bits).
const TOKEN_LENGTH: usize = 32;

/// Generate a new random authentication token.
/// Returns an error if the system's random number generator fails.
pub fn generate_token() -> Result<String, std::io::Error> {
    use std::fmt::Write;
    let mut bytes = [0u8; TOKEN_LENGTH];
    getrandom::getrandom(&mut bytes).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, format!("RNG failed: {}", e))
    })?;
    let mut hex = String::with_capacity(TOKEN_LENGTH * 2);
    for byte in bytes {
        // write! to a String cannot fail
        let _ = write!(hex, "{:02x}", byte);
    }
    Ok(hex)
}

/// Get the path to the token file.
pub fn token_file_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ringlet");
    config_dir.join("http_token")
}

/// Save token to file with restricted permissions.
pub fn save_token(token: &str) -> std::io::Result<()> {
    let path = token_file_path();

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Write token
    std::fs::write(&path, token)?;

    // Set permissions to user-only on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }

    Ok(())
}

/// Load token from file.
pub fn load_token() -> std::io::Result<String> {
    let path = token_file_path();
    std::fs::read_to_string(path).map(|s| s.trim().to_string())
}

/// State for the auth middleware.
#[derive(Clone)]
pub struct AuthState {
    pub token: Arc<String>,
}

/// Authentication middleware - validates bearer token using constant-time comparison.
pub async fn auth_middleware(
    State(auth): State<AuthState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract token from Authorization header only (no query params for security)
    let token = extract_token(&request);

    match token {
        Some(t) => {
            // Use constant-time comparison to prevent timing attacks
            let token_bytes = t.as_bytes();
            let expected_bytes = auth.token.as_bytes();

            // Length check plus constant-time content comparison
            if token_bytes.len() == expected_bytes.len()
                && bool::from(token_bytes.ct_eq(expected_bytes))
            {
                debug!("Request authenticated successfully");

                // Inject token hash into request extensions for session ownership tracking
                let token_hash = hash_token(t);
                request.extensions_mut().insert(AuthenticatedTokenHash(token_hash));

                Ok(next.run(request).await)
            } else {
                warn!("Invalid authentication token");
                Err(StatusCode::UNAUTHORIZED)
            }
        }
        None => {
            warn!("Missing authentication token");
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

/// Extract token from request Authorization header.
///
/// SECURITY: Only accepts tokens via Authorization header, not query parameters.
/// Query parameters are logged in access logs, browser history, and Referer headers,
/// making them unsuitable for sensitive credentials.
///
/// For WebSocket connections, clients should use the Sec-WebSocket-Protocol header
/// or establish an authenticated session before upgrading.
fn extract_token(request: &Request) -> Option<&str> {
    // Only accept Authorization header - query params are insecure
    if let Some(auth_header) = request.headers().get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                return Some(token);
            }
        }
    }

    // Also check Sec-WebSocket-Protocol for WebSocket upgrades
    // Format: "bearer, <token>" or just the token in first position
    if let Some(protocol_header) = request.headers().get("sec-websocket-protocol") {
        if let Ok(protocol_str) = protocol_header.to_str() {
            // Handle "bearer, <token>" format
            let parts: Vec<&str> = protocol_str.split(',').map(|s| s.trim()).collect();
            if parts.len() >= 2 && parts[0].to_lowercase() == "bearer" {
                return Some(parts[1]);
            }
        }
    }

    None
}
