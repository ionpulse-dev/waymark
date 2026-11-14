//! Git helpers for tests.

use std::path::Path;
use std::process::Command;

/// Run git in a folder with the author of the test. Panics if git fails.
pub fn git(dir: &Path, args: &[&str]) -> String {
    let out = run(dir, args);
    assert!(out.0, "git {args:?} failed in {}: {}", dir.display(), out.2);
    out.1
}

/// Run git. Returns (ok, stdout, stderr).
pub fn run(dir: &Path, args: &[&str]) -> (bool, String, String) {
    let o = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .env("GIT_AUTHOR_NAME", "Test Setup")
        .env("GIT_AUTHOR_EMAIL", "setup@example.org")
        .env("GIT_COMMITTER_NAME", "Test Setup")
        .env("GIT_COMMITTER_EMAIL", "setup@example.org")
        .output()
        .expect("run git");
    (
        o.status.success(),
        String::from_utf8_lossy(&o.stdout).to_string(),
        String::from_utf8_lossy(&o.stderr).to_string(),
    )
}

/// The subjects of the commits, newest first.
pub fn subjects(dir: &Path) -> Vec<String> {
    git(dir, &["log", "--format=%s"])
        .lines()
        .map(String::from)
        .collect()
}

/// True if no rebase or merge is in progress.
pub fn no_operation_in_progress(dir: &Path) -> bool {
    let gd = git(dir, &["rev-parse", "--absolute-git-dir"]);
    let gd = Path::new(gd.trim());
    !gd.join("rebase-merge").exists()
        && !gd.join("rebase-apply").exists()
        && !gd.join("MERGE_HEAD").exists()
}
