//! End-to-end tests of `restore_document` and `supersedes` in `update_document`
//! (see `specs/features/restore-and-replace/test.md`).

mod common;
use common::git::{git, subjects};
use common::{McpClient, Person, Spawn, World};
use serde_json::{json, Value};
use std::path::Path;

fn client(w: &World) -> McpClient {
    McpClient::for_person(w, Person::A, Spawn::default())
}

fn add(c: &mut McpClient, title: &str) -> Value {
    let r = c.call(
        "add_document",
        json!({"folder": "ops", "title": title, "description": format!("About {title}."), "body": "x"}),
    );
    assert!(!r.is_error, "{}", r.json);
    r.json
}

fn version(c: &mut McpClient, id: &str) -> String {
    let r = c.call("show", json!({"id": id}));
    assert!(!r.is_error, "{}", r.json);
    r.json["version"].as_str().unwrap().to_string()
}

fn file(dir: &Path, id: &str) -> String {
    std::fs::read_to_string(dir.join("agent-memory").join(format!("{id}.md"))).unwrap()
}

/// Add a document and deprecate it. Returns its ID.
fn deprecated(c: &mut McpClient, title: &str) -> String {
    let id = add(c, title)["id"].as_str().unwrap().to_string();
    let base = version(c, &id);
    let r = c.call(
        "deprecate_document",
        json!({"id": id, "base": base, "reason": "Old."}),
    );
    assert!(!r.is_error, "{}", r.json);
    id
}

#[test]
fn restore_makes_a_deprecated_document_active_in_one_commit() {
    let w = World::new();
    let mut c = client(&w);
    let id = deprecated(&mut c, "Gone");
    assert!(file(&w.a, &id).contains("status: deprecated"));
    let before = subjects(&w.remote).len();
    let base = version(&mut c, &id);
    let r = c.call(
        "restore_document",
        json!({"id": id, "base": base, "reason": "It was right."}),
    );
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(r.json["status"], "active");
    assert_eq!(r.json["pushed"], true);
    assert!(r.json["commit"].is_string());
    let text = file(&w.a, &id);
    assert!(
        text.contains("status: active") && !text.contains("deprecated"),
        "{text}"
    );
    let log = subjects(&w.remote);
    assert_eq!(log.len(), before + 1);
    assert_eq!(log[0], "waymark: restore \"Gone\"");
    let body = git(&w.remote, &["log", "-1", "--format=%b"]);
    assert!(body.contains("Reason: It was right."), "{body}");
    // The document can be found again as an active document.
    let s = c.call("search", json!({"query": "Gone"}));
    assert_eq!(s.json["total_matches"], 1);
}

#[test]
fn restore_refuses_an_active_document_and_an_old_base() {
    let w = World::new();
    let mut c = client(&w);
    let id = add(&mut c, "Alive")["id"].as_str().unwrap().to_string();
    let base = version(&mut c, &id);
    let r = c.call("restore_document", json!({"id": id, "base": base}));
    assert_eq!(r.code(), Some("validation_failed"));
    assert!(r.message().contains("already active"), "{}", r.message());
    let id = deprecated(&mut c, "Gone");
    let old = version(&mut c, &id);
    // Another change moves the version on.
    let r = c.call(
        "update_document",
        json!({"id": id, "base": old, "tags": ["x"]}),
    );
    assert!(!r.is_error, "{}", r.json);
    let before = file(&w.a, &id);
    let r = c.call("restore_document", json!({"id": id, "base": old}));
    assert_eq!(r.code(), Some("stale_base"));
    assert_eq!(file(&w.a, &id), before);
}

#[test]
fn restore_names_the_active_document_that_replaces_it() {
    let w = World::new();
    let mut c = client(&w);
    let old = add(&mut c, "Old way")["id"].as_str().unwrap().to_string();
    let r = c.call(
        "add_document",
        json!({"folder": "ops", "title": "New way", "description": "About the new way.", "body": "x", "supersedes": old}),
    );
    assert!(!r.is_error, "{}", r.json);
    let new_id = r.json["id"].as_str().unwrap().to_string();
    let base = version(&mut c, &old);
    let r = c.call("restore_document", json!({"id": old, "base": base}));
    assert!(!r.is_error, "{}", r.json);
    let notices = r.json["notices"].to_string();
    assert!(
        notices.contains(&new_id) && notices.contains("replaces"),
        "{notices}"
    );
    assert!(file(&w.a, &old).contains("status: active"));
}

#[test]
fn restore_resets_the_trust_with_verification() {
    let w = World::new();
    let path = w.a.join("agent-memory/.waymark/settings.toml");
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::write(
        &path,
        text.replacen("verification = false", "verification = true", 1),
    )
    .unwrap();
    git(&w.a, &["add", "-A"]);
    git(&w.a, &["commit", "-q", "-m", "test: settings"]);
    git(&w.a, &["push", "-q"]);
    let mut c = client(&w);
    let id = add(&mut c, "Checked")["id"].as_str().unwrap().to_string();
    let base = version(&mut c, &id);
    let r = c.call("verify_document", json!({"id": id, "base": base}));
    assert!(!r.is_error, "{}", r.json);
    let base = version(&mut c, &id);
    let r = c.call(
        "reject_document",
        json!({"id": id, "base": base, "reason": "Wrong."}),
    );
    assert!(!r.is_error, "{}", r.json);
    let base = version(&mut c, &id);
    let r = c.call("restore_document", json!({"id": id, "base": base}));
    assert!(!r.is_error, "{}", r.json);
    let text = file(&w.a, &id);
    assert!(text.contains("trust: unverified"), "{text}");
    assert!(
        !text.contains("verified_by") && !text.contains("verified:"),
        "{text}"
    );
}

#[test]
fn update_document_supersedes_links_and_deprecates_the_old_one() {
    let w = World::new();
    let mut c = client(&w);
    let old = add(&mut c, "Old")["id"].as_str().unwrap().to_string();
    let new_id = add(&mut c, "New")["id"].as_str().unwrap().to_string();
    let before = subjects(&w.remote).len();
    let base = version(&mut c, &new_id);
    let r = c.call(
        "update_document",
        json!({"id": new_id, "base": base, "supersedes": old}),
    );
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(r.json["changed"], json!(["supersedes"]));
    assert_eq!(r.json["deprecated"], json!([old]));
    assert!(file(&w.a, &new_id).contains(&format!("[[{old}]]")));
    assert!(file(&w.a, &old).contains("status: deprecated"));
    assert_eq!(subjects(&w.remote).len(), before + 1, "one commit");
    // `show` reports the replacement.
    let s = c.call("show", json!({"id": old}));
    assert_eq!(s.json["replaced_by"], json!([new_id]));
    // The same call again changes nothing.
    let base = version(&mut c, &new_id);
    let r = c.call(
        "update_document",
        json!({"id": new_id, "base": base, "supersedes": old}),
    );
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(r.json["message"], "No change.");
}

#[test]
fn update_document_refuses_itself_a_missing_document_and_a_loop() {
    let w = World::new();
    let mut c = client(&w);
    let a = add(&mut c, "Alpha")["id"].as_str().unwrap().to_string();
    let b = add(&mut c, "Beta")["id"].as_str().unwrap().to_string();
    let base = version(&mut c, &a);
    let r = c.call(
        "update_document",
        json!({"id": a, "base": base, "supersedes": a}),
    );
    assert_eq!(r.code(), Some("invalid_arguments"));
    let r = c.call(
        "update_document",
        json!({"id": a, "base": base, "supersedes": "ops/nope"}),
    );
    assert_eq!(r.code(), Some("not_found"));
    // Alpha replaces Beta. Then Beta must not replace Alpha.
    let r = c.call(
        "update_document",
        json!({"id": a, "base": base, "supersedes": b}),
    );
    assert!(!r.is_error, "{}", r.json);
    let base_b = version(&mut c, &b);
    let before = file(&w.a, &b);
    let r = c.call(
        "update_document",
        json!({"id": b, "base": base_b, "supersedes": a}),
    );
    assert_eq!(r.code(), Some("validation_failed"), "{}", r.json);
    assert_eq!(file(&w.a, &b), before);
}

#[test]
fn restore_follows_the_limits_and_undo_reverts_it() {
    let w = World::new();
    let mut c = client(&w);
    let id = deprecated(&mut c, "Gone");
    let base = version(&mut c, &id);
    // Preview writes nothing.
    let r = c.call(
        "restore_document",
        json!({"id": id, "base": base, "preview": true}),
    );
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(r.json["preview"], true);
    assert!(file(&w.a, &id).contains("status: deprecated"));
    // Limits.
    let p = w.root.join("home-a/.config/waymark/config.toml");
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    for (limits, code) in [
        ("[limits]\naccess = \"read-only\"\n", "access_denied"),
        ("[limits]\npaused = true\n", "paused"),
        (
            "[limits]\ntools_denied = [\"restore_document\"]\n",
            "access_denied",
        ),
        ("[limits]\nfolders = [\"work/**\"]\n", "access_denied"),
    ] {
        std::fs::write(&p, limits).unwrap();
        let mut c2 = client(&w);
        let r = c2.call("restore_document", json!({"id": id, "base": base}));
        assert!(
            r.code() == Some(code) || r.code() == Some("not_found"),
            "{limits}: {}",
            r.json
        );
        assert!(file(&w.a, &id).contains("status: deprecated"));
    }
    std::fs::remove_file(&p).unwrap();
    // Undo reverts the restore in one step.
    let mut c = client(&w);
    let r = c.call("restore_document", json!({"id": id, "base": base}));
    assert!(!r.is_error, "{}", r.json);
    let commit = r.json["commit"].as_str().unwrap().to_string();
    let u = c.call("undo", json!({"commit": commit}));
    assert!(!u.is_error, "{}", u.json);
    assert!(file(&w.a, &id).contains("status: deprecated"));
    let a = c.call("activity", json!({}));
    assert_eq!(a.json["entries"][1]["verb"], "restore");
}
