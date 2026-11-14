//! End-to-end tests of `status` and its checks (see `specs/features/maintenance/test.md`).

mod common;
use common::git::git;
use common::{McpClient, Person, Spawn, World};
use serde_json::{json, Value};
use std::time::{Duration, Instant};

fn status(c: &mut McpClient) -> Value {
    let r = c.call("status", json!({}));
    assert!(!r.is_error, "{}", r.json);
    r.json
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

fn names(s: &Value) -> Vec<String> {
    s["checks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap().to_string())
        .collect()
}

fn personal(w: &World, text: &str) {
    let p = w.root.join("home-a/.config/waymark/config.toml");
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, text).unwrap();
}

fn script(w: &World, name: &str, body: &str) -> String {
    let p = w.root.join(name);
    std::fs::write(&p, format!("#!/bin/sh\n{body}\n")).unwrap();
    std::process::Command::new("chmod")
        .args(["+x", p.to_str().unwrap()])
        .status()
        .unwrap();
    p.display().to_string()
}

fn with_env(w: &World, env: Vec<(String, String)>) -> McpClient {
    McpClient::for_person(
        w,
        Person::A,
        Spawn {
            env,
            ..Spawn::default()
        },
    )
}

const LOGIN_FIX: &str = "Git cannot sign in to this repository. Sign in with Git first, for example with `gh auth login`, or set up an SSH key.";

#[test]
fn a1_a_refused_login_is_reported_with_the_fix() {
    let w = World::new();
    git(
        &w.a,
        &[
            "remote",
            "set-url",
            "origin",
            "ssh://git@example.invalid/x.git",
        ],
    );
    let ssh = script(
        &w,
        "deny.sh",
        "echo 'git@example.invalid: Permission denied (publickey).' >&2\nexit 255",
    );
    let mut c = with_env(&w, vec![("GIT_SSH_COMMAND".into(), ssh)]);
    let s = status(&mut c);
    let login = check(&s, "git_login");
    assert_eq!(login["ok"], false);
    assert_eq!(login["fix"], LOGIN_FIX);
    assert_eq!(check(&s, "remote")["ok"], true);
    assert_eq!(s["healthy"], false);
}

#[test]
fn a2_a_healthy_setup_has_all_checks_ok_in_order() {
    let w = World::new();
    let mut c = with_env(&w, vec![]);
    let s = status(&mut c);
    assert_eq!(s["healthy"], true, "{s}");
    assert_eq!(s["message"], "All checks are ok.");
    assert_eq!(
        names(&s),
        [
            "git",
            "clone",
            "working_tree",
            "remote",
            "git_login",
            "memory_repository_id",
            "settings",
            "author"
        ]
    );
    assert_eq!(s["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(s["memory_repository"]["state"], "ready");
    assert_eq!(s["memory_repository"]["root"], "agent-memory");
    assert!(s["last_error"].is_null());
    assert!(s["limit_sources"].is_object());
}

#[test]
fn a3_read_only_and_paused_still_work() {
    let w = World::new();
    personal(&w, "[limits]\naccess = \"read-only\"\n");
    let mut c = with_env(&w, vec![]);
    let s = status(&mut c);
    assert_eq!(s["settings"]["access"], "read-only");
    drop(c);
    personal(&w, "[limits]\npaused = true\n");
    let mut c = with_env(&w, vec![]);
    let s = status(&mut c);
    assert_eq!(s["settings"]["paused"], true);
    assert!(s["notices"]
        .as_array()
        .unwrap()
        .iter()
        .any(|n| n.as_str().unwrap().contains("paused")));
}

#[test]
fn a4_status_cannot_be_hidden() {
    let w = World::new();
    personal(&w, "[limits]\ntools_denied = [\"status\"]\n");
    let mut c = with_env(&w, vec![]);
    assert!(c.tool_names().contains(&"status".to_string()));
    assert!(!c.call("status", json!({})).is_error);
}

#[test]
fn a5_another_memory_repository_id_is_reported() {
    let w = World::new();
    let other = "123e4567-e89b-42d3-a456-426614174000";
    let mut c = McpClient::for_person(&w, Person::A, Spawn::args(&["--memory-repository-id", other]));
    let s = status(&mut c);
    let k = check(&s, "memory_repository_id");
    assert_eq!(k["ok"], false);
    let fix = k["fix"].as_str().unwrap();
    assert!(fix.contains(other) && fix.contains("--memory-repository-id"));
}

#[test]
fn a6_a_remote_that_does_not_answer_ends_in_time() {
    let w = World::new();
    git(
        &w.a,
        &[
            "remote",
            "set-url",
            "origin",
            "ssh://git@example.invalid/x.git",
        ],
    );
    let ssh = script(&w, "slow.sh", "sleep 60");
    let mut c = with_env(&w, vec![("GIT_SSH_COMMAND".into(), ssh)]);
    let t = Instant::now();
    let s = status(&mut c);
    assert!(t.elapsed() < Duration::from_secs(12), "{:?}", t.elapsed());
    let r = check(&s, "remote");
    assert_eq!(r["ok"], false);
    assert!(r["detail"].as_str().unwrap().contains("not reachable"));
}

#[test]
fn a7_a_broken_config_file_is_named() {
    let w = World::new();
    personal(&w, "[limits\nbroken");
    let mut c = with_env(&w, vec![]);
    let s = status(&mut c);
    let st = check(&s, "settings");
    assert_eq!(st["ok"], false);
    assert!(st["detail"].as_str().unwrap().contains("config.toml"));
    assert!(st["fix"].as_str().unwrap().contains("syntax error"));
    let r = c.call("search", json!({"query": "x"}));
    assert_eq!(r.code(), Some("setup_problem"));
}

#[test]
fn a8_limit_sources_name_the_layers() {
    let w = World::new();
    let toml = w.a.join("agent-memory/.waymark/settings.toml");
    let old = std::fs::read_to_string(&toml).unwrap();
    std::fs::write(
        &toml,
        old.replacen(
            "[limits]\n",
            "[limits]\nmax_results = 3\ntools_denied = [\"delete\"]\n",
            1,
        ),
    )
    .unwrap();
    personal(&w, "[limits]\nmax_results = 5\ntools_denied = [\"move\"]\n");
    let mut c = with_env(&w, vec![]);
    let s = status(&mut c);
    let src = &s["limit_sources"];
    assert_eq!(src["max_results"], json!(["shared"]));
    assert_eq!(src["max_tokens"], json!(["default"]));
    assert_eq!(src["tools_denied"], json!(["shared", "personal"]));
}

#[test]
fn a9_the_removal_steps_are_in_the_install_guide() {
    let text =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/docs/install.md")).unwrap();
    let section = text.split("## Remove waymark").nth(1).expect("section");
    for needle in [
        "claude mcp remove waymark",
        "[mcp_servers.waymark]",
        "git log @{u}..",
        "rm -r ~/.config/waymark",
        "brew uninstall waymark",
    ] {
        assert!(section.contains(needle), "{needle}");
    }
}

#[test]
fn a10_no_secret_in_the_result() {
    let w = World::new();
    git(
        &w.a,
        &[
            "remote",
            "set-url",
            "origin",
            "https://jane:s3cret-value@127.0.0.1:1/x.git",
        ],
    );
    let mut c = with_env(&w, vec![("GH_TOKEN".into(), "ghp_testtoken".into())]);
    let s = status(&mut c);
    let text = s.to_string();
    assert!(
        !text.contains("s3cret-value") && !text.contains("ghp_testtoken"),
        "{text}"
    );
    assert_eq!(check(&s, "remote")["ok"], false);
}

#[test]
fn a11_while_cloning_only_git_and_clone_run() {
    let w = World::new();
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
    let dir = w.root.join("slowgit");
    std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join("git");
    std::fs::write(
        &f,
        format!("#!/bin/sh\ncase \"$*\" in *clone*) sleep 3;; esac\nexec {real} \"$@\"\n"),
    )
    .unwrap();
    std::process::Command::new("chmod")
        .args(["+x", f.to_str().unwrap()])
        .status()
        .unwrap();
    let path = format!("{}:{}", dir.display(), std::env::var("PATH").unwrap());
    let h = w.root.join("h11");
    std::fs::create_dir_all(&h).unwrap();
    let url = format!("file://{}", w.remote.display());
    let mut c = McpClient::start(
        &h,
        Spawn {
            args: vec!["--repo".into(), url],
            env: vec![("PATH".into(), path)],
            ..Spawn::default()
        },
    );
    let s = status(&mut c);
    assert_eq!(names(&s), ["git", "clone"], "{s}");
    assert_eq!(check(&s, "clone")["ok"], false);
    assert!(check(&s, "clone")["fix"]
        .as_str()
        .unwrap()
        .contains("again in a minute"));
}

#[test]
fn a12_no_remote_is_ok_with_a_notice() {
    let w = World::new();
    git(&w.a, &["remote", "remove", "origin"]);
    let mut c = with_env(&w, vec![]);
    let s = status(&mut c);
    let r = check(&s, "remote");
    assert_eq!(r["ok"], true);
    assert_eq!(r["detail"], "no remote");
    assert!(s["notices"]
        .as_array()
        .unwrap()
        .iter()
        .any(|n| n.as_str().unwrap().contains("no remote")));
}

#[test]
fn a13_a_half_done_rebase_is_reported() {
    let w = World::new();
    std::fs::create_dir_all(w.a.join(".git/rebase-merge")).unwrap();
    let mut c = with_env(&w, vec![]);
    let s = status(&mut c);
    let t = check(&s, "working_tree");
    assert_eq!(t["ok"], false);
    assert!(t["fix"].as_str().unwrap().contains("git status"));
}

#[test]
fn a14_status_changes_nothing() {
    let w = World::new();
    let head_a = git(&w.a, &["rev-parse", "HEAD"]);
    let remote = git(&w.remote, &["rev-parse", "main"]);
    let refs_before = git(&w.a, &["for-each-ref"]);
    let mut c = with_env(&w, vec![]);
    status(&mut c);
    assert_eq!(git(&w.a, &["rev-parse", "HEAD"]), head_a);
    assert_eq!(git(&w.remote, &["rev-parse", "main"]), remote);
    assert_eq!(git(&w.a, &["for-each-ref"]), refs_before);
    assert_eq!(git(&w.a, &["status", "--porcelain"]), "");
}
