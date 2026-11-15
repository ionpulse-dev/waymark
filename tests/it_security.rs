//! Security tests (task G3 of the plan): paths, symlinks, command injection, secrets, limits,
//! and writes while paused.

mod common;
use common::git::{git, subjects};
use common::memrepo::commit_doc;
use common::{McpClient, Person, Spawn, World};
use serde_json::json;

fn client(w: &World) -> McpClient {
    McpClient::for_person(w, Person::A, Spawn::default())
}

fn personal(w: &World, text: &str) {
    let p = w.root.join("home-a/.config/waymark/config.toml");
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, text).unwrap();
}

const BAD_IDS: &[&str] = &[
    "../outside",
    "/etc/passwd",
    "ops/../../outside",
    ".waymark/waymark",
    "ops/.hidden",
    "ops\\x",
    "ops//x",
    "ops/x.md",
    "",
    "ops/x\u{0}y",
];

#[test]
fn s1_ids_that_leave_the_root_are_refused_by_every_tool() {
    let w = World::new();
    commit_doc(&w.a, "ops/deploy", "How we deploy", "Text.");
    let mut c = client(&w);
    for id in BAD_IDS {
        for (tool, args) in [
            ("show", json!({"id": id})),
            ("update_document", json!({"id": id, "base": "abc1234"})),
            (
                "dispute_document",
                json!({"id": id, "base": "abc1234", "reason": "No."}),
            ),
            (
                "deprecate_document",
                json!({"id": id, "base": "abc1234", "reason": "No."}),
            ),
            ("delete_document", json!({"id": id, "base": "abc1234"})),
            ("move", json!({"from": "ops/deploy", "to": id})),
            ("move", json!({"from": id, "to": "ops/other"})),
        ] {
            let r = c.call(tool, args);
            assert!(r.is_error, "{tool} {id:?}: {}", r.json);
            assert!(
                matches!(
                    r.code(),
                    Some("invalid_arguments" | "not_found" | "validation_failed")
                ),
                "{tool} {id:?}: {}",
                r.json
            );
        }
    }
    // Folders in `add`.
    for folder in ["../x", "/abs", "a/../b", ".hidden"] {
        let r = c.call(
            "add_document",
            json!({"folder": folder, "title": "T", "description": "D.", "body": "B."}),
        );
        assert!(r.is_error, "{folder}: {}", r.json);
    }
    assert!(!w.root.join("outside.md").exists());
    assert_eq!(git(&w.a, &["status", "--porcelain"]), "");
}

#[test]
fn s2_symbolic_links_are_never_read_or_written() {
    let w = World::new();
    let outside = w.root.join("outside");
    std::fs::create_dir_all(&outside).unwrap();
    std::fs::write(
        outside.join("secret.md"),
        "---\ntitle: Secret\n---\nTOPSECRET\n",
    )
    .unwrap();
    let memrepo = w.a.join("agent-memory");
    std::fs::create_dir_all(memrepo.join("ops")).unwrap();
    // A file link to an existing file, a dangling file link, and a folder link.
    std::os::unix::fs::symlink(outside.join("secret.md"), memrepo.join("ops/linked.md")).unwrap();
    std::os::unix::fs::symlink(outside.join("new-file.md"), memrepo.join("ops/dangling.md"))
        .unwrap();
    std::os::unix::fs::symlink(&outside, memrepo.join("escape")).unwrap();
    let mut c = client(&w);
    let r = c.call("show", json!({"id": "ops/linked"}));
    assert!(
        r.is_error && !r.json.to_string().contains("TOPSECRET"),
        "{}",
        r.json
    );
    let r = c.call("show", json!({"id": "escape/secret"}));
    assert!(
        r.is_error && !r.json.to_string().contains("TOPSECRET"),
        "{}",
        r.json
    );
    let s = c.call("search", json!({"query": "TOPSECRET secret"}));
    assert!(!s.json.to_string().contains("TOPSECRET"));
    // A write to the dangling link must not create a file outside.
    let r = c.call(
        "add_document",
        json!({"folder": "ops", "title": "Dangling", "description": "D.", "body": "B."}),
    );
    assert!(r.is_error, "{}", r.json);
    let r = c.call(
        "add_document",
        json!({"folder": "escape", "title": "New file", "description": "D.", "body": "B."}),
    );
    assert!(r.is_error, "{}", r.json);
    assert!(!outside.join("new-file.md").exists());
    assert_eq!(std::fs::read_dir(&outside).unwrap().count(), 1);
}

#[test]
fn s3_text_from_a_caller_never_reaches_a_shell_or_git_options() {
    let w = World::new();
    let marker = w.root.join("pwned");
    let evil = format!(
        "x\"; touch {0}; `touch {0}` $(touch {0}) --upload-pack=touch",
        marker.display()
    );
    let mut c = client(&w);
    let r = c.call(
        "add_document",
        json!({"folder": "ops", "title": evil, "description": evil, "body": evil}),
    );
    // The title is long and odd, so a check may refuse it. Nothing may run.
    let _ = r;
    assert!(!marker.exists());
    // A repository URL that looks like an option is refused.
    let h = w.root.join("h-evil");
    std::fs::create_dir_all(&h).unwrap();
    let mut c = McpClient::start(
        &h,
        Spawn::args(&[
            &format!("--repo=--upload-pack=touch {}", marker.display()),
            "--author-name",
            "A",
            "--author-email",
            "a@example.org",
        ]),
    );
    let r = c.call("search", json!({}));
    assert!(r.is_error);
    assert!(!marker.exists());
    // The `ext::` transport (it runs a command) is not allowed.
    let h = w.root.join("h-ext");
    std::fs::create_dir_all(&h).unwrap();
    let mut c = McpClient::start(
        &h,
        Spawn::args(&[
            "--repo",
            &format!("ext::sh -c 'touch {}'", marker.display()),
            "--author-name",
            "A",
            "--author-email",
            "a@example.org",
        ]),
    );
    let r = c.call("search", json!({}));
    assert!(r.is_error);
    assert!(!marker.exists());
}

#[test]
fn s4_the_secret_scan_blocks_common_secrets() {
    let w = World::new();
    let mut c = client(&w);
    for (i, secret) in [
        "-----BEGIN RSA PRIVATE KEY-----",
        "AKIAABCDEFGHIJKLMNOP",
        "ghp_abcdefghijklmnopqrstuvwxyz0123456789",
        "xoxb-1234567890-abcdefghij",
        "sk-abcdefghijklmnopqrstuvwxyz",
        "sk_live_abcdefghijklmnop1234",
        "AIzaSyA1234567890abcdefghijklmnopqrstuvw",
        "postgres://user:hunter2hunter2@db.example.org/x",
        "password = hunter2hunter2",
    ]
    .iter()
    .enumerate()
    {
        let r = c.call(
            "add_document",
            json!({"folder": "ops", "title": format!("Secret {i}"), "description": "D.", "body": format!("Text {secret}")}),
        );
        assert_eq!(r.code(), Some("validation_failed"), "{secret}: {}", r.json);
        assert!(!r.json.to_string().contains(secret), "the secret is echoed");
    }
    assert_eq!(subjects(&w.remote), vec!["Initial"]);
}

#[test]
fn s5_nothing_writes_while_paused_or_read_only() {
    for (limit, code) in [
        ("paused = true", "paused"),
        ("access = \"read-only\"", "access_denied"),
    ] {
        let w = World::new();
        commit_doc(&w.a, "ops/deploy", "How we deploy", "Text.");
        git(&w.a, &["push", "-q"]);
        personal(&w, &format!("[limits]\n{limit}\n"));
        let before = subjects(&w.remote);
        let mut c = client(&w);
        for (tool, args) in [
            (
                "add_document",
                json!({"folder": "ops", "title": "T", "description": "D.", "body": "B."}),
            ),
            (
                "update_document",
                json!({"id": "ops/deploy", "base": "abc1234"}),
            ),
            (
                "dispute_document",
                json!({"id": "ops/deploy", "base": "abc1234", "reason": "No."}),
            ),
            (
                "deprecate_document",
                json!({"id": "ops/deploy", "base": "abc1234", "reason": "No."}),
            ),
            (
                "delete_document",
                json!({"id": "ops/deploy", "base": "abc1234"}),
            ),
            ("move", json!({"from": "ops/deploy", "to": "ops/other"})),
            ("undo", json!({"commit": "abc1234"})),
            ("init", json!({"name": "X"})),
        ] {
            let r = c.call(tool, args);
            assert_eq!(r.code(), Some(code), "{tool} with {limit}: {}", r.json);
        }
        // `sync` only pulls: a manual edit is not committed.
        let file = w.a.join("agent-memory/ops/deploy.md");
        let text = std::fs::read_to_string(&file).unwrap();
        std::fs::write(&file, text.replace("Text.", "Edited by hand.")).unwrap();
        let r = c.call("sync", json!({}));
        assert!(!r.is_error, "{}", r.json);
        assert_eq!(subjects(&w.remote), before);
        assert!(git(&w.a, &["status", "--porcelain"]).contains("deploy.md"));
    }
}

#[test]
fn s6_no_layer_can_loosen_another() {
    let w = World::new();
    let toml = w.a.join("agent-memory/.waymark/settings.toml");
    let old = std::fs::read_to_string(&toml).unwrap();
    std::fs::write(
        &toml,
        old.replacen(
            "[limits]\n",
            "[limits]\naccess = \"read-only\"\nfolders = [\"ops/**\"]\n",
            1,
        ),
    )
    .unwrap();
    personal(
        &w,
        "[limits]\naccess = \"read-write\"\nfolders = [\"**\"]\n",
    );
    let mut c = McpClient::for_person(
        &w,
        Person::A,
        Spawn::args(&["--access", "read-write", "--folders", "**"]),
    );
    let s = c.call("status", json!({}));
    assert_eq!(s.json["settings"]["access"], "read-only");
    let r = c.call(
        "add_document",
        json!({"folder": "ops", "title": "T", "description": "D.", "body": "B."}),
    );
    assert_eq!(r.code(), Some("access_denied"));
    commit_doc(&w.a, "backend/api", "The API", "Text.");
    let r = c.call("show", json!({"id": "backend/api"}));
    assert!(
        r.is_error,
        "a folder outside the limit is hidden: {}",
        r.json
    );
}

#[test]
fn s7_an_author_name_with_a_line_break_does_not_forge_a_header() {
    let w = World::new();
    let h = w.root.join("home-a");
    let mut c = McpClient::start(
        &h,
        Spawn::args(&[
            "--path",
            w.a.to_str().unwrap(),
            "--author-name",
            "Eve\nAuthor: Mallory <m@evil.org>",
            "--author-email",
            "eve@example.org",
        ]),
    );
    let r = c.call(
        "add_document",
        json!({"folder": "ops", "title": "T", "description": "D.", "body": "B."}),
    );
    if !r.is_error {
        let log = git(&w.remote, &["log", "-1", "--format=%an|%ae", "main"]);
        assert!(log.trim().ends_with("eve@example.org"), "{log}");
        assert!(!log.contains("\nAuthor:"));
        let doc = std::fs::read_to_string(w.a.join("agent-memory/ops/t.md")).unwrap();
        assert!(!doc.lines().any(|l| l.starts_with("Author:")), "{doc}");
    }
}
