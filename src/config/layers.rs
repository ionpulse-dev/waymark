//! Load the layers of limits from files (see `trust-model`).
//!
//! A file that exists but cannot be read or parsed is an error. It is never ignored,
//! because ignoring it would loosen a limit.

use super::limits::{Access, ApprovalVia, Limits};
use serde::Deserialize;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

/// A problem with a layer file. It maps to the error code `setup_problem`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigError {
    pub path: PathBuf,
    pub message: String,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.path.display(), self.message)
    }
}

#[derive(Debug, Default, Deserialize)]
struct LimitsSection {
    access: Option<Access>,
    paused: Option<bool>,
    tools_denied: Option<Vec<String>>,
    folders: Option<Vec<String>>,
    max_results: Option<usize>,
    max_tokens: Option<usize>,
    max_documents_opened: Option<usize>,
    allowed_repo_hosts: Option<Vec<String>>,
}

#[derive(Debug, Default, Deserialize)]
struct ApprovalSection {
    required_for: Option<Vec<String>>,
    via: Option<ApprovalVia>,
}

/// The shape of a layer file: the sections `[limits]` and `[approval]`.
/// Other sections are ignored, so the shared file can hold more.
#[derive(Debug, Default, Deserialize)]
pub struct LayerFile {
    #[serde(default)]
    limits: LimitsSection,
    #[serde(default)]
    approval: ApprovalSection,
}

impl From<LayerFile> for Limits {
    fn from(f: LayerFile) -> Limits {
        Limits {
            access: f.limits.access,
            paused: f.limits.paused,
            tools_denied: f.limits.tools_denied,
            folders: f.limits.folders,
            max_results: f.limits.max_results,
            max_tokens: f.limits.max_tokens,
            max_documents_opened: f.limits.max_documents_opened,
            allowed_repo_hosts: f.limits.allowed_repo_hosts,
            approval_required_for: f.approval.required_for,
            approval_via: f.approval.via,
        }
    }
}

/// Parse the text of a layer file.
pub fn parse_layer(text: &str) -> Result<Limits, String> {
    toml::from_str::<LayerFile>(text)
        .map(Limits::from)
        .map_err(|e| e.to_string())
}

/// Read one layer file. A missing file is no layer (`Ok(None)`).
pub fn read_layer(path: &Path) -> Result<Option<Limits>, ConfigError> {
    let text = match fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
        Err(e) => {
            return Err(ConfigError {
                path: path.to_path_buf(),
                message: format!("cannot read the file: {e}"),
            })
        }
    };
    parse_layer(&text).map(Some).map_err(|message| ConfigError {
        path: path.to_path_buf(),
        message,
    })
}

/// Where the layer files are.
#[derive(Debug, Clone)]
pub struct Paths {
    /// The system files, read in order. The real path, then the override of `WAYMARK_SYSTEM_CONFIG`.
    pub system: Vec<PathBuf>,
    pub personal: PathBuf,
}

pub const SYSTEM_CONFIG: &str = "/Library/Application Support/waymark/config.toml";

impl Paths {
    pub fn from_env() -> Paths {
        let mut system = vec![PathBuf::from(SYSTEM_CONFIG)];
        if let Some(p) = std::env::var_os("WAYMARK_SYSTEM_CONFIG") {
            system.push(PathBuf::from(p));
        }
        let config_home = std::env::var_os("XDG_CONFIG_HOME")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
            .unwrap_or_else(|| PathBuf::from(".config"));
        Paths {
            system,
            personal: config_home.join("waymark").join("config.toml"),
        }
    }
}

/// Load the system and personal layers, in this order.
pub fn load_file_layers(paths: &Paths) -> Result<Vec<Limits>, ConfigError> {
    let mut out = Vec::new();
    for p in &paths.system {
        if let Some(l) = read_layer(p)? {
            out.push(l);
        }
    }
    if let Some(l) = read_layer(&paths.personal)? {
        out.push(l);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::limits::Effective;

    #[test]
    fn parses_the_sections() {
        let l = parse_layer(
            "[limits]\naccess = \"read-only\"\npaused = true\ntools_denied = [\"delete\"]\nmax_results = 3\n[approval]\nrequired_for = [\"write\"]\nvia = \"console\"\n[people]\nemail_domains = []\n",
        )
        .unwrap();
        assert_eq!(l.access, Some(Access::ReadOnly));
        assert_eq!(l.paused, Some(true));
        assert_eq!(l.max_results, Some(3));
        assert_eq!(l.approval_via, Some(ApprovalVia::Console));
    }

    #[test]
    fn an_empty_file_says_nothing() {
        assert_eq!(parse_layer("").unwrap(), Limits::default());
    }

    #[test]
    fn a_syntax_error_is_an_error_with_the_path() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("config.toml");
        fs::write(&p, "[limits\naccess = ").unwrap();
        let e = read_layer(&p).unwrap_err();
        assert_eq!(e.path, p);
    }

    #[test]
    fn a_wrong_value_is_an_error() {
        assert!(parse_layer("[limits]\naccess = \"read-everything\"\n").is_err());
        assert!(parse_layer("[limits]\nmax_results = \"many\"\n").is_err());
    }

    #[test]
    fn a_missing_file_is_no_layer() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(read_layer(&dir.path().join("none.toml")).unwrap(), None);
    }

    #[test]
    fn the_system_file_beats_the_personal_file() {
        let dir = tempfile::tempdir().unwrap();
        let sys = dir.path().join("system.toml");
        let pers = dir.path().join("personal.toml");
        fs::write(&sys, "[limits]\naccess = \"read-only\"\n").unwrap();
        fs::write(&pers, "[limits]\naccess = \"read-write\"\n").unwrap();
        let layers = load_file_layers(&Paths {
            system: vec![sys],
            personal: pers,
        })
        .unwrap();
        assert_eq!(Effective::from_layers(&layers).access, Access::ReadOnly);
    }

    #[test]
    fn a_broken_layer_file_fails_the_load() {
        let dir = tempfile::tempdir().unwrap();
        let pers = dir.path().join("personal.toml");
        fs::write(&pers, "not toml = = =").unwrap();
        assert!(load_file_layers(&Paths {
            system: vec![],
            personal: pers
        })
        .is_err());
    }

    #[test]
    fn the_file_changes_between_two_loads() {
        let dir = tempfile::tempdir().unwrap();
        let pers = dir.path().join("personal.toml");
        let paths = Paths {
            system: vec![],
            personal: pers.clone(),
        };
        assert!(!Effective::from_layers(&load_file_layers(&paths).unwrap()).paused);
        fs::write(&pers, "[limits]\npaused = true\n").unwrap();
        assert!(Effective::from_layers(&load_file_layers(&paths).unwrap()).paused);
    }
}
