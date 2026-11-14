//! End-to-end tests of `move` (see `specs/features/move-documents/test.md`).

mod common;
use common::git::{git, subjects};
use common::memrepo::write_doc;
use common::{McpClient, Person, Spawn, World};
use serde_json::{json, Value};

const ANNA: &str = "Anna Example <anna@example.org>";

fn client(w: &World) -> McpClient {
    McpClient::for_person(w, Person::A, Spawn::default())
}

fn doc(title: &str, props: &str, body: &str) -> String {
    format!(
        "---\ntitle: \"{title}\"\ndescription: About {title}.\nstatus: active\ncreated: 2020-01-01\nupdated: 2020-01-01\ncreated_by: \"{ANNA}\"\nupdated_by: \"{ANNA}\"\n{props}---\n\n{body}\n"
    )
}

fn commit_all(w: &World, msg: &str) {
    git(&w.a, &["add", "-A"]);
    git(&w.a, &["commit", "-q", "-m", msg]);
    git(&w.a, &["push", "-q"]);
}

/// A knowledge base with a folder and cross links.
fn setup(w: &World) {
    write_doc(
        &w.a,
        "backend/payments/keys",
        &doc(
            "Idempotency keys",
            "related:\n  - \"[[backend/payments/retries]]\"\n",
            "See [retries](retries.md) and [[backend/payments/retries|the retries]].",
        ),
    );
    write_doc(
        &w.a,
        "backend/payments/retries",
        &doc("Retry policy", "", "Back to [[keys]]."),
    );
    write_doc(&w.a, "ops/deploy", &doc("How we deploy", "", "Payments: [keys](../backend/payments/keys.md#top), [[backend/payments/retries#Limits|limits]], and `[[backend/payments/keys]]`.\n\n```\n[[backend/payments/keys]]\n```"));
    write_doc(
        &w.a,
        "ops/unrelated",
        &doc("Unrelated", "", "No links here."),
    );
    std::fs::create_dir_all(w.a.join("agent-memory/backend/payments/img")).unwrap();
    std::fs::write(
        w.a.join("agent-memory/backend/payments/img/diagram.png"),
        b"png",
    )
    .unwrap();
    commit_all(w, "test: setup");
}

fn file(w: &World, id: &str) -> String {
    std::fs::read_to_string(w.a.join("agent-memory").join(format!("{id}.md"))).unwrap()
}

#[test]
fn a_folder_move_keeps_links_and_history_in_two_commits() {
    let w = World::new();
    setup(&w);
    let mut c = client(&w);
    let r = c.call(
        "move",
        json!({"from": "backend/payments/", "to": "platform/payments/"}),
    );
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(r.json["kind"], "folder");
    assert_eq!(r.json["moved_total"], 2);
    assert_eq!(r.json["history_checked"], true);
    assert_eq!(r.json["pushed"], true);
    assert!(r.json["notices"]
        .to_string()
        .contains("outside this knowledge base are not checked"));
    let commits = r.json["commits"].as_array().unwrap();
    assert_eq!(commits.len(), 2);
    // The files moved, including the image.
    assert!(w.a.join("agent-memory/platform/payments/keys.md").exists());
    assert!(w
        .a
        .join("agent-memory/platform/payments/img/diagram.png")
        .exists());
    assert!(!w.a.join("agent-memory/backend/payments").exists());
    // Commit 1: renames only (R100). Commit 2: link rewrites, with the Move line.
    let log = subjects(&w.remote);
    assert_eq!(log[1], "waymark: move \"platform/payments/\"");
    assert_eq!(log[0], "waymark: update \"platform/payments/\"");
    let first = git(
        &w.remote,
        &[
            "diff-tree",
            "-r",
            "-M",
            "--no-commit-id",
            "--name-status",
            "HEAD~1",
        ],
    );
    assert!(first.lines().all(|l| l.starts_with("R100")), "{first}");
    assert_eq!(first.lines().count(), 3);
    let body = git(&w.remote, &["log", "-1", "--format=%b"]);
    assert_eq!(
        body.trim(),
        format!("Move: {}", commits[0].as_str().unwrap())
    );
    // The history follows.
    let follow = git(
        &w.a,
        &[
            "log",
            "--follow",
            "--format=%s",
            "--",
            "agent-memory/platform/payments/keys.md",
        ],
    );
    assert!(follow.contains("test: setup"), "{follow}");
    // Links in other documents work.
    let deploy = file(&w, "ops/deploy");
    assert!(
        deploy.contains("[keys](../platform/payments/keys.md#top)"),
        "{deploy}"
    );
    assert!(deploy.contains("[[platform/payments/retries#Limits|limits]]"));
    // Code is not touched.
    assert!(deploy.contains("`[[backend/payments/keys]]`"));
    assert!(deploy.contains("```\n[[backend/payments/keys]]\n```"));
    // Links between moved documents stay: same depth. The property and full-path link are replaced.
    let keys = file(&w, "platform/payments/keys");
    assert!(
        keys.contains("[retries](retries.md)")
            && keys.contains("[[platform/payments/retries|the retries]]")
    );
    assert!(keys.contains("[[platform/payments/retries]]"));
    // updated and updated_by change only where links changed.
    assert!(!file(&w, "ops/unrelated")
        .contains("updated_by: \"Anna Example <anna@example.org>\"\nupdated: "));
    assert!(file(&w, "ops/unrelated").contains("updated: 2020-01-01"));
    assert!(!deploy.contains("updated: 2020-01-01"));
    // Nothing is broken.
    let lint = c.call("lint", json!({}));
    let broken: Vec<&Value> = lint.json["findings"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| f["rule"] == "broken-link")
        .collect();
    assert!(broken.is_empty(), "{broken:?}");
}

#[test]
fn a_rename_rewrites_links_and_the_short_name_link() {
    let w = World::new();
    setup(&w);
    let mut c = client(&w);
    let r = c.call(
        "move",
        json!({"from": "backend/payments/retries", "to": "backend/payments/retry-policy"}),
    );
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(r.json["kind"], "document");
    assert_eq!(subjects(&w.remote)[1], "waymark: move \"Retry policy\"");
    let keys = file(&w, "backend/payments/keys");
    // The relative link is recomputed (the file name changed), and the short name keeps working.
    assert!(keys.contains("[retries](retry-policy.md)"), "{keys}");
    assert!(keys.contains("[[backend/payments/retry-policy|the retries]]"));
    assert!(file(&w, "backend/payments/retry-policy").contains("Back to [[keys]]."));
}

#[test]
fn a_move_with_no_inbound_links_makes_one_commit() {
    let w = World::new();
    setup(&w);
    let before = subjects(&w.remote).len();
    let mut c = client(&w);
    let r = c.call(
        "move",
        json!({"from": "ops/unrelated", "to": "archive/unrelated"}),
    );
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(r.json["commits"].as_array().unwrap().len(), 1);
    assert_eq!(subjects(&w.remote).len(), before + 1);
    assert_eq!(r.json["links_updated"], json!([]));
}

#[test]
fn a_move_that_cannot_happen_fails_clearly() {
    let w = World::new();
    setup(&w);
    let mut c = client(&w);
    let before = subjects(&w.remote);
    let code = |c: &mut McpClient, from: &str, to: &str| {
        c.call("move", json!({"from": from, "to": to}))
            .code()
            .map(String::from)
    };
    assert_eq!(
        code(&mut c, "ops/unrelated", "ops/deploy").as_deref(),
        Some("validation_failed")
    );
    assert_eq!(
        code(&mut c, "backend/", "backend/inner/").as_deref(),
        Some("validation_failed")
    );
    assert_eq!(
        code(&mut c, "backend/", "ops/").as_deref(),
        Some("validation_failed")
    );
    assert_eq!(
        code(&mut c, "ops/unrelated", "ops/Bad Name").as_deref(),
        Some("validation_failed")
    );
    assert_eq!(
        code(&mut c, "ops/unrelated", "ops/").as_deref(),
        Some("invalid_arguments")
    );
    assert_eq!(
        code(&mut c, "ops/unrelated", "ops/unrelated").as_deref(),
        Some("invalid_arguments")
    );
    assert_eq!(
        code(&mut c, "ops/unrelated", "ops/new.md").as_deref(),
        Some("invalid_arguments")
    );
    assert_eq!(
        code(&mut c, "/", "x/").as_deref(),
        Some("validation_failed")
    );
    assert_eq!(
        code(&mut c, ".waymark/", "x/").as_deref(),
        Some("invalid_arguments")
    );
    assert_eq!(
        code(&mut c, "ops/missing", "ops/other").as_deref(),
        Some("not_found")
    );
    assert_eq!(
        code(&mut c, "nofolder/", "x/").as_deref(),
        Some("not_found")
    );
    assert_eq!(subjects(&w.remote), before);
}

#[test]
fn preview_changes_nothing() {
    let w = World::new();
    setup(&w);
    let before = subjects(&w.remote);
    let mut c = client(&w);
    let r = c.call(
        "move",
        json!({"from": "backend/payments/", "to": "platform/payments/", "preview": true}),
    );
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(r.json["moved_total"], 2);
    assert!(r.json["commits"].is_null() && r.json["history_checked"].is_null());
    assert!(!r.json["links_updated"].as_array().unwrap().is_empty());
    assert!(w.a.join("agent-memory/backend/payments/keys.md").exists());
    assert!(!w.a.join("agent-memory/platform").exists());
    assert_eq!(subjects(&w.remote), before);
    assert!(git(&w.a, &["status", "--porcelain"]).trim().is_empty());
}

#[test]
fn tools_denied_hides_move_and_folders_limit_blocks_it() {
    let w = World::new();
    setup(&w);
    let mut c = McpClient::for_person(&w, Person::A, Spawn::args(&["--tools-denied", "move"]));
    assert!(!c.tool_names().contains(&"move".to_string()));
    let mut c = McpClient::for_person(&w, Person::A, Spawn::args(&["--folders", "backend/**"]));
    let r = c.call(
        "move",
        json!({"from": "backend/payments/", "to": "platform/payments/"}),
    );
    assert_eq!(r.code(), Some("access_denied"));
    assert!(w.a.join("agent-memory/backend/payments/keys.md").exists());
    // A linking document outside the limit blocks the move too.
    let r = c.call(
        "move",
        json!({"from": "backend/payments/keys", "to": "backend/payments/keys2"}),
    );
    assert_eq!(r.code(), Some("access_denied"));
    assert!(r.message().contains("ops/deploy") || r.message().contains("backend/payments"));
}

#[test]
fn a_failed_history_check_removes_the_commit_and_pushes_nothing() {
    let w = World::new();
    setup(&w);
    let before = subjects(&w.remote);
    let local_before = subjects(&w.a);
    let mut c = McpClient::for_person(
        &w,
        Person::A,
        Spawn {
            env: vec![("WAYMARK_TEST_HIDE_HISTORY".into(), "1".into())],
            ..Spawn::default()
        },
    );
    let r = c.call(
        "move",
        json!({"from": "backend/payments/", "to": "platform/payments/"}),
    );
    assert_eq!(r.code(), Some("validation_failed"), "{}", r.json);
    assert_eq!(subjects(&w.remote), before);
    assert_eq!(subjects(&w.a), local_before);
    assert!(w.a.join("agent-memory/backend/payments/keys.md").exists());
    assert!(!w.a.join("agent-memory/platform").exists());
    assert!(git(&w.a, &["status", "--porcelain"]).trim().is_empty());
}

#[test]
fn the_other_clone_sees_the_new_layout_and_a_second_move_is_not_found() {
    let w = World::new();
    setup(&w);
    let mut c = client(&w);
    assert!(
        !c.call(
            "move",
            json!({"from": "backend/payments/", "to": "platform/payments/"})
        )
        .is_error
    );
    git(&w.b, &["pull", "-q"]);
    assert!(w.b.join("agent-memory/platform/payments/keys.md").exists());
    let again = c.call(
        "move",
        json!({"from": "backend/payments/", "to": "platform/payments/"}),
    );
    assert_eq!(again.code(), Some("not_found"));
}

#[test]
fn a_document_with_a_blocked_manual_edit_cannot_move() {
    let w = World::new();
    setup(&w);
    std::fs::write(w.a.join("agent-memory/ops/unrelated.md"), "no frontmatter\n").unwrap();
    let mut c = client(&w);
    let r = c.call(
        "move",
        json!({"from": "ops/unrelated", "to": "archive/unrelated"}),
    );
    assert_eq!(r.code(), Some("stale_base"));
}
