//! The metadata that tells a client where and how to sign in (MCP authorization, revision
//! 2026-07-28: RFC 9728 and RFC 8414).

use super::Door;
use serde_json::{json, Value};

/// The protected resource metadata: this server and its authorization server.
pub fn resource_metadata(door: &Door) -> Value {
    json!({
        "resource": door.settings.resource(),
        "authorization_servers": [door.settings.public_url],
        "bearer_methods_supported": ["header"],
        "resource_name": "Waymark"
    })
}

/// The authorization server metadata.
pub fn server_metadata(door: &Door) -> Value {
    let base = &door.settings.public_url;
    json!({
        "issuer": base,
        "authorization_endpoint": format!("{base}/authorize"),
        "token_endpoint": format!("{base}/token"),
        "registration_endpoint": format!("{base}/register"),
        "response_types_supported": ["code"],
        "grant_types_supported": ["authorization_code", "refresh_token"],
        "code_challenge_methods_supported": ["S256"],
        "token_endpoint_auth_methods_supported": ["none", "client_secret_post", "client_secret_basic"],
        "client_id_metadata_document_supported": true,
        "authorization_response_iss_parameter_supported": true
    })
}

/// The value of the `WWW-Authenticate` header of a `401`.
pub fn challenge(door: &Door, invalid_token: bool) -> String {
    let base = &door.settings.public_url;
    let url = format!("{base}/.well-known/oauth-protected-resource/mcp");
    if invalid_token {
        format!("Bearer error=\"invalid_token\", resource_metadata=\"{url}\"")
    } else {
        format!("Bearer resource_metadata=\"{url}\"")
    }
}
