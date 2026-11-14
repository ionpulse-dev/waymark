//! End-to-end tests of `login-and-identity` (see `specs/features/login-and-identity/test.md`).

mod common;
use common::git::{git, subjects};
use common::{McpClient, Person, Spawn, World};
use serde_json::{json, Value};
use std::time::{Duration, Instant};

/// Start the server for clone A with explicit arguments (no author arguments added).
fn start(w: &World, args: &[&str], env: &[(&str, &str)]) -> McpClient {
    let mut a: Vec<String> = vec!["--path".into(), w.a.display().to_string()];
    a.extend(args.iter().map(|s| s.to_string()));
    McpClient::start(
        &w.root.join("home-a"),
        Spawn {
            args: a,
            env: env
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            ..Spawn::default()
        },
    )
}

fn add(c: &mut McpClient, title: &str) -> common::Reply {
    c.call(
        "add",
        json!({"folder": "ops", "title": title, "description": "About it.", "body": "Text."}),
    )
}

fn author_of(c: &mut McpClient) -> Value {
    let r = c.call("status", json!({}));
    r.json["author"].clone()
}

fn set_shared(w: &World, extra: &str) {
    let path = w.a.join("knowledge/.mkbase/waymark.toml");
    let old = std::fs::read_to_string(&path).unwrap();
    std::fs::write(&path, format!("{old}\n{extra}\n")).unwrap();
    git(&w.a, &["add", "-A"]);
    git(
        &w.a,
        &[
            "-c",
            "user.name=T",
            "-c",
            "user.email=t@corp.org",
            "commit",
            "-q",
            "-m",
            "test: settings",
        ],
    );
    git(&w.a, &["push", "-q"]);
}

#[test]
fn a1_author_sources_in_order() {
    let w = World::new();
    git(&w.a, &["config", "user.name", "Git Name"]);
    git(&w.a, &["config", "user.email", "git@corp.org"]);
    // Git config only.
    let mut c = start(&w, &[], &[]);
    let a = author_of(&mut c);
    assert_eq!(a["name"], "Git Name");
    assert_eq!(a["name_source"], "git_config");
    assert_eq!(a["email_source"], "git_config");
    drop(c);
    // Environment beats the Git config. Each part is resolved on its own.
    let mut c = start(&w, &[], &[("WAYMARK_AUTHOR_NAME", "Env Name")]);
    let a = author_of(&mut c);
    assert_eq!(a["name"], "Env Name");
    assert_eq!(a["name_source"], "environment");
    assert_eq!(a["email"], "git@corp.org");
    assert_eq!(a["email_source"], "git_config");
    drop(c);
    // The argument beats the environment.
    let mut c = start(
        &w,
        &[
            "--author-name",
            "Arg Name",
            "--author-email",
            "arg@corp.org",
        ],
        &[("WAYMARK_AUTHOR_NAME", "Env Name")],
    );
    let a = author_of(&mut c);
    assert_eq!(a["name"], "Arg Name");
    assert_eq!(a["name_source"], "argument");
    assert_eq!(a["email_source"], "argument");
}

#[test]
fn a2_no_author_fails_a_write_but_not_a_read() {
    let w = World::new();
    let before = subjects(&w.remote);
    let mut c = start(&w, &[], &[]);
    assert!(!c.call("list", json!({})).is_error, "reads need no author");
    let r = add(&mut c, "No author");
    assert_eq!(r.code(), Some("setup_problem"));
    assert!(r.json["next"].as_str().unwrap().contains("--author-name"));
    assert_eq!(subjects(&w.remote), before);
}

#[test]
fn a3_arguments_win_over_the_git_config() {
    let w = World::new();
    git(&w.a, &["config", "user.name", "Other"]);
    git(&w.a, &["config", "user.email", "other@corp.org"]);
    let mut c = start(
        &w,
        &[
            "--author-name",
            "Jane Doe",
            "--author-email",
            "jane@example.org",
        ],
        &[],
    );
    assert!(!add(&mut c, "Jane writes").is_error);
    let fmt = git(
        &w.remote,
        &["log", "-1", "--format=%an|%ae|%cn|%ce", "main"],
    );
    assert_eq!(
        fmt.trim(),
        "Jane Doe|jane@example.org|Jane Doe|jane@example.org"
    );
}

#[test]
fn a4_status_checks_the_login_and_shows_no_secret() {
    let w = World::new();
    let mut c = McpClient::for_person(&w, Person::A, Spawn::default());
    let r = c.call("status", json!({}));
    let checks = r.json["checks"].as_array().unwrap();
    let find = |n: &str| checks.iter().find(|c| c["name"] == n).unwrap().clone();
    assert_eq!(find("git_login")["ok"], true);
    assert_eq!(find("author")["ok"], true);
    assert_eq!(r.json["author"]["name_source"], "argument");
}

#[test]
fn a5_a_failed_login_is_sync_failed_and_fast() {
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
    let script = w.root.join("fake-ssh.sh");
    std::fs::write(
        &script,
        "#!/bin/sh\necho 'git@example.invalid: Permission denied (publickey).' >&2\nexit 255\n",
    )
    .unwrap();
    std::process::Command::new("chmod")
        .args(["+x", script.to_str().unwrap()])
        .status()
        .unwrap();
    let mut c = McpClient::for_person(
        &w,
        Person::A,
        Spawn {
            env: vec![("GIT_SSH_COMMAND".into(), script.display().to_string())],
            ..Spawn::default()
        },
    );
    let t = Instant::now();
    let r = c.call("sync", json!({}));
    assert_eq!(r.code(), Some("sync_failed"), "{}", r.json);
    assert_eq!(
        r.json["next"].as_str().unwrap(),
        "Git cannot sign in to this repository. Sign in with Git first, for example with `gh auth login`, or set up an SSH key."
    );
    assert!(t.elapsed() < Duration::from_secs(10));
    // The status check says the same.
    let s = c.call("status", json!({}));
    let login = s.json["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "git_login")
        .unwrap()
        .clone();
    assert_eq!(login["ok"], false);
    assert!(login["fix"].as_str().unwrap().contains("gh auth login"));
}

#[test]
fn a6_a_blocked_address_stops_the_write() {
    let w = World::new();
    let before = subjects(&w.remote);
    let mut c = start(
        &w,
        &[
            "--author-name",
            "Jane",
            "--author-email",
            "jane@users.noreply.github.com",
        ],
        &[],
    );
    let r = add(&mut c, "Blocked");
    assert_eq!(r.code(), Some("setup_problem"));
    assert!(r.message().contains("jane@users.noreply.github.com"));
    assert!(r.json["next"].as_str().unwrap().contains("--author-email"));
    assert_eq!(subjects(&w.remote), before);
    let s = c.call("status", json!({}));
    let author = s.json["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "author")
        .unwrap()
        .clone();
    assert_eq!(author["ok"], false);
}

#[test]
fn a7_allowed_domains_from_the_shared_file() {
    let w = World::new();
    set_shared(&w, "[people]\nemail_domains = [\"corp.example.org\"]");
    let mut c = start(
        &w,
        &[
            "--author-name",
            "Jane",
            "--author-email",
            "jane@corp.example.org",
        ],
        &[],
    );
    assert!(!add(&mut c, "Allowed").is_error);
    drop(c);
    for bad in ["jane@other.org", "jane@sub.corp.example.org"] {
        let mut c = start(&w, &["--author-name", "Jane", "--author-email", bad], &[]);
        let r = add(&mut c, "Refused");
        assert_eq!(r.code(), Some("setup_problem"), "{bad}");
    }
}

#[test]
fn a8_the_committed_email_passed_the_check() {
    let w = World::new();
    let mut c = McpClient::for_person(&w, Person::A, Spawn::default());
    assert!(!add(&mut c, "Checked").is_error);
    let email = git(&w.remote, &["log", "-1", "--format=%ae", "main"]);
    assert!(waymark::identity::check_email(email.trim(), &Default::default()).is_ok());
}

#[test]
fn a9_no_secret_leaves_the_server() {
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
    let mut c = McpClient::for_person(
        &w,
        Person::A,
        Spawn {
            env: vec![("GH_TOKEN".into(), "ghp_testtoken".into())],
            ..Spawn::default()
        },
    );
    let mut all = String::new();
    all.push_str(&c.initialize_result.to_string());
    for (tool, args) in [
        ("status", json!({})),
        ("sync", json!({})),
        ("list", json!({})),
        (
            "add",
            json!({"folder": "ops", "title": "T", "description": "D.", "body": "B."}),
        ),
    ] {
        all.push_str(&c.call(tool, args).json.to_string());
    }
    assert!(!all.contains("s3cret-value"), "{all}");
    assert!(!all.contains("ghp_testtoken"));
}

#[test]
fn a10_a_blank_name_counts_as_missing() {
    let w = World::new();
    let mut c = start(
        &w,
        &["--author-name", "   ", "--author-email", "jane@example.org"],
        &[],
    );
    let r = add(&mut c, "Blank");
    assert_eq!(r.code(), Some("setup_problem"));
    assert!(r.message().contains("name"));
}
