//! The folders of a memory repository (see `specs/features/folders`).
//!
//! A folder exists when it holds a document, a file, a subfolder, or a `.gitkeep`. Git stores no
//! empty folder, so an empty folder on disk is not a folder.

use super::MemRepo;
use std::fs;
use std::path::Path;

/// The name of the file that keeps an empty folder in Git.
pub const GITKEEP: &str = ".gitkeep";

/// One folder and what it holds directly.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Folder {
    /// The path below the root, for example `ops/deploy`.
    pub path: String,
    /// The IDs of the documents in it, without generated index files.
    pub documents: Vec<String>,
    /// Files that are not documents and not `.gitkeep`, as paths below the root.
    pub other_files: Vec<String>,
    pub gitkeep: bool,
}

impl MemRepo {
    /// All folders below the root, parents before their children.
    pub fn folders(&self) -> Vec<Folder> {
        let exclude = self.excluded_paths();
        let mut out = Vec::new();
        walk(&self.root, &self.root, "", &exclude, &mut out);
        out.sort_by(|a, b| a.path.split('/').cmp(b.path.split('/')));
        out
    }

    /// The path of a file below the repository folder, for a path below the root.
    pub fn repo_relative_file(&self, path: &str) -> String {
        let abs = self.root.join(path);
        abs.strip_prefix(&self.repo_dir)
            .unwrap_or(&abs)
            .to_string_lossy()
            .to_string()
    }
}

fn walk(root: &Path, dir: &Path, rel: &str, exclude: &[String], out: &mut Vec<Folder>) -> bool {
    let Ok(rd) = fs::read_dir(dir) else {
        return false;
    };
    let mut entries: Vec<_> = rd.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.file_name());
    let mut f = Folder {
        path: rel.to_string(),
        ..Folder::default()
    };
    let mut any = false;
    for e in entries {
        let name = e.file_name().to_string_lossy().to_string();
        let Ok(ft) = e.file_type() else { continue };
        if ft.is_symlink() {
            continue;
        }
        if name == GITKEEP && ft.is_file() {
            f.gitkeep = true;
            continue;
        }
        if name.starts_with('.') {
            continue;
        }
        let p = e.path();
        let rel_p = p
            .strip_prefix(root)
            .unwrap_or(&p)
            .to_string_lossy()
            .to_string();
        if exclude
            .iter()
            .any(|x| rel_p == *x || rel_p.starts_with(&format!("{x}/")))
        {
            continue;
        }
        if ft.is_dir() {
            if walk(root, &p, &rel_p, exclude, out) {
                any = true;
            }
        } else if let Some(id) = rel_p.strip_suffix(".md") {
            any = true;
            f.documents.push(id.to_string());
        } else {
            any = true;
            f.other_files.push(rel_p);
        }
    }
    let exists = any || f.gitkeep;
    if exists && !rel.is_empty() {
        out.push(f);
    }
    exists
}
