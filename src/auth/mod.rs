//! The sign-in door of the personal server (see `specs/foundation/personal-server`).
//!
//! An OAuth 2.1 authorization server in front of the MCP core. It follows the MCP authorization
//! standard (revision 2026-07-28). The server signs its own codes and tokens, so it keeps no state.
//! GitHub only answers "who is this person?". A GitHub token never reaches a client.

pub mod discovery;
pub mod github;
pub mod identity;
pub mod registration;
pub mod tokens;

use crate::http::settings::ServerSettings;
use crate::memrepo::settings::People;
use axum::extract::{Query, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Form, Json, Router};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use identity::IdentityProvider;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use subtle::ConstantTimeEq;
use tokens::Claims;

/// What the routes of the door need.
pub struct Door {
    pub settings: Arc<ServerSettings>,
    pub identity: Arc<dyn IdentityProvider>,
    /// The `[people]` rules that apply at sign-in. The memory repository may not be cloned yet.
    /// Then the rules of the repository are checked at the first write instead.
    pub people: Box<dyn Fn() -> People + Send + Sync>,
}

/// Percent-encode a value for a URL query.
pub fn urlencode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn no_store(mut r: Response) -> Response {
    let h = r.headers_mut();
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    h.insert(header::PRAGMA, HeaderValue::from_static("no-cache"));
    r
}

/// A small page for a person in a browser (a refusal or an error).
fn page(status: StatusCode, title: &str, text: &str) -> Response {
    let body = format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>Waymark</title><style>body{{font-family:system-ui,sans-serif;max-width:32rem;margin:3rem auto;padding:0 1rem;line-height:1.5}}</style></head><body><h1>{}</h1><p>{}</p></body></html>",
        escape_html(title),
        escape_html(text)
    );
    let mut r = (status, body).into_response();
    let h = r.headers_mut();
    h.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    h.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("default-src 'none'; style-src 'unsafe-inline'"),
    );
    h.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    no_store(r)
}

fn redirect(url: &str) -> Response {
    let mut r = StatusCode::FOUND.into_response();
    if let Ok(v) = HeaderValue::from_str(url) {
        r.headers_mut().insert(header::LOCATION, v);
    }
    no_store(r)
}

fn json_error(status: StatusCode, error: &str, description: &str) -> Response {
    no_store(
        (
            status,
            Json(json!({"error": error, "error_description": description})),
        )
            .into_response(),
    )
}

/// `resource` may name the server or its MCP address. Both mean the MCP resource.
fn resource_ok(door: &Door, resource: &str) -> bool {
    let r = resource.trim_end_matches('/');
    r == door.settings.public_url || r == door.settings.resource()
}

pub fn routes(door: Arc<Door>) -> Router {
    Router::new()
        .route(
            "/.well-known/oauth-protected-resource",
            get(protected_resource),
        )
        .route(
            "/.well-known/oauth-protected-resource/mcp",
            get(protected_resource),
        )
        .route(
            "/.well-known/oauth-authorization-server",
            get(server_metadata),
        )
        .route("/register", post(register))
        .route("/authorize", get(authorize))
        .route("/callback", get(callback))
        .route("/token", post(token))
        .with_state(door)
}

async fn protected_resource(State(d): State<Arc<Door>>) -> Response {
    Json(discovery::resource_metadata(&d)).into_response()
}

async fn server_metadata(State(d): State<Arc<Door>>) -> Response {
    Json(discovery::server_metadata(&d)).into_response()
}

async fn register(
    State(d): State<Arc<Door>>,
    body: Result<Json<Value>, axum::extract::rejection::JsonRejection>,
) -> Response {
    let Ok(Json(body)) = body else {
        return json_error(
            StatusCode::BAD_REQUEST,
            "invalid_client_metadata",
            "The body must be JSON.",
        );
    };
    match registration::register(&d, &body) {
        Ok(v) => no_store((StatusCode::CREATED, Json(v)).into_response()),
        Err((e, m)) => json_error(StatusCode::BAD_REQUEST, &e, &m),
    }
}

async fn authorize(
    State(d): State<Arc<Door>>,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    tokio::task::spawn_blocking(move || authorize_blocking(&d, &q))
        .await
        .unwrap_or_else(|_| {
            page(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error",
                "Something went wrong.",
            )
        })
}

fn authorize_blocking(d: &Door, q: &HashMap<String, String>) -> Response {
    let get = |k: &str| q.get(k).map(String::as_str).unwrap_or("");
    let bad = |m: &str| page(StatusCode::BAD_REQUEST, "Cannot sign in", m);
    let client = match registration::resolve(d, get("client_id")) {
        Ok(c) => c,
        Err(m) => return bad(&m),
    };
    if let Err(m) = registration::check_redirect(&client, get("redirect_uri")) {
        return bad(&m);
    }
    if get("response_type") != "code" {
        return bad("response_type must be code.");
    }
    let challenge = get("code_challenge");
    if get("code_challenge_method") != "S256" || !(43..=128).contains(&challenge.len()) {
        return bad("PKCE with code_challenge_method=S256 is required.");
    }
    let resource = get("resource");
    if !resource.is_empty() && !resource_ok(d, resource) {
        return bad("The resource is not this server.");
    }
    let life = d.settings.lifetimes.code.max(600);
    let txn = tokens::sign(
        &d.settings.signing_key,
        &Claims {
            typ: tokens::TYPE_TRANSACTION.into(),
            exp: tokens::now() + life,
            aud: d.settings.resource(),
            cid: client.id,
            redirect: get("redirect_uri").to_string(),
            challenge: challenge.to_string(),
            state: get("state").to_string(),
            ..Claims::default()
        },
    );
    let callback = format!("{}/callback", d.settings.public_url);
    redirect(&d.identity.authorize_url(&txn, &callback))
}

async fn callback(
    State(d): State<Arc<Door>>,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    tokio::task::spawn_blocking(move || callback_blocking(&d, &q))
        .await
        .unwrap_or_else(|_| {
            page(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error",
                "Something went wrong.",
            )
        })
}

fn callback_blocking(d: &Door, q: &HashMap<String, String>) -> Response {
    let get = |k: &str| q.get(k).map(String::as_str).unwrap_or("");
    let Ok(txn) = tokens::verify(
        &d.settings.signing_key,
        get("state"),
        tokens::TYPE_TRANSACTION,
    ) else {
        return page(
            StatusCode::BAD_REQUEST,
            "Cannot sign in",
            "The sign-in took too long or did not start here. Start again from your app.",
        );
    };
    if !q.contains_key("code") || q.contains_key("error") {
        return page(
            StatusCode::BAD_REQUEST,
            "Cannot sign in",
            "The sign-in was not finished.",
        );
    }
    let callback = format!("{}/callback", d.settings.public_url);
    let person = match d.identity.identify(get("code"), &callback) {
        Ok(p) => p,
        Err(e) => {
            crate::log::event("sign_in", &[("outcome", "provider_error"), ("error", &e)]);
            return page(
                StatusCode::BAD_GATEWAY,
                "Cannot sign in",
                "The sign-in provider did not answer. Try again.",
            );
        }
    };
    if !person.login.eq_ignore_ascii_case(&d.settings.owner) {
        crate::log::event(
            "sign_in",
            &[("account", &person.login), ("outcome", "refused")],
        );
        return page(
            StatusCode::FORBIDDEN,
            "Not allowed",
            "This server is not for this account.",
        );
    }
    let chosen = match identity::choose(&person, &(d.people)()) {
        Ok(c) => c,
        Err(m) => {
            crate::log::event(
                "sign_in",
                &[("account", &person.login), ("outcome", "no_valid_email")],
            );
            return page(StatusCode::FORBIDDEN, "Cannot sign in", &m);
        }
    };
    crate::log::event("sign_in", &[("account", &person.login), ("outcome", "ok")]);
    let code = tokens::sign(
        &d.settings.signing_key,
        &Claims {
            typ: tokens::TYPE_CODE.into(),
            exp: tokens::now() + d.settings.lifetimes.code,
            aud: txn.aud,
            sub: person.login,
            name: chosen.name,
            email: chosen.email,
            cid: txn.cid,
            redirect: txn.redirect.clone(),
            challenge: txn.challenge,
            ..Claims::default()
        },
    );
    let sep = if txn.redirect.contains('?') { '&' } else { '?' };
    let mut url = format!(
        "{}{sep}code={}&iss={}",
        txn.redirect,
        urlencode(&code),
        urlencode(&d.settings.public_url)
    );
    if !txn.state.is_empty() {
        url.push_str(&format!("&state={}", urlencode(&txn.state)));
    }
    redirect(&url)
}

fn basic_credentials(headers: &HeaderMap) -> Option<(String, String)> {
    let v = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let b = v.strip_prefix("Basic ")?;
    let text = String::from_utf8(
        base64::engine::general_purpose::STANDARD
            .decode(b.trim())
            .ok()?,
    )
    .ok()?;
    let (id, secret) = text.split_once(':')?;
    Some((id.to_string(), secret.to_string()))
}

async fn token(
    State(d): State<Arc<Door>>,
    headers: HeaderMap,
    Form(form): Form<HashMap<String, String>>,
) -> Response {
    token_endpoint(&d, &headers, &form)
}

fn token_endpoint(d: &Door, headers: &HeaderMap, form: &HashMap<String, String>) -> Response {
    let get = |k: &str| form.get(k).map(String::as_str).unwrap_or("");
    let basic = basic_credentials(headers);
    let client_id = if !get("client_id").is_empty() {
        get("client_id").to_string()
    } else {
        basic.as_ref().map(|b| b.0.clone()).unwrap_or_default()
    };
    // The manual client has a secret. The other clients are public clients.
    let (manual_id, manual_secret) = tokens::manual_client(&d.settings.signing_key);
    if client_id == manual_id {
        let given = if !get("client_secret").is_empty() {
            get("client_secret").to_string()
        } else {
            basic.map(|b| b.1).unwrap_or_default()
        };
        if !bool::from(given.as_bytes().ct_eq(manual_secret.as_bytes())) {
            return json_error(
                StatusCode::UNAUTHORIZED,
                "invalid_client",
                "The client secret is wrong.",
            );
        }
    }
    let grant = |m: &str| json_error(StatusCode::BAD_REQUEST, "invalid_grant", m);
    let key = &d.settings.signing_key;
    let (base, with_renewal) = match get("grant_type") {
        "authorization_code" => {
            let Ok(c) = tokens::verify(key, get("code"), tokens::TYPE_CODE) else {
                return grant("The code is wrong or has expired.");
            };
            if c.cid != client_id {
                return grant("The code was issued to another client.");
            }
            if !get("redirect_uri").is_empty() && get("redirect_uri") != c.redirect {
                return grant("The redirect address does not match.");
            }
            let verifier = get("code_verifier");
            let hashed = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
            if !(43..=128).contains(&verifier.len())
                || !bool::from(hashed.as_bytes().ct_eq(c.challenge.as_bytes()))
            {
                return grant("The PKCE verifier does not match.");
            }
            (c, true)
        }
        "refresh_token" => {
            let Ok(c) = tokens::verify(key, get("refresh_token"), tokens::TYPE_RENEWAL) else {
                return grant("The renewal token is wrong or has expired.");
            };
            if c.cid != client_id {
                return grant("The renewal token was issued to another client.");
            }
            (c, false)
        }
        _ => {
            return json_error(
                StatusCode::BAD_REQUEST,
                "unsupported_grant_type",
                "Use authorization_code or refresh_token.",
            )
        }
    };
    if !get("resource").is_empty() && !resource_ok(d, get("resource")) {
        return json_error(
            StatusCode::BAD_REQUEST,
            "invalid_target",
            "The resource is not this server.",
        );
    }
    let life = d.settings.lifetimes;
    let now = tokens::now();
    let access = tokens::sign(
        key,
        &Claims {
            typ: tokens::TYPE_ACCESS.into(),
            exp: now + life.access,
            aud: base.aud.clone(),
            sub: base.sub.clone(),
            name: base.name.clone(),
            email: base.email.clone(),
            cid: base.cid.clone(),
            ..Claims::default()
        },
    );
    // The renewal token does not rotate. A renewal returns the same token (see `decisions.md`).
    let renewal = if with_renewal {
        tokens::sign(
            key,
            &Claims {
                typ: tokens::TYPE_RENEWAL.into(),
                exp: now + life.renewal,
                aud: base.aud,
                sub: base.sub,
                name: base.name,
                email: base.email,
                cid: base.cid,
                ..Claims::default()
            },
        )
    } else {
        get("refresh_token").to_string()
    };
    no_store(
        Json(json!({
            "access_token": access,
            "token_type": "Bearer",
            "expires_in": life.access,
            "refresh_token": renewal,
        }))
        .into_response(),
    )
}

/// Check the token of a request to the MCP path. `Err(true)` means a token was sent and is not
/// valid. `Err(false)` means that none was sent.
pub fn authenticate(door: &Door, headers: &HeaderMap) -> Result<Claims, bool> {
    let value = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or(false)?;
    let token = value
        .strip_prefix("Bearer ")
        .or_else(|| value.strip_prefix("bearer "))
        .ok_or(true)?;
    let c = tokens::verify(
        &door.settings.signing_key,
        token.trim(),
        tokens::TYPE_ACCESS,
    )
    .map_err(|_| true)?;
    if c.aud != door.settings.resource() {
        return Err(true);
    }
    Ok(c)
}
