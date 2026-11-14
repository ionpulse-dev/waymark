//! Builders for documents and knowledge base content in tests.

use super::git::git;
use std::path::Path;

pub const TODAY: &str = "2026-11-14";
pub const AUTHOR: &str = "Anna Example <anna@example.org>";

/// A valid document with the given title and body.
pub fn doc(title: &str, body: &str) -> String {
    format!(
        "---\ntitle: \"{title}\"\ndescription: A test document about {title}.\nstatus: active\ncreated: {TODAY}\nupdated: {TODAY}\ncreated_by: \"{AUTHOR}\"\nupdated_by: \"{AUTHOR}\"\n---\n\n{body}\n"
    )
}

/// Write a document below the root, without a commit. `id` is the path without `.md`.
pub fn write_doc(repo_dir: &Path, id: &str, text: &str) {
    let path = repo_dir.join("knowledge").join(format!("{id}.md"));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

/// Write a document, commit it, and push it.
pub fn commit_doc(repo_dir: &Path, id: &str, title: &str, body: &str) {
    write_doc(repo_dir, id, &doc(title, body));
    git(repo_dir, &["add", "-A"]);
    git(
        repo_dir,
        &["commit", "-q", "-m", &format!("test: add {id}")],
    );
    git(repo_dir, &["push", "-q"]);
}
