//! Which documents a call may see: the `folders` limit, and the `folders` argument.

use crate::config::Effective;

pub struct Scope<'a> {
    effective: &'a Effective,
    folders: Vec<String>,
}

impl<'a> Scope<'a> {
    /// `folders` is the argument of the call. No folder, or the root, means all.
    pub fn new(effective: &'a Effective, folders: &[String]) -> Scope<'a> {
        let folders: Vec<String> = folders
            .iter()
            .map(|f| f.trim_matches('/').to_string())
            .collect();
        let folders = if folders.iter().any(|f| f.is_empty() || f == ".") {
            Vec::new()
        } else {
            folders
        };
        Scope { effective, folders }
    }

    /// The limit only, for a call that names an ID.
    pub fn limit(effective: &'a Effective) -> Scope<'a> {
        Scope {
            effective,
            folders: Vec::new(),
        }
    }

    pub fn allows(&self, id: &str) -> bool {
        if !self.effective.allows_folder(id) {
            return false;
        }
        self.folders.is_empty()
            || self
                .folders
                .iter()
                .any(|f| id == f || id.starts_with(&format!("{f}/")))
    }
}
