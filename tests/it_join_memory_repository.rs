//! End-to-end tests of `join-memory-repository` (see `specs/features/join-memory-repository/test.md`).

mod common;
use common::git::git;
use common::{McpClient, Spawn, World};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const AUTHOR: [&str; 4] = [
    "--author-name",
    "Anna Example",
    "--author-email",
    "anna@example.org",
];

fn home(w: &World, name: &str) -> PathBuf {
    let h = w.root.join(name);
    std::fs::create_dir_all(&h).unwrap();
    h
}

fn url(w: &World) -> String {
    format!("file://{}", w.remote.display())
}

fn start_with(home: &Path, args: &[&str], author: bool, env: Vec<(String, String)>) -> McpClient {
    let mut a: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    if author {
        a.extend(AUTHOR.iter().map(|s| s.to_string()));
    }
    McpClient::start(
        home,
        Spawn {
            args: a,
            env,
            ..Spawn::default()
        },
    )
}

fn start(home: &Path, args: &[&str]) -> McpClient {
    start_with(home, args, true, vec![])
}

fn status(c: &mut McpClient) -> Value {
    c.call("status", json!({})).json
}

/// Poll `status` until the clone is over.
fn wait_state(c: &mut McpClient) -> Value {
    let t = Instant::now();
    loop {
        let s = status(c);
        if s["memory_repository"]["state"] != "cloning" || t.elapsed() > Duration::from_secs(30) {
            return s;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn check(s: &Value, name: &str) -> Value {
    s["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == name)
        .unwrap_or_else(|| panic!("no check {name}: {s}"))
        .clone()
}

fn personal(h: &Path, text: &str) {
    let p = h.join(".config/waymark/config.toml");
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, text).unwrap();
}

/// A folder with a `git` script that runs first in `PATH`.
fn fake_git(w: &World, name: &str, script: &str) -> Vec<(String, String)> {
    let dir = w.root.join(name);
    std::fs::create_dir_all(&dir).unwrap();
    let real = String::from_utf8(
        std::process::Command::new("which")
            .arg("git")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
    .trim()
    .to_string();
    let file = dir.join("git");
    std::fs::write(&file, script.replace("REAL_GIT", &real)).unwrap();
    std::process::Command::new("chmod")
        .args(["+x", file.to_str().unwrap()])
        .status()
        .unwrap();
    let path = format!(
        "{}:{}",
        dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    vec![("PATH".into(), path)]
}

#[test]
fn a1_no_working_git_means_no_clone() {
    let w = World::new();
    let h = home(&w, "h1");
    let env = fake_git(&w, "fake1", "#!/bin/sh\nexit 1\n");
    let u = url(&w);
    let target = h.join("clone");
    let mut c = start_with(
        &h,
        &["--repo", &u, "--path", target.to_str().unwrap()],
        true,
        env,
    );
    let s = wait_state(&mut c);
    let g = check(&s, "git");
    assert_eq!(g["ok"], false);
    assert!(g["fix"].as_str().unwrap().contains("xcode-select"));
    assert_eq!(s["memory_repository"]["state"], "broken");
    assert!(!target.exists());
}

#[test]
fn a2_a_clone_is_made_and_status_is_ready() {
    let w = World::new();
    let h = home(&w, "h2");
    let u = url(&w);
    let target = h.join("clone");
    let mut c = start(&h, &["--repo", &u, "--path", target.to_str().unwrap()]);
    let s = wait_state(&mut c);
    assert_eq!(s["memory_repository"]["state"], "ready", "{s}");
    assert!(target.join(".git").is_dir());
    assert_eq!(s["healthy"], true, "{s}");
}

#[test]
fn a3_the_default_path_comes_from_the_url() {
    let w = World::new();
    let h = home(&w, "h3");
    let u = url(&w);
    let mut c = start(&h, &["--repo", &u]);
    let s = wait_state(&mut c);
    assert_eq!(s["memory_repository"]["state"], "ready", "{s}");
    // Decided in `decisions.md`: the default folder keeps the form of `project-binding`.
    assert!(h.join("Projects/remote-agent-memory/.git").is_dir());
}

#[test]
fn a4_a_personal_value_cannot_loosen_the_shared_limit() {
    let w = World::new();
    let path = w.a.join("agent-memory/.waymark/settings.toml");
    let old = std::fs::read_to_string(&path).unwrap();
    assert!(old.contains("[limits]\n"));
    std::fs::write(
        &path,
        old.replacen("[limits]\n", "[limits]\naccess = \"read-only\"\n", 1),
    )
    .unwrap();
    git(&w.a, &["add", "-A"]);
    git(&w.a, &["commit", "-q", "-m", "test: read-only"]);
    git(&w.a, &["push", "-q"]);
    let h = home(&w, "h4");
    personal(&h, "[limits]\naccess = \"read-write\"\n");
    let u = url(&w);
    let mut c = start(&h, &["--repo", &u]);
    wait_state(&mut c);
    // The limits of a call are read when the call starts, so ask once more after the clone.
    let s = status(&mut c);
    assert_eq!(s["settings"]["access"], "read-only", "{s}");
    let r = c.call(
        "add",
        json!({"folder": "ops", "title": "T", "description": "D.", "body": "B."}),
    );
    assert_eq!(r.code(), Some("access_denied"));
}

#[test]
fn a5_two_entries_do_not_mix() {
    let w1 = World::new();
    let w2 = World::new();
    let h = home(&w1, "h5");
    let (u1, u2) = (url(&w1), url(&w2));
    let (p1, p2) = (h.join("one"), h.join("two"));
    let mut c1 = start(&h, &["--repo", &u1, "--path", p1.to_str().unwrap()]);
    assert_eq!(wait_state(&mut c1)["memory_repository"]["state"], "ready");
    let before = git(&p1, &["rev-parse", "HEAD"]);
    let mut c2 = start(&h, &["--repo", &u2, "--path", p2.to_str().unwrap()]);
    assert_eq!(wait_state(&mut c2)["memory_repository"]["state"], "ready");
    assert_eq!(git(&p1, &["rev-parse", "HEAD"]), before);
    assert!(p2.join(".git").is_dir());
}

#[test]
fn a6_a_missing_id_is_broken() {
    let w = World::new();
    let path = w.a.join("agent-memory/.waymark/settings.toml");
    let text = std::fs::read_to_string(&path).unwrap();
    let no_id: String = text
        .lines()
        .filter(|l| !l.starts_with("id ="))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&path, no_id).unwrap();
    git(&w.a, &["add", "-A"]);
    git(&w.a, &["commit", "-q", "-m", "test: no id"]);
    git(&w.a, &["push", "-q"]);
    let h = home(&w, "h6");
    let u = url(&w);
    let mut c = start(&h, &["--repo", &u]);
    let s = wait_state(&mut c);
    assert_eq!(s["memory_repository"]["state"], "broken", "{s}");
    let r = c.call("search", json!({"query": "x"}));
    assert_eq!(r.code(), Some("setup_problem"));
}

#[test]
fn a7_two_servers_start_together_and_clone_once() {
    let w = World::new();
    let h = home(&w, "h7");
    let u = url(&w);
    let target = h.join("shared-clone");
    let args = ["--repo", u.as_str(), "--path", target.to_str().unwrap()];
    let mut c1 = start(&h, &args);
    let mut c2 = start(&h, &args);
    assert_eq!(wait_state(&mut c1)["memory_repository"]["state"], "ready");
    assert_eq!(wait_state(&mut c2)["memory_repository"]["state"], "ready");
    assert!(target.join(".git").is_dir());
    let left: Vec<String> = std::fs::read_dir(&h)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.contains("waymark-clone"))
        .collect();
    assert!(left.is_empty(), "{left:?}");
}

#[test]
fn a8_a_path_that_is_in_use_is_never_touched() {
    let w = World::new();
    let h = home(&w, "h8");
    let u = url(&w);
    // A folder with a file.
    let folder = h.join("not-empty");
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::write(folder.join("keep.txt"), "mine").unwrap();
    let mut c = start(&h, &["--repo", &u, "--path", folder.to_str().unwrap()]);
    let s = wait_state(&mut c);
    assert_eq!(s["memory_repository"]["state"], "broken", "{s}");
    assert_eq!(
        std::fs::read_to_string(folder.join("keep.txt")).unwrap(),
        "mine"
    );
    drop(c);
    // A clone of another repository.
    let other = World::new();
    let target = h.join("other-clone");
    git(&h, &["clone", "-q", &url(&other), target.to_str().unwrap()]);
    let head = git(&target, &["rev-parse", "HEAD"]);
    let mut c = start(&h, &["--repo", &u, "--path", target.to_str().unwrap()]);
    let r = c.call("list", json!({}));
    assert_eq!(r.code(), Some("setup_problem"), "{}", r.json);
    assert_eq!(git(&target, &["rev-parse", "HEAD"]), head);
}

#[test]
fn a9_initialize_does_not_wait_for_a_slow_clone() {
    let w = World::new();
    let h = home(&w, "h9");
    let env = fake_git(
        &w,
        "fake9",
        "#!/bin/sh\ncase \"$*\" in *clone*) sleep 3;; esac\nexec REAL_GIT \"$@\"\n",
    );
    let u = url(&w);
    let t = Instant::now();
    let mut c = start_with(&h, &["--repo", &u], true, env);
    assert!(t.elapsed() < Duration::from_secs(2), "initialize was slow");
    let s = status(&mut c);
    assert_eq!(s["memory_repository"]["state"], "cloning", "{s}");
    let s = wait_state(&mut c);
    assert_eq!(s["memory_repository"]["state"], "ready", "{s}");
}

#[test]
fn a10_no_author_does_not_stop_reads() {
    let w = World::new();
    let h = home(&w, "h10");
    let u = url(&w);
    let mut c = start_with(&h, &["--repo", &u], false, vec![]);
    let s = wait_state(&mut c);
    assert_eq!(s["memory_repository"]["state"], "ready", "{s}");
    let a = check(&s, "author");
    assert_eq!(a["ok"], false);
    assert!(a["fix"].as_str().unwrap().contains("--author-name"));
    assert!(!c.call("search", json!({"query": "anything"})).is_error);
}

#[test]
fn a11_a_host_that_is_not_allowed_is_not_cloned() {
    let w = World::new();
    let h = home(&w, "h11");
    personal(&h, "[limits]\nallowed_repo_hosts = [\"github.com\"]\n");
    let u = url(&w);
    let mut c = start(&h, &["--repo", &u]);
    let r = c.call("list", json!({}));
    assert_eq!(r.code(), Some("access_denied"), "{}", r.json);
    assert!(!h.join("Projects").exists());
}

#[test]
fn a12_an_empty_repository_is_the_empty_state() {
    let w = World::new();
    let empty = w.root.join("empty.git");
    std::fs::create_dir_all(&empty).unwrap();
    git(&empty, &["init", "--bare", "--initial-branch=main", "-q"]);
    let h = home(&w, "h12");
    let u = format!("file://{}", empty.display());
    let mut c = start(&h, &["--repo", &u]);
    let s = wait_state(&mut c);
    assert_eq!(s["memory_repository"]["state"], "empty", "{s}");
    let r = c.call("search", json!({"query": "x"}));
    assert_eq!(r.code(), Some("setup_problem"));
    assert!(r.json["next"].as_str().unwrap().contains("init"));
}
