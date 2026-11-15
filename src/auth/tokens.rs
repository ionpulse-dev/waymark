//! Signed codes and tokens, with no stored state (see `specs/foundation/personal-server`).
//!
//! A token is `payload.signature`. The payload is JSON in base64url. The signature is HMAC-SHA256
//! with the signing key from the environment. The server checks the signature, the type, the
//! expiry, and (for access tokens) the audience. It looks nothing up. A new signing key makes
//! every token invalid.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use hmac::{Hmac, KeyInit, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// How long each kind of value lives, in seconds. All of them are settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lifetimes {
    /// Authorization code (and the sign-in transaction).
    pub code: u64,
    pub access: u64,
    pub renewal: u64,
}

impl Default for Lifetimes {
    fn default() -> Self {
        Lifetimes {
            code: 300,
            access: 3600,
            renewal: 30 * 24 * 3600,
        }
    }
}

pub const TYPE_CODE: &str = "code";
pub const TYPE_ACCESS: &str = "access";
pub const TYPE_RENEWAL: &str = "renewal";
pub const TYPE_TRANSACTION: &str = "txn";
pub const TYPE_CLIENT: &str = "client";

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Claims {
    /// The kind of value. A value of one kind is never accepted as another.
    pub typ: String,
    pub exp: u64,
    /// The server (resource) that the token is for.
    #[serde(default)]
    pub aud: String,
    /// The account that signed in.
    #[serde(default)]
    pub sub: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub email: String,
    /// The client ID.
    #[serde(default)]
    pub cid: String,
    #[serde(default)]
    pub redirect: String,
    #[serde(default)]
    pub challenge: String,
    /// The `state` of the client.
    #[serde(default)]
    pub state: String,
    /// The redirect addresses of a registered client.
    #[serde(default)]
    pub uris: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenError {
    Malformed,
    BadSignature,
    Expired,
    WrongType,
}

pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn mac(key: &str) -> HmacSha256 {
    HmacSha256::new_from_slice(key.as_bytes()).expect("HMAC accepts any key length")
}

pub fn sign(key: &str, claims: &Claims) -> String {
    let payload = URL_SAFE_NO_PAD.encode(serde_json::to_vec(claims).unwrap_or_default());
    let mut m = mac(key);
    m.update(payload.as_bytes());
    format!(
        "{payload}.{}",
        URL_SAFE_NO_PAD.encode(m.finalize().into_bytes())
    )
}

/// Check the signature, the type, and the expiry at the time `at`.
pub fn verify_at(key: &str, token: &str, typ: &str, at: u64) -> Result<Claims, TokenError> {
    let (payload, sig) = token.split_once('.').ok_or(TokenError::Malformed)?;
    let sig = URL_SAFE_NO_PAD
        .decode(sig)
        .map_err(|_| TokenError::Malformed)?;
    let mut m = mac(key);
    m.update(payload.as_bytes());
    // `verify_slice` compares in constant time.
    m.verify_slice(&sig).map_err(|_| TokenError::BadSignature)?;
    let bytes = URL_SAFE_NO_PAD
        .decode(payload)
        .map_err(|_| TokenError::Malformed)?;
    let claims: Claims = serde_json::from_slice(&bytes).map_err(|_| TokenError::Malformed)?;
    if claims.typ != typ {
        return Err(TokenError::WrongType);
    }
    if claims.exp <= at {
        return Err(TokenError::Expired);
    }
    Ok(claims)
}

pub fn verify(key: &str, token: &str, typ: &str) -> Result<Claims, TokenError> {
    verify_at(key, token, typ, now())
}

/// The ID and the secret of the one manual client, derived from the signing key.
pub fn manual_client(key: &str) -> (String, String) {
    let derive = |label: &str| {
        let mut m = mac(key);
        m.update(label.as_bytes());
        URL_SAFE_NO_PAD.encode(m.finalize().into_bytes())
    };
    let id = format!("waymark-{}", &derive("manual-client-id")[..16]);
    (id, derive("manual-client-secret"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: &str = "0123456789abcdef0123456789abcdef";

    fn claims(typ: &str, exp: u64) -> Claims {
        Claims {
            typ: typ.into(),
            exp,
            aud: "https://x.example/mcp".into(),
            sub: "owner".into(),
            ..Claims::default()
        }
    }

    #[test]
    fn a_valid_token_checks_out() {
        let t = sign(KEY, &claims(TYPE_ACCESS, now() + 60));
        let c = verify(KEY, &t, TYPE_ACCESS).unwrap();
        assert_eq!(c.sub, "owner");
        assert_eq!(c.aud, "https://x.example/mcp");
    }

    #[test]
    fn an_expired_token_is_refused() {
        let t = sign(KEY, &claims(TYPE_ACCESS, 1000));
        assert_eq!(
            verify_at(KEY, &t, TYPE_ACCESS, 1000),
            Err(TokenError::Expired)
        );
        assert!(verify_at(KEY, &t, TYPE_ACCESS, 999).is_ok());
    }

    #[test]
    fn another_key_is_refused() {
        let t = sign(KEY, &claims(TYPE_ACCESS, now() + 60));
        assert_eq!(
            verify("another-key-another-key-another!", &t, TYPE_ACCESS),
            Err(TokenError::BadSignature)
        );
    }

    #[test]
    fn a_value_of_another_type_is_refused() {
        let t = sign(KEY, &claims(TYPE_CODE, now() + 60));
        assert_eq!(verify(KEY, &t, TYPE_ACCESS), Err(TokenError::WrongType));
        let r = sign(KEY, &claims(TYPE_RENEWAL, now() + 60));
        assert_eq!(verify(KEY, &r, TYPE_ACCESS), Err(TokenError::WrongType));
    }

    #[test]
    fn an_altered_payload_is_refused() {
        let t = sign(KEY, &claims(TYPE_ACCESS, now() + 60));
        let (_, sig) = t.split_once('.').unwrap();
        let forged = URL_SAFE_NO_PAD
            .encode(serde_json::to_vec(&claims(TYPE_ACCESS, now() + 99999)).unwrap());
        assert_eq!(
            verify(KEY, &format!("{forged}.{sig}"), TYPE_ACCESS),
            Err(TokenError::BadSignature)
        );
    }

    #[test]
    fn garbage_is_malformed() {
        assert_eq!(verify(KEY, "abc", TYPE_ACCESS), Err(TokenError::Malformed));
        assert_eq!(verify(KEY, "a.!!", TYPE_ACCESS), Err(TokenError::Malformed));
    }

    #[test]
    fn the_manual_client_depends_on_the_key() {
        let a = manual_client(KEY);
        assert_eq!(a, manual_client(KEY));
        assert_ne!(a, manual_client("another-key-another-key-another!"));
        assert!(a.0.starts_with("waymark-"));
    }
}
