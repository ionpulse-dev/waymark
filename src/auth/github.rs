//! "Sign in with GitHub" as the identity provider (see `specs/foundation/personal-server`).
//! The GitHub token of the person is used once, to read the account and its emails, and dropped.

use super::identity::{EmailEntry, IdentityProvider, Person};
use crate::http::client;

pub struct Github {
    pub client_id: String,
    pub client_secret: String,
    /// `https://github.com`. A setting, so that tests can use a fake.
    pub web_base: String,
    /// `https://api.github.com`.
    pub api_base: String,
}

impl IdentityProvider for Github {
    fn authorize_url(&self, state: &str, callback: &str) -> String {
        format!(
            "{}/login/oauth/authorize?client_id={}&redirect_uri={}&state={}&allow_signup=false",
            self.web_base.trim_end_matches('/'),
            super::urlencode(&self.client_id),
            super::urlencode(callback),
            super::urlencode(state)
        )
    }

    fn identify(&self, code: &str, callback: &str) -> Result<Person, String> {
        let web = self.web_base.trim_end_matches('/');
        let api = self.api_base.trim_end_matches('/');
        let reply = client::post_form(
            &format!("{web}/login/oauth/access_token"),
            &[
                ("client_id", &self.client_id),
                ("client_secret", &self.client_secret),
                ("code", code),
                ("redirect_uri", callback),
            ],
        )?;
        let token = reply
            .json
            .get("access_token")
            .and_then(|t| t.as_str())
            .ok_or_else(|| "GitHub did not accept the sign-in.".to_string())?
            .to_string();
        let user = client::get_json(&format!("{api}/user"), Some(&token))?;
        if user.status != 200 {
            return Err("GitHub did not tell who signed in.".into());
        }
        let login = user
            .json
            .get("login")
            .and_then(|v| v.as_str())
            .ok_or("GitHub did not tell who signed in.")?
            .to_string();
        let name = user
            .json
            .get("name")
            .and_then(|v| v.as_str())
            .map(String::from);
        let emails = client::get_json(&format!("{api}/user/emails"), Some(&token))?;
        let list = if emails.status == 200 {
            emails.json.as_array().cloned().unwrap_or_default()
        } else {
            // Without the permission "Email addresses: read" the list is not readable.
            Vec::new()
        };
        let emails = list
            .iter()
            .filter_map(|e| {
                Some(EmailEntry {
                    address: e.get("email")?.as_str()?.to_string(),
                    verified: e.get("verified").and_then(|v| v.as_bool()).unwrap_or(false),
                    primary: e.get("primary").and_then(|v| v.as_bool()).unwrap_or(false),
                })
            })
            .collect();
        Ok(Person {
            login,
            name,
            emails,
        })
    }
}
