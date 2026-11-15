//! The settings of `personal-server` mode (see `specs/foundation/personal-server`).
//!
//! Every value comes from an environment variable, or from a file that `<NAME>_FILE` names
//! (a mounted secret). Waymark never writes a secret anywhere.

use crate::auth::tokens::Lifetimes;
use crate::mcp::args::Args;

/// The modes of the server (the values of `--mode`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Local,
    PersonalServer,
}

pub const MODES: &str = "local, personal-server, and team-server";

/// Read the mode from the value of `--mode` or `WAYMARK_MODE`.
pub fn parse_mode(value: Option<&str>) -> Result<Mode, String> {
    match value.map(str::trim) {
        None | Some("") | Some("local") => Ok(Mode::Local),
        Some("personal-server") => Ok(Mode::PersonalServer),
        Some("team-server") => Err(
            "The mode team-server is not available yet. The modes you can use now are local and personal-server."
                .to_string(),
        ),
        Some(other) => Err(format!(
            "Unknown mode \"{other}\". The modes are {MODES}."
        )),
    }
}

/// The text for `--print-client`: the manual client ID and secret. It needs only the signing key.
pub fn print_client(get: &dyn Fn(&str) -> Option<String>) -> Result<String, String> {
    let key = match get("WAYMARK_SIGNING_KEY_FILE").filter(|p| !p.is_empty()) {
        Some(path) => std::fs::read_to_string(&path)
            .map_err(|e| format!("Cannot read the file of WAYMARK_SIGNING_KEY_FILE: {e}"))?
            .trim_end()
            .to_string(),
        None => get("WAYMARK_SIGNING_KEY").unwrap_or_default(),
    };
    if key.len() < 32 {
        return Err(
            "Set WAYMARK_SIGNING_KEY (at least 32 characters) to the value of the server.".into(),
        );
    }
    let (id, secret) = crate::auth::tokens::manual_client(&key);
    Ok(format!("Client ID: {id}\nClient secret: {secret}"))
}

#[derive(Debug, Clone)]
pub struct ServerSettings {
    pub port: u16,
    /// `--listen`: an address that replaces `0.0.0.0:$PORT`.
    pub listen: Option<String>,
    /// The public address, without a trailing slash. It starts with `https://`.
    pub public_url: String,
    /// The Git URL of the memory repository.
    pub repo: String,
    /// The one GitHub account that may sign in.
    pub owner: String,
    pub signing_key: String,
    pub app_id: String,
    pub installation_id: String,
    pub app_key: String,
    pub client_id: String,
    pub client_secret: String,
    /// `https://api.github.com`. Tests use a fake.
    pub github_api: String,
    /// `https://github.com`. Tests use a fake.
    pub github_web: String,
    pub lifetimes: Lifetimes,
}

impl ServerSettings {
    /// The address of this server as a client sees the MCP resource.
    pub fn resource(&self) -> String {
        format!("{}/mcp", self.public_url)
    }

    /// True for an address on this computer. Tests use it. A real server is never local.
    pub fn is_local(&self) -> bool {
        let host = crate::mcp::repo_url::host(&self.public_url);
        matches!(host.as_str(), "localhost" | "127.0.0.1" | "[::1]" | "::1")
    }

    /// The URL prefix for which `git` gets the credential, such as `https://github.com/`.
    pub fn git_credential_prefix(&self) -> Option<String> {
        let (scheme, rest) = if let Some(r) = self.repo.strip_prefix("https://") {
            ("https", r)
        } else {
            ("http", self.repo.strip_prefix("http://")?)
        };
        let host = rest.split('/').next()?.rsplit('@').next()?;
        Some(format!("{scheme}://{host}/"))
    }

    pub fn from_env(args: &Args) -> Result<ServerSettings, String> {
        ServerSettings::from_lookup(args, &|name| std::env::var(name).ok())
    }

    pub fn from_lookup(
        args: &Args,
        get: &dyn Fn(&str) -> Option<String>,
    ) -> Result<ServerSettings, String> {
        // Settings of another mode, or that this mode does not use, stop the start.
        let mut refused: Vec<String> = Vec::new();
        if args.repo.is_some() {
            refused.push("--repo (use WAYMARK_REPO)".into());
        }
        if args.author_name.is_some() || args.author_email.is_some() {
            refused.push(
                "--author-name and --author-email (the author is the person who signs in)".into(),
            );
        }
        for var in [
            "WAYMARK_AUTHOR_NAME",
            "WAYMARK_AUTHOR_EMAIL",
            "WAYMARK_ROLES_FILE",
        ] {
            if get(var).is_some_and(|v| !v.is_empty()) {
                refused.push(var.to_string());
            }
        }
        if !refused.is_empty() {
            return Err(format!(
                "These settings are not allowed in personal-server mode: {}.",
                refused.join(", ")
            ));
        }

        let value = |name: &str| -> Result<Option<String>, String> {
            if let Some(path) = get(&format!("{name}_FILE")).filter(|p| !p.is_empty()) {
                let text = std::fs::read_to_string(&path)
                    .map_err(|e| format!("Cannot read the file of {name}_FILE: {e}"))?;
                return Ok(Some(text.trim_end().to_string()));
            }
            Ok(get(name).filter(|v| !v.trim().is_empty()))
        };
        let mut missing: Vec<&str> = Vec::new();
        let mut need = |name: &'static str| -> Result<String, String> {
            let v = value(name)?;
            if v.is_none() {
                missing.push(name);
            }
            Ok(v.unwrap_or_default())
        };
        let public_url = need("WAYMARK_PUBLIC_URL")?;
        let repo = need("WAYMARK_REPO")?;
        let owner = need("WAYMARK_OWNER")?;
        let signing_key = need("WAYMARK_SIGNING_KEY")?;
        let app_id = need("WAYMARK_GITHUB_APP_ID")?;
        let installation_id = need("WAYMARK_GITHUB_INSTALLATION_ID")?;
        let app_key = need("WAYMARK_GITHUB_APP_KEY")?;
        let client_id = need("WAYMARK_GITHUB_CLIENT_ID")?;
        let client_secret = need("WAYMARK_GITHUB_CLIENT_SECRET")?;
        if !missing.is_empty() {
            return Err(format!(
                "personal-server mode needs these settings: {}.",
                missing.join(", ")
            ));
        }

        let public_url = public_url.trim().trim_end_matches('/').to_string();
        let local = {
            let host = crate::mcp::repo_url::host(&public_url);
            matches!(host.as_str(), "localhost" | "127.0.0.1" | "[::1]" | "::1")
        };
        let ok_scheme =
            public_url.starts_with("https://") || (local && public_url.starts_with("http://"));
        if !ok_scheme {
            return Err(format!(
                "WAYMARK_PUBLIC_URL must start with https:// (a localhost address may use http://). It is \"{}\".",
                crate::log::clean(&public_url)
            ));
        }
        if signing_key.len() < 32 {
            return Err("WAYMARK_SIGNING_KEY must have at least 32 characters.".into());
        }

        let number = |name: &str, default: u64| -> Result<u64, String> {
            match value(name)? {
                None => Ok(default),
                Some(v) => v
                    .trim()
                    .parse::<u64>()
                    .ok()
                    .filter(|n| *n > 0)
                    .ok_or_else(|| format!("{name} must be a number of seconds above 0.")),
            }
        };
        let defaults = Lifetimes::default();
        let lifetimes = Lifetimes {
            code: number("WAYMARK_CODE_SECONDS", defaults.code)?,
            access: number("WAYMARK_ACCESS_TOKEN_SECONDS", defaults.access)?,
            renewal: number("WAYMARK_RENEWAL_TOKEN_SECONDS", defaults.renewal)?,
        };
        let port = match get("PORT").filter(|p| !p.trim().is_empty()) {
            None => 8080,
            Some(p) => p
                .trim()
                .parse::<u16>()
                .map_err(|_| "PORT must be a port number.".to_string())?,
        };
        Ok(ServerSettings {
            port,
            listen: None,
            public_url,
            repo,
            owner,
            signing_key,
            app_id,
            installation_id,
            app_key,
            client_id,
            client_secret,
            github_api: value("WAYMARK_GITHUB_API_URL")?
                .unwrap_or_else(|| "https://api.github.com".into()),
            github_web: value("WAYMARK_GITHUB_URL")?.unwrap_or_else(|| "https://github.com".into()),
            lifetimes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn base() -> HashMap<&'static str, String> {
        let mut m = HashMap::new();
        for (k, v) in [
            ("WAYMARK_PUBLIC_URL", "https://mem.example.org/"),
            ("WAYMARK_REPO", "https://github.com/me/mem.git"),
            ("WAYMARK_OWNER", "me"),
            ("WAYMARK_SIGNING_KEY", "0123456789abcdef0123456789abcdef"),
            ("WAYMARK_GITHUB_APP_ID", "1"),
            ("WAYMARK_GITHUB_INSTALLATION_ID", "2"),
            ("WAYMARK_GITHUB_APP_KEY", "pem"),
            ("WAYMARK_GITHUB_CLIENT_ID", "cid"),
            ("WAYMARK_GITHUB_CLIENT_SECRET", "secret"),
        ] {
            m.insert(k, v.to_string());
        }
        m
    }

    fn load(m: &HashMap<&'static str, String>) -> Result<ServerSettings, String> {
        ServerSettings::from_lookup(&Args::default(), &|n| m.get(n).cloned())
    }

    #[test]
    fn modes() {
        assert_eq!(parse_mode(None), Ok(Mode::Local));
        assert_eq!(parse_mode(Some("local")), Ok(Mode::Local));
        assert_eq!(
            parse_mode(Some("personal-server")),
            Ok(Mode::PersonalServer)
        );
        let team = parse_mode(Some("team-server")).unwrap_err();
        assert!(team.contains("not available yet"), "{team}");
        let bad = parse_mode(Some("remote")).unwrap_err();
        for m in ["local", "personal-server", "team-server"] {
            assert!(bad.contains(m), "{bad}");
        }
    }

    #[test]
    fn complete_settings_load() {
        let s = load(&base()).unwrap();
        assert_eq!(s.public_url, "https://mem.example.org");
        assert_eq!(s.resource(), "https://mem.example.org/mcp");
        assert_eq!(s.port, 8080);
        assert_eq!(s.lifetimes, Lifetimes::default());
        assert_eq!(
            s.git_credential_prefix().as_deref(),
            Some("https://github.com/")
        );
    }

    #[test]
    fn a_missing_setting_is_named() {
        let mut m = base();
        m.remove("WAYMARK_OWNER");
        let e = load(&m).unwrap_err();
        assert!(e.contains("WAYMARK_OWNER"), "{e}");
        assert!(!e.contains("WAYMARK_REPO"), "{e}");
    }

    #[test]
    fn the_public_url_must_be_https() {
        let mut m = base();
        m.insert("WAYMARK_PUBLIC_URL", "http://mem.example.org".into());
        assert!(load(&m).unwrap_err().contains("https://"));
        m.insert("WAYMARK_PUBLIC_URL", "http://localhost:9000".into());
        assert!(load(&m).is_ok());
    }

    #[test]
    fn port_and_lifetimes_come_from_the_environment() {
        let mut m = base();
        m.insert("PORT", "9191".into());
        m.insert("WAYMARK_ACCESS_TOKEN_SECONDS", "120".into());
        let s = load(&m).unwrap();
        assert_eq!(s.port, 9191);
        assert_eq!(s.lifetimes.access, 120);
        m.insert("PORT", "x".into());
        assert!(load(&m).is_err());
    }

    #[test]
    fn settings_of_another_mode_stop_the_start() {
        let mut m = base();
        m.insert("WAYMARK_ROLES_FILE", "/x".into());
        assert!(load(&m).unwrap_err().contains("WAYMARK_ROLES_FILE"));
        let args = Args {
            author_name: Some("x".into()),
            ..Args::default()
        };
        let e = ServerSettings::from_lookup(&args, &|n| base().get(n).cloned()).unwrap_err();
        assert!(e.contains("--author-name"), "{e}");
    }

    #[test]
    fn a_short_signing_key_is_refused() {
        let mut m = base();
        m.insert("WAYMARK_SIGNING_KEY", "short".into());
        assert!(load(&m).unwrap_err().contains("32"));
    }

    #[test]
    fn a_secret_can_come_from_a_file() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("owner");
        std::fs::write(&f, "me\n").unwrap();
        let mut m = base();
        m.remove("WAYMARK_OWNER");
        m.insert("WAYMARK_OWNER_FILE", f.to_string_lossy().to_string());
        assert_eq!(load(&m).unwrap().owner, "me");
    }
}
