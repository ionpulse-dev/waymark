//! Pending manual edits: changes in the working tree that waymark did not make.

use crate::error::Result;
use crate::gitsync::cmd;
use crate::gitsync::Repo;
use crate::memrepo::MemRepo;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    New,
    Changed,
    Deleted,
}

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Kind::New => "new",
            Kind::Changed => "changed",
            Kind::Deleted => "deleted",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Edit {
    pub id: String,
    /// The path below the repository folder.
    pub rel: String,
    pub kind: Kind,
}

#[derive(Debug, Clone)]
pub struct Skip {
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Default)]
pub struct Pending {
    pub edits: Vec<Edit>,
    pub skipped: Vec<Skip>,
}

/// The changes in the working tree, sorted into documents and other files.
pub fn find(repo: &Repo, memrepo: &MemRepo) -> Result<Pending> {
    let out = cmd::run_ok(
        repo.dir(),
        None,
        &["status", "--porcelain=v1", "-z", "--untracked-files=all"],
    )?;
    let prefix = memrepo
        .root
        .strip_prefix(&memrepo.repo_dir)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let mut entries: Vec<(String, Kind)> = Vec::new();
    let mut parts = out.stdout.split('\0').filter(|p| !p.is_empty());
    while let Some(entry) = parts.next() {
        if entry.len() < 4 {
            continue;
        }
        let (xy, path) = entry.split_at(3);
        let (x, y) = (xy.as_bytes()[0] as char, xy.as_bytes()[1] as char);
        let path = path.to_string();
        if x == 'R' || x == 'C' {
            // A staged rename: the next entry is the old path.
            if let Some(old) = parts.next() {
                entries.push((old.to_string(), Kind::Deleted));
            }
            entries.push((path, Kind::New));
        } else if x == 'D' || y == 'D' {
            entries.push((path, Kind::Deleted));
        } else if x == 'A' || x == '?' {
            entries.push((path, Kind::New));
        } else {
            entries.push((path, Kind::Changed));
        }
    }

    let mut pending = Pending::default();
    for (path, kind) in entries {
        let below = if prefix.is_empty() {
            Some(path.as_str())
        } else {
            path.strip_prefix(&format!("{prefix}/"))
        };
        let skip = |reason: &str, pending: &mut Pending| {
            pending.skipped.push(Skip {
                path: path.clone(),
                reason: reason.to_string(),
            })
        };
        let Some(below) = below else {
            skip("Not a document.", &mut pending);
            continue;
        };
        let Some(id) = below.strip_suffix(".md") else {
            skip("Not a document.", &mut pending);
            continue;
        };
        if below.split('/').any(|s| s.starts_with('.')) {
            skip("Not a document.", &mut pending);
            continue;
        }
        if memrepo.is_excluded(id) {
            skip("Excluded.", &mut pending);
            continue;
        }
        pending.edits.push(Edit {
            id: id.to_string(),
            rel: path.clone(),
            kind,
        });
    }
    pending.edits.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(pending)
}
