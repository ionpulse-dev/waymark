//! A small state file for each person and knowledge base (see `specs/features/updates`).
//!
//! It holds facts that belong to this machine, not to the knowledge base: for example the time of
//! the last `updates` call. The folder is `$WAYMARK_STATE_DIR`, else `$XDG_STATE_HOME/waymark`,
//! else `~/.local/state/waymark`.

use serde_json::{Map, Value};
use std::path::PathBuf;

pub struct State {
    path: PathBuf,
}

fn state_dir() -> PathBuf {
    if let Some(d) = std::env::var_os("WAYMARK_STATE_DIR") {
        return PathBuf::from(d);
    }
    if let Some(d) = std::env::var_os("XDG_STATE_HOME") {
        return PathBuf::from(d).join("waymark");
    }
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".local/state/waymark")
}

impl State {
    /// The state of one knowledge base, by its ID.
    pub fn for_kb(kb_id: &str) -> State {
        State {
            path: state_dir().join(format!("{kb_id}.json")),
        }
    }

    pub fn at(path: PathBuf) -> State {
        State { path }
    }

    fn read(&self) -> Map<String, Value> {
        std::fs::read_to_string(&self.path)
            .ok()
            .and_then(|t| serde_json::from_str::<Value>(&t).ok())
            .and_then(|v| v.as_object().cloned())
            .unwrap_or_default()
    }

    pub fn get(&self, key: &str) -> Option<Value> {
        self.read().get(key).cloned()
    }

    pub fn get_str(&self, key: &str) -> Option<String> {
        self.get(key).and_then(|v| v.as_str().map(String::from))
    }

    /// Set one key. Other keys stay. The file is replaced in one step.
    pub fn set(&self, key: &str, value: Value) -> std::io::Result<()> {
        let mut map = self.read();
        map.insert(key.to_string(), value);
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let tmp = self
            .path
            .with_extension(format!("json.tmp-{}", std::process::id()));
        std::fs::write(&tmp, serde_json::to_string_pretty(&Value::Object(map))?)?;
        std::fs::rename(&tmp, &self.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_missing_file_is_empty_and_other_keys_stay() {
        let dir = tempfile::tempdir().unwrap();
        let s = State::at(dir.path().join("sub/kb.json"));
        assert!(s.get("a").is_none());
        s.set("a", json!("1")).unwrap();
        s.set("b", json!("2")).unwrap();
        s.set("a", json!("3")).unwrap();
        assert_eq!(s.get_str("a").as_deref(), Some("3"));
        assert_eq!(s.get_str("b").as_deref(), Some("2"));
    }

    #[test]
    fn a_broken_file_reads_as_empty() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("kb.json");
        std::fs::write(&p, "not json").unwrap();
        let s = State::at(p);
        assert!(s.get("a").is_none());
        s.set("a", json!("x")).unwrap();
        assert_eq!(s.get_str("a").as_deref(), Some("x"));
    }
}
