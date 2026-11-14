//! A world for one test: a bare remote and two clones, with their own authors.

use super::git::git;
use std::path::{Path, PathBuf};
use std::sync::Once;
use waymark::gitsync::{Author, Repo};

static ISOLATE: Once = Once::new();

/// Make the test process independent of the real home and the real Git config.
fn isolate() {
    ISOLATE.call_once(|| {
        let home = std::env::temp_dir().join(format!("waymark-test-home-{}", std::process::id()));
        std::fs::create_dir_all(&home).unwrap();
        std::env::set_var("HOME", &home);
        std::env::set_var("XDG_CONFIG_HOME", home.join(".config"));
        std::env::set_var("GIT_CONFIG_GLOBAL", "/dev/null");
        std::env::set_var("GIT_CONFIG_SYSTEM", "/dev/null");
        std::env::set_var("GIT_TERMINAL_PROMPT", "0");
    });
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Person {
    A,
    B,
}

pub struct World {
    pub root: PathBuf,
    pub remote: PathBuf,
    pub a: PathBuf,
    pub b: PathBuf,
    keep: bool,
}

impl World {
    /// A bare remote with no commit and two clones of it. No knowledge base exists.
    pub fn empty() -> World {
        isolate();
        let root = tempfile::Builder::new()
            .prefix("waymark-world-")
            .tempdir()
            .unwrap()
            .keep();
        let remote = root.join("remote.git");
        std::fs::create_dir_all(&remote).unwrap();
        git(&remote, &["init", "--bare", "--initial-branch=main", "-q"]);
        git(&root, &["clone", "-q", remote.to_str().unwrap(), "a"]);
        git(&root, &["clone", "-q", remote.to_str().unwrap(), "b"]);
        World {
            a: root.join("a"),
            b: root.join("b"),
            root,
            remote,
            keep: false,
        }
    }

    /// A remote with one commit (a knowledge base from `init`) and two clones.
    pub fn new() -> World {
        isolate();
        let root = tempfile::Builder::new()
            .prefix("waymark-world-")
            .tempdir()
            .unwrap()
            .keep();
        let remote = root.join("remote.git");
        let a = root.join("a");
        let b = root.join("b");
        std::fs::create_dir_all(&remote).unwrap();
        git(&remote, &["init", "--bare", "--initial-branch=main", "-q"]);
        git(&root, &["clone", "-q", remote.to_str().unwrap(), "a"]);
        git(&a, &["checkout", "-q", "-b", "main"]);
        waymark::kb::init(&a, "Test", "test").unwrap();
        git(&a, &["add", "-A"]);
        git(&a, &["commit", "-q", "-m", "Initial"]);
        git(&a, &["push", "-q", "-u", "origin", "main"]);
        git(&root, &["clone", "-q", remote.to_str().unwrap(), "b"]);
        World {
            root,
            remote,
            a,
            b,
            keep: false,
        }
    }

    pub fn dir(&self, p: Person) -> &Path {
        match p {
            Person::A => &self.a,
            Person::B => &self.b,
        }
    }

    pub fn author(p: Person) -> Author {
        match p {
            Person::A => Author {
                name: "Anna Example".into(),
                email: "anna@example.org".into(),
            },
            Person::B => Author {
                name: "Ben Example".into(),
                email: "ben@example.org".into(),
            },
        }
    }

    pub fn repo(&self, p: Person) -> Repo {
        Repo::open(self.dir(p), World::author(p)).unwrap()
    }
}

impl Default for World {
    fn default() -> Self {
        World::new()
    }
}

impl Drop for World {
    fn drop(&mut self) {
        if std::thread::panicking() {
            self.keep = true;
            eprintln!("The test folder is kept: {}", self.root.display());
        }
        if !self.keep {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }
}
