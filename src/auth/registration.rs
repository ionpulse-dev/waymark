//! How a client identifies itself. Three methods, each in its own function, so that one can change
//! with no effect on the others (see `plan.md`, section 11):
//!
//! 1. a client document at a URL (the preferred method of the MCP standard),
//! 2. automatic registration, for older clients (the client is a signed value, nothing is stored),
//! 3. one manual client ID and secret, for apps that can do neither.

use super::tokens::{self, Claims};
use super::Door;
use serde_json::{json, Value};
use std::net::{IpAddr, ToSocketAddrs};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientKind {
    Document,
    Registered,
    Manual,
}

#[derive(Debug, Clone)]
pub struct Client {
    pub id: String,
    pub kind: ClientKind,
    /// The redirect addresses that the client may use. `None` for the manual client.
    pub redirect_uris: Option<Vec<String>>,
}

/// Redirect addresses must be HTTPS or on this computer (MCP standard).
pub fn redirect_ok(uri: &str) -> bool {
    if uri.contains('#') || uri.chars().any(|c| c.is_whitespace()) {
        return false;
    }
    if uri.starts_with("https://") {
        return true;
    }
    ["http://localhost", "http://127.0.0.1", "http://[::1]"]
        .iter()
        .any(|p| {
            uri.strip_prefix(p)
                .is_some_and(|r| r.is_empty() || r.starts_with(':') || r.starts_with('/'))
        })
}

/// Find the client for a `client_id`. Blocking: a client document is fetched.
pub fn resolve(door: &Door, client_id: &str) -> Result<Client, String> {
    let (manual_id, _) = tokens::manual_client(&door.settings.signing_key);
    if client_id == manual_id {
        return Ok(Client {
            id: manual_id,
            kind: ClientKind::Manual,
            redirect_uris: None,
        });
    }
    if client_id.starts_with("https://") || client_id.starts_with("http://") {
        return from_document(door, client_id);
    }
    let claims = tokens::verify(&door.settings.signing_key, client_id, tokens::TYPE_CLIENT)
        .map_err(|_| "The client is not known to this server.".to_string())?;
    Ok(Client {
        id: client_id.to_string(),
        kind: ClientKind::Registered,
        redirect_uris: Some(claims.uris),
    })
}

/// Is this redirect address allowed for the client?
pub fn check_redirect(client: &Client, redirect_uri: &str) -> Result<(), String> {
    if !redirect_ok(redirect_uri) {
        return Err("The redirect address must use https, or be on localhost.".into());
    }
    match &client.redirect_uris {
        None => Ok(()),
        Some(list) if list.iter().any(|u| u == redirect_uri) => Ok(()),
        Some(_) => Err("The redirect address is not one of the client's addresses.".into()),
    }
}

// ---- 1. client document -------------------------------------------------

fn private_address(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v) => {
            v.is_loopback()
                || v.is_private()
                || v.is_link_local()
                || v.is_unspecified()
                || v.is_broadcast()
                || (v.octets()[0] == 100 && (64..128).contains(&v.octets()[1]))
        }
        IpAddr::V6(v) => {
            v.is_loopback()
                || v.is_unspecified()
                || (v.segments()[0] & 0xfe00) == 0xfc00
                || (v.segments()[0] & 0xffc0) == 0xfe80
        }
    }
}

/// Refuse a document address that points into the network of the server (SSRF).
fn check_fetch_address(url: &str, allow_local: bool) -> Result<(), String> {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| allow_local.then(|| url.strip_prefix("http://")).flatten())
        .ok_or("The client ID must be an https address.")?;
    let authority = rest.split(['/', '?']).next().unwrap_or("");
    if authority.contains('@') {
        return Err("The client ID must not hold a user name.".into());
    }
    if allow_local {
        return Ok(());
    }
    // Add the default port when the address has none (an IPv6 address ends with `]` then).
    let has_port = authority
        .rsplit_once(':')
        .is_some_and(|(_, p)| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()));
    let host_port = if has_port {
        authority.to_string()
    } else {
        format!("{authority}:443")
    };
    let addrs = host_port
        .to_socket_addrs()
        .map_err(|_| "The client address cannot be resolved.".to_string())?;
    for a in addrs {
        if private_address(a.ip()) {
            return Err("The client address points to a private network.".into());
        }
    }
    Ok(())
}

fn from_document(door: &Door, url: &str) -> Result<Client, String> {
    let local = door.settings.is_local();
    let path_ok = url
        .split_once("://")
        .map(|(_, r)| r.contains('/') && !r.ends_with('/'))
        .unwrap_or(false);
    if !path_ok {
        return Err("The client ID must be an address with a path.".into());
    }
    check_fetch_address(url, local)?;
    let reply = crate::http::client::get_json(url, None)
        .map_err(|_| "The client document cannot be fetched.".to_string())?;
    if reply.status != 200 {
        return Err("The client document cannot be fetched.".into());
    }
    let doc = reply.json;
    // The `client_id` of the document must be the address, exactly.
    if doc.get("client_id").and_then(|v| v.as_str()) != Some(url) {
        return Err("The client document does not name its own address as client_id.".into());
    }
    if doc.get("client_name").and_then(|v| v.as_str()).is_none() {
        return Err("The client document has no client_name.".into());
    }
    let uris: Vec<String> = doc
        .get("redirect_uris")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|u| u.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    if uris.is_empty() || !uris.iter().all(|u| redirect_ok(u)) {
        return Err("The client document has no valid redirect_uris.".into());
    }
    Ok(Client {
        id: url.to_string(),
        kind: ClientKind::Document,
        redirect_uris: Some(uris),
    })
}

// ---- 2. automatic registration -----------------------------------------

/// RFC 7591. The client ID is a signed value that holds the redirect addresses. It has no secret.
pub fn register(door: &Door, body: &Value) -> Result<Value, (String, String)> {
    let bad = |m: &str| ("invalid_client_metadata".to_string(), m.to_string());
    let uris: Vec<String> = body
        .get("redirect_uris")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|u| u.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    if uris.is_empty() || uris.len() > 10 {
        return Err(bad("redirect_uris must hold 1 to 10 addresses."));
    }
    if !uris.iter().all(|u| redirect_ok(u)) {
        return Err((
            "invalid_redirect_uri".into(),
            "Redirect addresses must use https, or be on localhost.".into(),
        ));
    }
    // A registered client does not expire for as long as the signing key stays. The client ID is
    // the signed value.
    let issued = tokens::now();
    let claims = Claims {
        typ: tokens::TYPE_CLIENT.into(),
        exp: issued + 10 * 365 * 24 * 3600,
        uris: uris.clone(),
        ..Claims::default()
    };
    let client_id = tokens::sign(&door.settings.signing_key, &claims);
    Ok(json!({
        "client_id": client_id,
        "client_id_issued_at": issued,
        "redirect_uris": uris,
        "client_name": body.get("client_name").and_then(|v| v.as_str()).unwrap_or("MCP client")
            .chars().take(100).collect::<String>(),
        "grant_types": ["authorization_code", "refresh_token"],
        "response_types": ["code"],
        "token_endpoint_auth_method": "none"
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redirect_rules() {
        assert!(redirect_ok("https://app.example/cb"));
        assert!(redirect_ok("http://localhost:3000/cb"));
        assert!(redirect_ok("http://127.0.0.1/cb"));
        assert!(redirect_ok("http://localhost"));
        assert!(!redirect_ok("http://example.com/cb"));
        assert!(!redirect_ok("http://localhost.evil.example/cb"));
        assert!(!redirect_ok("myapp://cb"));
        assert!(!redirect_ok("https://a.example/cb#frag"));
    }

    #[test]
    fn private_networks_are_refused() {
        assert!(check_fetch_address("https://127.0.0.1/x", false).is_err());
        assert!(check_fetch_address("https://10.0.0.5/x", false).is_err());
        assert!(check_fetch_address("https://169.254.169.254/x", false).is_err());
        assert!(check_fetch_address("http://example.com/x", false).is_err());
        assert!(check_fetch_address("https://user@example.com/x", false).is_err());
    }
}
