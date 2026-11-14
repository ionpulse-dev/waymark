//! The layout of a memory repository and its root (see `memory-repository`).

use super::agents_file::Audience;
use super::settings;
use crate::error::{Error, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub const MARKER: &str = ".waymark";
pub const SETTINGS_FILE: &str = "settings.toml";

/// Is the repository empty? No commit, or only files that a Git host creates.
pub fn repo_is_empty(dir: &Path) -> bool {
    use crate::gitsync::cmd;
    match cmd::run(dir, None, &["rev-parse", "--verify", "-q", "HEAD"]) {
        Ok(o) if !o.ok => return true,
        Err(_) => return false,
        _ => {}
    }
    let Ok(o) = cmd::run(dir, None, &["ls-files"]) else {
        return false;
    };
    o.ok && o
        .stdout
        .lines()
        .all(|f| matches!(f, "README.md" | "README" | "LICENSE" | "LICENSE.md"))
}

/// Find the root of a memory repository: the folder that contains the marker folder `.waymark`.
/// Looks at the folder itself, then `agent-memory/`, then below it (up to 3 levels), then above it.
pub fn find_root(start: &Path) -> Result<PathBuf> {
    let has_marker = |p: &Path| p.join(MARKER).is_dir();
    if has_marker(start) {
        return Ok(start.to_path_buf());
    }
    if has_marker(&start.join("agent-memory")) {
        return Ok(start.join("agent-memory"));
    }
    // Below the start folder, breadth first.
    let mut level = vec![start.to_path_buf()];
    for _ in 0..3 {
        let mut next = Vec::new();
        for dir in &level {
            let Ok(rd) = fs::read_dir(dir) else { continue };
            let mut subs: Vec<PathBuf> = rd
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                .map(|e| e.path())
                .filter(|p| {
                    !p.file_name()
                        .map(|n| n.to_string_lossy().starts_with('.'))
                        .unwrap_or(true)
                })
                .collect();
            subs.sort();
            for s in subs {
                if has_marker(&s) {
                    return Ok(s);
                }
                next.push(s);
            }
        }
        level = next;
    }
    // Above the start folder.
    let mut cur = start.parent();
    while let Some(p) = cur {
        if has_marker(p) {
            return Ok(p.to_path_buf());
        }
        cur = p.parent();
    }
    Err(Error::setup(format!(
        "No memory repository found at {}.",
        start.display()
    ))
    .with_next("Check the path, or create a memory repository with the init tool."))
}

/// The files that `init` writes, as (path below the repository folder, content).
pub fn init_files(
    name: &str,
    slug: &str,
    id: &str,
    audience: Audience,
    email_domains: &[String],
) -> Vec<(String, String)> {
    super::init::files(
        &super::init::InitInput::plain(name, slug, audience, email_domains),
        id,
    )
    .expect("plain settings are valid")
}

/// Write the layout of a new memory repository into a folder. Writes nothing and fails with
/// `validation_failed` if a file of the layout exists already.
/// Returns the paths that were written, below the repository folder.
pub fn init(repo_dir: &Path, name: &str, slug: &str) -> Result<Vec<String>> {
    init_with(repo_dir, name, slug, Audience::Personal, &[])
}

/// Like `init`, for an audience and the allowed email domains of a team.
pub fn init_with(
    repo_dir: &Path,
    name: &str,
    slug: &str,
    audience: Audience,
    email_domains: &[String],
) -> Result<Vec<String>> {
    let id = uuid::Uuid::new_v4().to_string();
    let files = init_files(name, slug, &id, audience, email_domains);
    for (rel, _) in &files {
        if repo_dir.join(rel).exists() {
            return Err(Error::validation(format!(
                "The file {rel} exists already. init does not overwrite files."
            ))
            .with_next("Use an empty repository, or remove the file first."));
        }
    }
    let mut written = Vec::new();
    for (rel, content) in files {
        let path = repo_dir.join(&rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| Error::setup(format!("Cannot create {}: {e}", parent.display())))?;
        }
        fs::write(&path, content)
            .map_err(|e| Error::setup(format!("Cannot write {}: {e}", path.display())))?;
        written.push(rel);
    }
    // The new settings must parse.
    let text = fs::read_to_string(repo_dir.join(format!("agent-memory/{MARKER}/{SETTINGS_FILE}")))
        .unwrap_or_default();
    settings::parse(&text, SETTINGS_FILE)?;
    Ok(written)
}

/// A URL-safe short name from a display name.
pub fn slug_from_name(name: &str) -> String {
    super::init::derive_slug(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Code;

    #[test]
    fn init_writes_the_whole_layout_and_a_valid_uuid() {
        let dir = tempfile::tempdir().unwrap();
        let written = init(dir.path(), "Example Company", "example").unwrap();
        assert_eq!(written.len(), 6);
        for rel in &written {
            assert!(dir.path().join(rel).is_file(), "{rel}");
        }
        let text =
            fs::read_to_string(dir.path().join("agent-memory/.waymark/settings.toml")).unwrap();
        let s = settings::parse(&text, "settings.toml").unwrap();
        assert_eq!(s.name, "Example Company");
        assert_eq!(s.slug, "example");
        assert!(uuid::Uuid::parse_str(&s.id).is_ok());
        assert_eq!(
            fs::read_to_string(dir.path().join("agent-memory/.waymark/version")).unwrap(),
            "1.0\n"
        );
    }

    #[test]
    fn init_refuses_to_overwrite_and_writes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("AGENTS.md"), "mine").unwrap();
        let e = init(dir.path(), "X", "x").unwrap_err();
        assert_eq!(e.code, Code::ValidationFailed);
        assert_eq!(
            fs::read_to_string(dir.path().join("AGENTS.md")).unwrap(),
            "mine"
        );
        assert!(!dir.path().join("agent-memory").exists());
    }

    #[test]
    fn obsidian_defaults_use_relative_markdown_links() {
        let dir = tempfile::tempdir().unwrap();
        init(dir.path(), "X", "x").unwrap();
        let j: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(dir.path().join(".obsidian/app.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(j["useMarkdownLinks"], true);
        assert_eq!(j["newLinkFormat"], "relative");
    }

    #[test]
    fn a_name_with_quotes_is_written_as_valid_toml() {
        let dir = tempfile::tempdir().unwrap();
        init(dir.path(), "The \"Best\" Team\\x", "team").unwrap();
        let text =
            fs::read_to_string(dir.path().join("agent-memory/.waymark/settings.toml")).unwrap();
        assert_eq!(
            settings::parse(&text, "w").unwrap().name,
            "The \"Best\" Team\\x"
        );
    }

    #[test]
    fn the_root_is_found() {
        let dir = tempfile::tempdir().unwrap();
        init(dir.path(), "X", "x").unwrap();
        let root = dir.path().join("agent-memory");
        assert_eq!(find_root(dir.path()).unwrap(), root);
        assert_eq!(find_root(&root).unwrap(), root);
        fs::create_dir_all(root.join("a/b")).unwrap();
        assert_eq!(find_root(&root.join("a/b")).unwrap(), root);
    }

    #[test]
    fn a_folder_without_a_memory_repository_is_a_setup_problem() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(find_root(dir.path()).unwrap_err().code, Code::SetupProblem);
    }

    #[test]
    fn slugs() {
        assert_eq!(slug_from_name("Example Company"), "example-company");
        assert_eq!(slug_from_name("!!!"), "agent-memory");
    }
}
