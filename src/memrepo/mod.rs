//! The memory repository: its root, its settings, and its documents (see `memory-repository`).
//!
//! This module reads files. It does not run Git (see `gitsync`).

pub mod agents_file;
pub mod folders;
pub mod init;
pub mod layout;
pub mod repairs;
pub mod settings;

pub use layout::{find_root, init, init_with, slug_from_name, MARKER};
pub use settings::Settings;

use crate::error::{Error, Result};
use std::fs;
use std::path::{Component, Path, PathBuf};

/// A memory repository on disk.
#[derive(Debug, Clone)]
pub struct MemRepo {
    /// The folder of the repository (the working tree).
    pub repo_dir: PathBuf,
    /// The root: the folder that contains `.waymark`.
    pub root: PathBuf,
    pub settings: Settings,
}

/// A document in the memory repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocRef {
    /// The path below the root without `.md`.
    pub id: String,
}

impl MemRepo {
    /// Open the memory repository in a repository folder.
    pub fn open(repo_dir: &Path) -> Result<MemRepo> {
        let root = find_root(repo_dir)?;
        let path = root.join(MARKER).join(layout::SETTINGS_FILE);
        let text = fs::read_to_string(&path).map_err(|e| {
            Error::setup(format!("Cannot read {}: {e}", path.display())).with_next(
                "Create the memory repository with the init tool, or restore the file from Git.",
            )
        })?;
        let settings = settings::parse(&text, ".waymark/settings.toml")?;
        Ok(MemRepo {
            repo_dir: repo_dir.to_path_buf(),
            root,
            settings,
        })
    }

    /// The path of a document below the repository folder, for Git commands.
    pub fn repo_relative(&self, id: &str) -> Result<String> {
        let abs = self.path_of(id)?;
        Ok(abs
            .strip_prefix(&self.repo_dir)
            .unwrap_or(&abs)
            .to_string_lossy()
            .to_string())
    }

    /// The absolute path of a document ID. Rejects `..`, absolute paths, empty parts,
    /// a leading dot, and any path that leaves the root.
    pub fn path_of(&self, id: &str) -> Result<PathBuf> {
        let bad = || {
            Error::invalid(format!("\"{id}\" is not a valid document ID."))
                .with_next("Use the path of the document below the root, without .md.")
        };
        if id.is_empty()
            || id.contains("//")
            || id.ends_with('/')
            || id.ends_with(".md")
            || id.contains('\\')
            || id.contains('\0')
        {
            return Err(bad());
        }
        let rel = Path::new(id);
        for c in rel.components() {
            match c {
                Component::Normal(s) if !s.to_string_lossy().starts_with('.') => {}
                _ => return Err(bad()),
            }
        }
        let mut abs = self.root.join(rel);
        abs.set_extension("md");
        // A document that is a symbolic link is never read or written, also a dangling one.
        if fs::symlink_metadata(&abs).is_ok_and(|m| m.file_type().is_symlink()) {
            return Err(bad());
        }
        // A symbolic link must not lead out of the root.
        if let (Ok(real), Ok(root)) = (
            abs.parent()
                .map(fs::canonicalize)
                .transpose()
                .map(|p| p.unwrap_or_default()),
            fs::canonicalize(&self.root),
        ) {
            if !real.as_os_str().is_empty() && !real.starts_with(&root) {
                return Err(bad());
            }
        }
        if let Ok(real) = fs::canonicalize(&abs) {
            if let Ok(root) = fs::canonicalize(&self.root) {
                if !real.starts_with(root) {
                    return Err(bad());
                }
            }
        }
        Ok(abs)
    }

    /// The paths of `.waymark/exclude`.
    pub(crate) fn excluded_paths(&self) -> Vec<String> {
        self.excluded()
    }

    fn excluded(&self) -> Vec<String> {
        fs::read_to_string(self.root.join(MARKER).join("exclude"))
            .unwrap_or_default()
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(|l| l.trim_start_matches('/').to_string())
            .collect()
    }

    /// Is the document ID in a path of `.waymark/exclude`? A write never goes there.
    pub fn is_excluded(&self, id: &str) -> bool {
        let path = format!("{id}.md");
        self.excluded()
            .iter()
            .any(|x| path == *x || path.starts_with(&format!("{x}/")))
    }

    /// All documents, sorted by ID. Skips names that start with `.`, excluded paths, and symbolic links.
    pub fn documents(&self) -> Vec<DocRef> {
        let exclude = self.excluded();
        let mut out = Vec::new();
        fn walk(root: &Path, dir: &Path, exclude: &[String], out: &mut Vec<DocRef>) {
            let Ok(rd) = fs::read_dir(dir) else { return };
            let mut entries: Vec<_> = rd.filter_map(|e| e.ok()).collect();
            entries.sort_by_key(|e| e.file_name());
            for e in entries {
                let name = e.file_name().to_string_lossy().to_string();
                if name.starts_with('.') {
                    continue;
                }
                let Ok(ft) = e.file_type() else { continue };
                if ft.is_symlink() {
                    continue;
                }
                let p = e.path();
                let rel = p
                    .strip_prefix(root)
                    .unwrap_or(&p)
                    .to_string_lossy()
                    .to_string();
                if exclude
                    .iter()
                    .any(|x| rel == *x || rel.starts_with(&format!("{x}/")))
                {
                    continue;
                }
                if ft.is_dir() {
                    walk(root, &p, exclude, out);
                } else if let Some(id) = rel.strip_suffix(".md") {
                    out.push(DocRef { id: id.to_string() });
                }
            }
        }
        walk(&self.root, &self.root, &exclude, &mut out);
        out
    }

    /// Read the text of a document.
    pub fn read(&self, id: &str) -> Result<String> {
        let path = self.path_of(id)?;
        match fs::read(&path) {
            Ok(bytes) => String::from_utf8(bytes)
                .map_err(|_| Error::validation(format!("The document {id} is not valid UTF-8."))),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(Error::not_found(format!(
                "There is no document with the ID \"{id}\"."
            ))
            .with_next("Use search or list to find the ID.")),
            Err(e) => Err(Error::setup(format!("Cannot read {}: {e}", path.display()))),
        }
    }

    /// The set of all document IDs, for link checks.
    pub fn id_set(&self) -> std::collections::HashSet<String> {
        self.documents().into_iter().map(|d| d.id).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Code;

    fn memrepo_with(files: &[(&str, &str)]) -> (tempfile::TempDir, MemRepo) {
        let dir = tempfile::tempdir().unwrap();
        init(dir.path(), "Test", "test").unwrap();
        for (rel, text) in files {
            let p = dir.path().join("agent-memory").join(rel);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, text).unwrap();
        }
        let memrepo = MemRepo::open(dir.path()).unwrap();
        (dir, memrepo)
    }

    #[test]
    fn open_reads_the_settings() {
        let (_d, memrepo) = memrepo_with(&[]);
        assert_eq!(memrepo.settings.name, "Test");
        assert!(memrepo.root.ends_with("agent-memory"));
    }

    #[test]
    fn a_missing_id_is_a_setup_problem() {
        let (d, _) = memrepo_with(&[]);
        fs::write(
            d.path().join("agent-memory/.waymark/settings.toml"),
            "name = \"x\"\n",
        )
        .unwrap();
        assert_eq!(
            MemRepo::open(d.path()).unwrap_err().code,
            Code::SetupProblem
        );
    }

    #[test]
    fn documents_skip_dot_names_excluded_paths_and_other_files() {
        let (_d, memrepo) = memrepo_with(&[
            ("a.md", "x"),
            ("backend/b.md", "x"),
            (".hidden/c.md", "x"),
            ("README.md", "x"),
            ("notes.txt", "x"),
        ]);
        fs::write(memrepo.root.join(".waymark/exclude"), "README.md\n").unwrap();
        let ids: Vec<String> = memrepo.documents().into_iter().map(|d| d.id).collect();
        assert_eq!(ids, vec!["a", "backend/b"]);
    }

    #[test]
    fn ids_cannot_leave_the_root() {
        let (_d, memrepo) = memrepo_with(&[("a.md", "x")]);
        for bad in [
            "../x",
            "/etc/passwd",
            "a/../../x",
            "",
            "a.md",
            ".waymark/version",
            "a//b",
            "a\\b",
        ] {
            assert!(memrepo.path_of(bad).is_err(), "{bad}");
        }
        assert!(memrepo.path_of("a").is_ok());
        assert!(memrepo.path_of("backend/payments/idempotency-keys").is_ok());
    }

    #[test]
    fn a_symbolic_link_cannot_lead_out_of_the_root() {
        let (d, memrepo) = memrepo_with(&[("a.md", "x")]);
        let outside = d.path().join("outside.md");
        fs::write(&outside, "secret").unwrap();
        std::os::unix::fs::symlink(&outside, memrepo.root.join("link.md")).unwrap();
        assert!(memrepo.path_of("link").is_err());
        let ids: Vec<String> = memrepo.documents().into_iter().map(|d| d.id).collect();
        assert_eq!(ids, vec!["a"]);
    }

    #[test]
    fn read_gives_not_found_with_a_next_sentence() {
        let (_d, memrepo) = memrepo_with(&[("a.md", "hello")]);
        assert_eq!(memrepo.read("a").unwrap(), "hello");
        let e = memrepo.read("zzz").unwrap_err();
        assert_eq!(e.code, Code::NotFound);
        assert!(e.next.is_some());
    }
}
