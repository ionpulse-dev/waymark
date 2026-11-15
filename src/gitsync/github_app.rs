//! Background Git access through a GitHub App (see `specs/foundation/personal-server`).
//!
//! The server signs a short statement with the private key of the App. GitHub returns a token for
//! the one repository where the App is installed. The token is kept in memory only, and made again
//! when it is old or when Git says that it was refused. Nothing is stored.

use super::cmd::CredentialProvider;
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use serde::Serialize;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// A token is made again after this time. GitHub gives about an hour (assumed, see `decisions.md`).
const MAX_AGE: Duration = Duration::from_secs(45 * 60);

pub struct GithubApp {
    pub app_id: String,
    pub installation_id: String,
    key_pem: String,
    api_base: String,
    cached: Mutex<Option<(String, Instant)>>,
}

#[derive(Serialize)]
struct AppClaims {
    iat: u64,
    exp: u64,
    iss: String,
}

impl GithubApp {
    pub fn new(app_id: &str, installation_id: &str, key_pem: &str, api_base: &str) -> GithubApp {
        GithubApp {
            app_id: app_id.to_string(),
            installation_id: installation_id.to_string(),
            // An environment variable often holds a key with `\n` in place of the line breaks.
            key_pem: key_pem.replace("\\n", "\n"),
            api_base: api_base.trim_end_matches('/').to_string(),
            cached: Mutex::new(None),
        }
    }

    /// Check that the private key can sign. Used at start, so that a wrong key stops the server.
    pub fn check_key(&self) -> Result<(), String> {
        self.statement().map(|_| ())
    }

    fn statement(&self) -> Result<String, String> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let claims = AppClaims {
            iat: now.saturating_sub(60),
            exp: now + 9 * 60,
            iss: self.app_id.clone(),
        };
        let key = EncodingKey::from_rsa_pem(self.key_pem.as_bytes()).map_err(|_| {
            "The GitHub App private key is not a valid RSA key in PEM form.".to_string()
        })?;
        jsonwebtoken::encode(&Header::new(Algorithm::RS256), &claims, &key)
            .map_err(|_| "Cannot sign with the GitHub App private key.".to_string())
    }

    fn make_token(&self) -> Result<String, String> {
        let statement = self.statement()?;
        let url = format!(
            "{}/app/installations/{}/access_tokens",
            self.api_base, self.installation_id
        );
        let reply = crate::http::client::post_empty(&url, &statement)
            .map_err(|e| format!("Cannot reach GitHub: {e}"))?;
        if reply.status != 201 && reply.status != 200 {
            return Err(format!(
                "GitHub refused the App token request (status {}). Check the App ID, the installation number, and the private key.",
                reply.status
            ));
        }
        reply
            .json
            .get("token")
            .and_then(|t| t.as_str())
            .map(String::from)
            .ok_or_else(|| "GitHub gave no token.".to_string())
    }

    /// The current token. Made again if there is none or it is old.
    pub fn current(&self) -> Result<String, String> {
        let mut slot = self.cached.lock().unwrap();
        if let Some((t, at)) = slot.as_ref() {
            if at.elapsed() < MAX_AGE {
                return Ok(t.clone());
            }
        }
        let t = self.make_token()?;
        *slot = Some((t.clone(), Instant::now()));
        Ok(t)
    }
}

impl CredentialProvider for GithubApp {
    fn token(&self) -> Option<String> {
        self.current().ok()
    }

    fn invalidate(&self) {
        *self.cached.lock().unwrap() = None;
    }
}
