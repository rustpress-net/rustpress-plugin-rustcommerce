//! Middleware for RustCommerce
//!
//! Authentication extraction and rate limiting types.
//! In a full RustPress integration, the auth middleware would
//! delegate to rustpress-auth. This provides standalone types.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents an authenticated user extracted from the request.
///
/// In production, this is populated from a JWT or session token
/// validated by the RustPress auth system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub email: String,
    pub roles: Vec<String>,
}

impl AuthUser {
    /// Check if the user has a specific role.
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    /// Check if the user is an admin.
    pub fn is_admin(&self) -> bool {
        self.has_role("admin") || self.has_role("super_admin")
    }

    /// Check if the user is a store manager.
    pub fn is_manager(&self) -> bool {
        self.is_admin() || self.has_role("store_manager")
    }
}

/// Extractor that requires a valid authenticated user.
///
/// Usage in handlers:
/// ```ignore
/// async fn protected_handler(user: RequireAuth) -> impl IntoResponse {
///     // user.0 is the AuthUser
/// }
/// ```
#[derive(Debug, Clone)]
pub struct RequireAuth(pub AuthUser);

impl<S: Send + Sync> FromRequestParts<S> for RequireAuth {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract the Authorization header
        let auth_header = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok());

        match auth_header {
            Some(header) if header.starts_with("Bearer ") => {
                let token = &header[7..];

                // In production, validate JWT and extract claims.
                // For standalone mode, we decode a simple JSON payload
                // or reject if the token is invalid.
                let user = decode_auth_token(token).map_err(|_| {
                    (StatusCode::UNAUTHORIZED, "Invalid or expired token")
                })?;

                Ok(RequireAuth(user))
            }
            _ => Err((StatusCode::UNAUTHORIZED, "Missing authorization header")),
        }
    }
}

/// Extractor that requires admin role.
#[derive(Debug, Clone)]
pub struct RequireAdmin(pub AuthUser);

impl<S: Send + Sync> FromRequestParts<S> for RequireAdmin {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let RequireAuth(user) = RequireAuth::from_request_parts(parts, state).await?;

        if !user.is_admin() {
            return Err((StatusCode::FORBIDDEN, "Admin access required"));
        }

        Ok(RequireAdmin(user))
    }
}

/// Decode an authentication token into an AuthUser.
///
/// In a full integration, this validates a JWT against the RustPress
/// auth service. Standalone mode accepts a base64-encoded JSON payload
/// for development/testing purposes.
fn decode_auth_token(token: &str) -> std::result::Result<AuthUser, &'static str> {
    // Attempt to decode as base64 JSON (development mode).
    // In production, this would be JWT validation via rustpress-auth.
    use std::io::Read;

    let decoded = base64_decode(token).map_err(|_| "Invalid token encoding")?;
    let json_str = std::str::from_utf8(&decoded).map_err(|_| "Invalid token encoding")?;
    let user: AuthUser = serde_json::from_str(json_str).map_err(|_| "Invalid token payload")?;

    Ok(user)
}

/// Simple base64 decoder (no external crate needed).
fn base64_decode(input: &str) -> std::result::Result<Vec<u8>, &'static str> {
    const TABLE: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let input = input.trim_end_matches('=');
    let mut output = Vec::with_capacity(input.len() * 3 / 4);
    let mut buf: u32 = 0;
    let mut bits: u32 = 0;

    for &byte in input.as_bytes() {
        let val = TABLE
            .iter()
            .position(|&c| c == byte)
            .ok_or("Invalid base64 character")? as u32;
        buf = (buf << 6) | val;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            output.push((buf >> bits) as u8);
            buf &= (1 << bits) - 1;
        }
    }

    Ok(output)
}

/// Rate limiting configuration.
///
/// Defines per-route rate limits. In production, this would integrate
/// with tower middleware for enforcement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum requests per window.
    pub max_requests: u32,
    /// Window duration in seconds.
    pub window_seconds: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window_seconds: 60,
        }
    }
}

/// Predefined rate limits for different endpoint groups.
pub struct RateLimits;

impl RateLimits {
    /// Standard API endpoints.
    pub fn standard() -> RateLimitConfig {
        RateLimitConfig {
            max_requests: 100,
            window_seconds: 60,
        }
    }

    /// Checkout endpoint (stricter).
    pub fn checkout() -> RateLimitConfig {
        RateLimitConfig {
            max_requests: 10,
            window_seconds: 60,
        }
    }

    /// Webhook endpoints (generous, since they come from Stripe).
    pub fn webhook() -> RateLimitConfig {
        RateLimitConfig {
            max_requests: 500,
            window_seconds: 60,
        }
    }

    /// Admin endpoints.
    pub fn admin() -> RateLimitConfig {
        RateLimitConfig {
            max_requests: 200,
            window_seconds: 60,
        }
    }
}
