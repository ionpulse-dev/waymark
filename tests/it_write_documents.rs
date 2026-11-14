//! End-to-end tests of the write tools (see `specs/features/write-documents/test.md`).

mod common;
use common::git::{git, subjects};
use common::memrepo::commit_doc;
use common::{McpClient, Person, Spawn, World};
use serde_json::{json, Value};
use std::path::Path;

fn client(w: &World, p: Person) -> McpClient {
    McpClient::for_person(w, p, Spawn::default())
}

fn add(c: &mut McpClient, folder: &str, title: &str, body: &str) -> Value {
    let r = c.call(
        "add",
        json!({"folder": folder, "title": title, "description": format!("About {title}."), "body": body}),
    );
    assert!(!r.is_error, "add failed: {}", r.json);
    r.json
}

fn file(dir: &Path, id: &str) -> String {
    std::fs::read_to_string(dir.join("agent-memory").join(format!("{id}.md"))).unwrap()
}

fn exists(dir: &Path, id: &str) -> bool {
    dir.join("agent-memory").join(format!("{id}.md")).exists()
}

fn remote_log(w: &World) -> Vec<String> {
    subjects(&w.remote)
}

/// Change the shared settings file: replace `from` with `to`, then commit and push.
fn set_shared(w: &World, from: &str, to: &str) {
    let path = w.a.join("agent-memory/.waymark/settings.toml");
    let old = std::fs::read_to_string(&path).unwrap();
    assert!(old.contains(from), "{from} not found");
    std::fs::write(&path, old.replacen(from, to, 1)).unwrap();
    git(&w.a, &["add", "-A"]);
    git(&w.a, &["commit", "-q", "-m", "test: settings"]);
    git(&w.a, &["push", "-q"]);
}

#[test]
fn add_commits_and_pushes() {
    let w = World::new();
    let mut c = client(&w, Person::A);
    let r = add(
        &mut c,
        "backend/payments",
        "Payment API v3 uses idempotency keys",
        "Send an `Idempotency-Key` header.",
    );
    assert_eq!(
        r["id"],
        "backend/payments/payment-api-v3-uses-idempotency-keys"
    );
    assert_eq!(r["pushed"], true);
    assert!(r["commit"].is_string() && r["version"] == r["commit"]);
    let text = file(&w.a, r["id"].as_str().unwrap());
    for field in [
        "title:",
        "description:",
        "status: active",
        "created:",
        "updated:",
        "created_by: ",
        "updated_by: ",
    ] {
        assert!(text.contains(field), "{field} missing in {text}");
    }
    assert!(text.contains("Anna Example <anna@example.org>"));
    // The commit reached the remote under the author, with the fixed subject and no agent tag.
    assert_eq!(
        remote_log(&w)[0],
        "waymark: add \"Payment API v3 uses idempotency keys\""
    );
    let author = git(&w.remote, &["log", "-1", "--format=%an <%ae>"]);
    assert_eq!(author.trim(), "Anna Example <anna@example.org>");
    let body = git(&w.remote, &["log", "-1", "--format=%b"]);
    assert!(body.trim().is_empty());
}

#[test]
fn add_rejects_secret() {
    let w = World::new();
    let mut c = client(&w, Person::A);
    let before = remote_log(&w);
    let r = c.call(
        "add",
        json!({"folder": "ops", "title": "Deploy keys", "description": "About keys.", "body": "line\npassword: hunter2hunter2"}),
    );
    assert!(r.is_error);
    assert_eq!(r.code(), Some("validation_failed"));
    assert!(r.message().contains("line 2"), "{}", r.message());
    assert!(!r.json.to_string().contains("hunter2"));
    assert!(!exists(&w.a, "ops/deploy-keys"));
    assert_eq!(remote_log(&w), before);
}

#[test]
fn add_rejects_duplicate_title() {
    let w = World::new();
    let mut c = client(&w, Person::A);
    let first = add(&mut c, "ops", "How we deploy", "Body.");
    let r = c.call(
        "add",
        json!({"folder": "ops", "title": "how we DEPLOY", "description": "Again.", "body": "Other."}),
    );
    assert_eq!(r.code(), Some("validation_failed"));
    assert!(r.message().contains(first["id"].as_str().unwrap()));
    assert!(r.json["next"].is_string());
}

#[test]
fn add_rejects_near_duplicate() {
    let w = World::new();
    let mut c = client(&w, Person::A);
    let first = add(&mut c, "ops", "one two three four five", "Body.");
    let r = c.call(
        "add",
        json!({"folder": "ops", "title": "one two three four six seven eight", "description": "D.", "body": "B."}),
    );
    assert_eq!(r.code(), Some("validation_failed"));
    assert!(r.message().contains("similar"));
    assert!(r.message().contains(first["id"].as_str().unwrap()));
    // 3 of 5 shared words pass.
    let ok = c.call(
        "add",
        json!({"folder": "ops", "title": "one two three x y z", "description": "D.", "body": "B."}),
    );
    assert!(!ok.is_error, "{}", ok.json);
}

#[test]
fn update_stale_base() {
    let w = World::new();
    let mut a = client(&w, Person::A);
    let mut b = client(&w, Person::B);
    let r = add(&mut a, "ops", "Runbook", "One.");
    let id = r["id"].as_str().unwrap();
    let old = r["version"].as_str().unwrap();
    // Person B changes the document first.
    let u = b.call(
        "update_document",
        json!({"id": id, "base": old, "body": "Two."}),
    );
    assert!(!u.is_error, "{}", u.json);
    let stale = a.call(
        "update_document",
        json!({"id": id, "base": old, "body": "Three."}),
    );
    assert_eq!(stale.code(), Some("stale_base"));
    assert!(stale.json["next"].as_str().unwrap().contains("show"));
}

#[test]
fn update_partial_keeps_body() {
    let w = World::new();
    let mut c = client(&w, Person::A);
    let r = add(&mut c, "ops", "Runbook", "Line one.\n\n- item\n");
    let id = r["id"].as_str().unwrap();
    let before = file(&w.a, id);
    let body_before = before.split_once("\n---\n").unwrap().1.to_string();
    let u = c.call(
        "update_document",
        json!({"id": id, "base": r["version"], "description": "A new description."}),
    );
    assert!(!u.is_error, "{}", u.json);
    assert_eq!(u.json["changed"], json!(["description"]));
    let after = file(&w.a, id);
    assert_eq!(after.split_once("\n---\n").unwrap().1, body_before);
    assert!(after.contains("description: A new description."));
    assert_eq!(remote_log(&w)[0], "waymark: update \"Runbook\"");
}

#[test]
fn update_with_no_change_commits_nothing() {
    let w = World::new();
    let mut c = client(&w, Person::A);
    let r = add(&mut c, "ops", "Runbook", "Body.");
    let before = remote_log(&w);
    let u = c.call(
        "update_document",
        json!({"id": r["id"], "base": r["version"], "title": "Runbook"}),
    );
    assert!(!u.is_error, "{}", u.json);
    assert_eq!(u.json["message"], "No change.");
    assert!(u.json["commit"].is_null());
    assert_eq!(remote_log(&w), before);
    let e = c.call(
        "update_document",
        json!({"id": r["id"], "base": r["version"]}),
    );
    assert_eq!(e.code(), Some("invalid_arguments"));
}

#[test]
fn update_title_keeps_id() {
    let w = World::new();
    let mut c = client(&w, Person::A);
    let r = add(&mut c, "ops", "Old title", "Body.");
    let id = r["id"].as_str().unwrap();
    let u = c.call(
        "update_document",
        json!({"id": id, "base": r["version"], "title": "A better title"}),
    );
    assert!(!u.is_error, "{}", u.json);
    assert_eq!(u.json["id"], id);
    assert!(exists(&w.a, id));
    assert!(file(&w.a, id).contains("title: A better title"));
    assert_eq!(remote_log(&w)[0], "waymark: update \"A better title\"");
}

#[test]
fn preview_writes_nothing() {
    let w = World::new();
    let mut c = client(&w, Person::A);
    let before = remote_log(&w);
    let p = c.call(
        "add",
        json!({"folder": "ops", "title": "Preview me", "description": "D.", "body": "B.", "preview": true}),
    );
    assert!(!p.is_error, "{}", p.json);
    assert_eq!(p.json["preview"], true);
    assert!(p.json["commit"].is_null() && p.json["pushed"].is_null());
    assert_eq!(
        p.json["changes"],
        json!([{"id": "ops/preview-me", "action": "create"}])
    );
    assert!(p.json["text"]
        .as_str()
        .unwrap()
        .contains("title: Preview me"));
    assert!(!exists(&w.a, "ops/preview-me"));
    assert_eq!(remote_log(&w), before);
    assert_eq!(subjects(&w.a), before);
    assert!(git(&w.a, &["status", "--porcelain"]).trim().is_empty());
}

#[test]
fn dispute_sets_fields() {
    let w = World::new();
    let mut c = client(&w, Person::A);
    let r = add(&mut c, "ops", "Runbook", "Body.");
    let id = r["id"].as_str().unwrap();
    let d = c.call(
        "dispute",
        json!({"id": id, "base": r["version"], "reason": "The port is wrong."}),
    );
    assert!(!d.is_error, "{}", d.json);
    assert_eq!(d.json["disputed"], true);
    let text = file(&w.a, id);
    assert!(text.contains("disputed: true") && text.contains("dispute_reason: The port is wrong."));
    assert_eq!(remote_log(&w)[0], "waymark: dispute \"Runbook\"");
    // A second dispute fails and names the old reason.
    let again = c.call(
        "dispute",
        json!({"id": id, "base": d.json["version"], "reason": "Other."}),
    );
    assert_eq!(again.code(), Some("validation_failed"));
    assert!(again.message().contains("The port is wrong."));
}

#[test]
fn dispute_clear() {
    let w = World::new();
    let mut c = client(&w, Person::A);
    let r = add(&mut c, "ops", "Runbook", "Body.");
    let id = r["id"].as_str().unwrap();
    let not_disputed = c.call(
        "dispute",
        json!({"id": id, "base": r["version"], "clear": true}),
    );
    assert_eq!(not_disputed.code(), Some("validation_failed"));
    let both = c.call(
        "dispute",
        json!({"id": id, "base": r["version"], "clear": true, "reason": "x"}),
    );
    assert_eq!(both.code(), Some("invalid_arguments"));
    let d = c.call(
        "dispute",
        json!({"id": id, "base": r["version"], "reason": "Wrong."}),
    );
    let cl = c.call(
        "dispute",
        json!({"id": id, "base": d.json["version"], "clear": true}),
    );
    assert!(!cl.is_error, "{}", cl.json);
    assert_eq!(cl.json["disputed"], false);
    let text = file(&w.a, id);
    assert!(!text.contains("disputed") && !text.contains("dispute_reason"));
}

#[test]
fn deprecate_keeps_file() {
    let w = World::new();
    let mut c = client(&w, Person::A);
    let r = add(&mut c, "ops", "Runbook", "Body.");
    let id = r["id"].as_str().unwrap();
    let d = c.call(
        "deprecate",
        json!({"id": id, "base": r["version"], "reason": "Replaced by the new process."}),
    );
    assert!(!d.is_error, "{}", d.json);
    let text = file(&w.a, id);
    assert!(text.contains("status: deprecated"));
    // The reason is in the commit message body, not in the file.
    assert!(!text.contains("Replaced by the new process."));
    let msg = git(&w.remote, &["log", "-1", "--format=%B"]);
    assert!(
        msg.starts_with("waymark: deprecate \"Runbook\"\n\nReason: Replaced by the new process."),
        "{msg}"
    );
    let again = c.call(
        "deprecate",
        json!({"id": id, "base": d.json["version"], "reason": "x"}),
    );
    assert_eq!(again.code(), Some("validation_failed"));
}

#[test]
fn add_supersedes_one_commit() {
    let w = World::new();
    let mut c = client(&w, Person::A);
    let old = add(&mut c, "ops", "Deploy process", "Old.");
    let count_before = remote_log(&w).len();
    let r = c.call(
        "add",
        json!({"folder": "ops", "title": "Deploy process", "description": "New.", "body": "New.", "supersedes": old["id"]}),
    );
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(r.json["deprecated"], json!([old["id"]]));
    assert_eq!(remote_log(&w).len(), count_before + 1);
    let old_text = file(&w.a, old["id"].as_str().unwrap());
    assert!(old_text.contains("status: deprecated"));
    assert!(!old_text.contains(r.json["id"].as_str().unwrap()));
    let new_text = file(&w.a, r.json["id"].as_str().unwrap());
    assert!(new_text.contains(&format!("[[{}]]", old["id"].as_str().unwrap())));
    // The new file name does not clash with the old one.
    assert_ne!(r.json["id"], old["id"]);
}

#[test]
fn delete_refuses_linked_document() {
    let w = World::new();
    let mut c = client(&w, Person::A);
    let old = add(&mut c, "ops", "Old deploy process", "Old.");
    let old_id = old["id"].as_str().unwrap();
    let linking = c.call(
        "add",
        json!({
            "folder": "ops", "title": "Runbook", "description": "D.",
            "body": format!("See [[{old_id}]] and [the old one](old-deploy-process.md).\n\n```\n[[{old_id}]]\n```"),
            "related": [old_id]
        }),
    );
    assert!(!linking.is_error, "{}", linking.json);
    let lid = linking.json["id"].as_str().unwrap().to_string();
    let head = remote_log(&w);
    let d = c.call("delete", json!({"id": old_id, "base": old["version"]}));
    assert_eq!(d.code(), Some("validation_failed"), "{}", d.json);
    assert!(
        d.message().contains("1 document still links"),
        "{}",
        d.message()
    );
    let from = d.json["data"]["linked_from"].as_array().unwrap();
    assert!(from
        .iter()
        .any(|f| f["id"] == lid.as_str() && f["where"] == "body" && f["count"] == 2));
    assert!(from
        .iter()
        .any(|f| f["id"] == lid.as_str() && f["where"] == "property" && f["count"] == 1));
    assert!(d.json["next"].as_str().unwrap().contains("update_document"));
    // Nothing changed.
    assert!(exists(&w.a, old_id));
    assert_eq!(remote_log(&w), head);
    // A preview fails the same way.
    let p = c.call(
        "delete",
        json!({"id": old_id, "base": old["version"], "preview": true}),
    );
    assert_eq!(p.code(), Some("validation_failed"));
}

#[test]
fn delete_after_links_removed() {
    let w = World::new();
    let mut c = client(&w, Person::A);
    let old = add(&mut c, "ops", "Old deploy process", "Old.");
    let old_id = old["id"].as_str().unwrap();
    let linking = add(&mut c, "ops", "Runbook", &format!("See [[{old_id}]]."));
    let lid = linking["id"].as_str().unwrap();
    let shown = c.call("show", json!({"id": lid}));
    let u = c.call(
        "update_document",
        json!({"id": lid, "base": shown.json["version"], "body": "See nothing."}),
    );
    assert!(!u.is_error, "{}", u.json);
    let d = c.call("delete", json!({"id": old_id, "base": old["version"]}));
    assert!(!d.is_error, "{}", d.json);
    assert!(!exists(&w.a, old_id));
    assert_eq!(remote_log(&w)[0], "waymark: delete \"Old deploy process\"");
    assert!(d.message().contains("Git history"));
}

#[test]
fn delete_ignores_a_generated_index_file() {
    let w = World::new();
    let mut c = client(&w, Person::A);
    let a = add(&mut c, "ops", "Alpha", "A.");
    assert!(!c.call("index", json!({})).is_error);
    assert!(exists(&w.a, "ops/index"));
    let d = c.call("delete", json!({"id": a["id"], "base": a["version"]}));
    assert!(!d.is_error, "{}", d.json);
}

#[test]
fn delete_hidden_when_denied() {
    let w = World::new();
    let mut c = McpClient::for_person(&w, Person::A, Spawn::args(&["--tools-denied", "delete"]));
    let names = c.tool_names();
    assert!(names.contains(&"add".to_string()));
    assert!(!names.contains(&"delete".to_string()));
    let r = c.call("delete", json!({"id": "x", "base": "y"}));
    assert_eq!(r.code(), Some("access_denied"));
}

#[test]
fn read_only_blocks_writes() {
    let w = World::new();
    let mut c = McpClient::for_person(&w, Person::A, Spawn::args(&["--access", "read-only"]));
    let names = c.tool_names();
    for t in ["add", "update_document", "dispute", "deprecate", "delete"] {
        assert!(!names.contains(&t.to_string()), "{t} must be hidden");
    }
    let before = remote_log(&w);
    let r = c.call(
        "add",
        json!({"folder": "ops", "title": "T", "description": "D.", "body": "B."}),
    );
    assert_eq!(r.code(), Some("access_denied"));
    assert_eq!(remote_log(&w), before);
    assert_eq!(subjects(&w.a), before);
}

#[test]
fn add_refuses_broken_body_link() {
    let w = World::new();
    let mut c = client(&w, Person::A);
    let r = c.call(
        "add",
        json!({"folder": "ops", "title": "Linker", "description": "D.", "body": "See [[ops/missing-one]] for more."}),
    );
    assert_eq!(r.code(), Some("validation_failed"), "{}", r.json);
    assert!(r.message().contains("ops/missing-one"), "{}", r.message());
    assert!(r.json["next"]
        .as_str()
        .unwrap()
        .contains("Write the linked document first"));
    assert!(!exists(&w.a, "ops/linker"));
}

#[test]
fn add_accepts_urls_other_files_and_code_in_the_body() {
    let w = World::new();
    let mut c = client(&w, Person::A);
    let body = "See [site](https://example.org/x.md), ![pic](img.png), [top](#top).\n\n`[x](gone.md)`\n\n```\n[[ops/gone]]\n```";
    add(&mut c, "ops", "Plain links", body);
    assert!(exists(&w.a, "ops/plain-links"));
}

#[test]
fn update_keeps_an_old_broken_link_and_refuses_a_new_one() {
    let w = World::new();
    commit_doc(&w.a, "ops/old", "Old", "See [x](gone.md).");
    let mut c = client(&w, Person::A);
    let shown = c.call("show", json!({"id": "ops/old"}));
    let base = shown.json["version"].clone();
    // An unrelated change goes through.
    let u = c.call(
        "update_document",
        json!({"id": "ops/old", "base": base, "title": "Old two"}),
    );
    assert!(!u.is_error, "{}", u.json);
    let shown = c.call("show", json!({"id": "ops/old"}));
    let base = shown.json["version"].clone();
    // A new broken link is refused.
    let r = c.call(
        "update_document",
        json!({"id": "ops/old", "base": base, "body": "See [x](gone.md) and [y](other-gone.md)."}),
    );
    assert_eq!(r.code(), Some("validation_failed"), "{}", r.json);
    assert!(r.message().contains("other-gone"), "{}", r.message());
}

#[test]
fn property_link_to_a_missing_document_fails_with_all_findings() {
    let w = World::new();
    let mut c = client(&w, Person::A);
    let r = c.call(
        "add",
        json!({
            "folder": "ops", "title": "Bad links", "description": "D.", "body": "B.",
            "related": ["ops/nope-one", "ops/nope-two"]
        }),
    );
    assert_eq!(r.code(), Some("validation_failed"));
    assert!(
        r.message().contains("nope-one") && r.message().contains("nope-two"),
        "{}",
        r.message()
    );
    assert!(!exists(&w.a, "ops/bad-links"));
}

#[test]
fn add_outside_folders() {
    let w = World::new();
    let mut c = McpClient::for_person(&w, Person::A, Spawn::args(&["--folders", "engineering/**"]));
    let r = c.call(
        "add",
        json!({"folder": "sales", "title": "T", "description": "D.", "body": "B."}),
    );
    assert_eq!(r.code(), Some("access_denied"));
    assert!(r.message().contains("outside the allowed folders"));
    let ok = c.call(
        "add",
        json!({"folder": "engineering/api", "title": "T", "description": "D.", "body": "B."}),
    );
    assert!(!ok.is_error, "{}", ok.json);
}

#[test]
fn an_id_outside_folders_is_not_found() {
    let w = World::new();
    commit_doc(&w.a, "sales/deal", "Deal", "Body.");
    let mut c = McpClient::for_person(&w, Person::A, Spawn::args(&["--folders", "engineering/**"]));
    let r = c.call(
        "update_document",
        json!({"id": "sales/deal", "base": "abc1234", "body": "x"}),
    );
    assert_eq!(r.code(), Some("not_found"));
}

#[test]
fn delete_outside_folders_for_a_linking_document_is_denied() {
    let w = World::new();
    commit_doc(&w.a, "eng/target", "Target", "Body.");
    commit_doc(&w.a, "sales/linker", "Linker", "See [[eng/target]].");
    let before = remote_log(&w);
    let mut c = McpClient::for_person(&w, Person::A, Spawn::args(&["--folders", "eng/**"]));
    let version = git(
        &w.a,
        &[
            "log",
            "-1",
            "--format=%h",
            "--",
            "agent-memory/eng/target.md",
        ],
    );
    let r = c.call(
        "delete",
        json!({"id": "eng/target", "base": version.trim()}),
    );
    assert_eq!(r.code(), Some("access_denied"));
    assert!(r.message().contains("sales/linker"));
    assert!(exists(&w.a, "eng/target"));
    assert_eq!(remote_log(&w), before);
}

#[test]
fn update_resets_trust() {
    let w = World::new();
    set_shared(&w, "verification = false", "verification = true");
    let mut c = client(&w, Person::A);
    let r = add(&mut c, "ops", "Runbook", "Body.");
    let id = r["id"].as_str().unwrap();
    assert!(file(&w.a, id).contains("trust: unverified"));
    // Verify the document by hand: this feature belongs to `verification`.
    let text = file(&w.a, id).replace(
        "trust: unverified",
        "trust: verified\nverified_by: \"Ben Example <ben@example.org>\"\nverified: 2026-11-14",
    );
    std::fs::write(w.a.join("agent-memory").join(format!("{id}.md")), text).unwrap();
    git(&w.a, &["add", "-A"]);
    git(&w.a, &["commit", "-q", "-m", "test: verify"]);
    git(&w.a, &["push", "-q"]);
    let version = git(
        &w.a,
        &[
            "log",
            "-1",
            "--format=%h",
            "--",
            &format!("agent-memory/{id}.md"),
        ],
    );
    let u = c.call(
        "update_document",
        json!({"id": id, "base": version.trim(), "body": "Changed."}),
    );
    assert!(!u.is_error, "{}", u.json);
    let after = file(&w.a, id);
    assert!(after.contains("trust: unverified"));
    assert!(!after.contains("verified_by") && !after.contains("verified:"));
}

#[test]
fn the_same_add_twice_leaves_the_state_of_the_first() {
    let w = World::new();
    let mut c = client(&w, Person::A);
    add(&mut c, "ops", "Same one", "Body.");
    let state = subjects(&w.a);
    let again = c.call(
        "add",
        json!({"folder": "ops", "title": "Same one", "description": "About Same one.", "body": "Body."}),
    );
    assert_eq!(again.code(), Some("validation_failed"));
    assert_eq!(subjects(&w.a), state);
    assert_eq!(remote_log(&w), state);
}

#[test]
fn two_people_write_different_documents_and_both_reach_the_remote() {
    let w = World::new();
    let mut a = client(&w, Person::A);
    let mut b = client(&w, Person::B);
    add(&mut a, "ops", "Alpha document", "A.");
    // B has not seen the commit of A. The write flow updates the clone first.
    add(&mut b, "ops", "Beta document", "B.");
    let log = remote_log(&w);
    assert!(log.contains(&"waymark: add \"Alpha document\"".to_string()));
    assert!(log.contains(&"waymark: add \"Beta document\"".to_string()));
    let author = git(&w.remote, &["log", "-1", "--format=%an"]);
    assert_eq!(author.trim(), "Ben Example");
}

#[test]
fn a_write_needing_approval_fails_closed_until_write_approval_exists() {
    let w = World::new();
    set_shared(&w, "[approval]\n", "[approval]\nrequired_for = [\"add\"]\n");
    let mut c = client(&w, Person::A);
    let before = remote_log(&w);
    let r = c.call(
        "add",
        json!({"folder": "ops", "title": "T", "description": "D.", "body": "B."}),
    );
    assert_eq!(r.code(), Some("approval_unavailable"));
    assert_eq!(remote_log(&w), before);
    assert!(!exists(&w.a, "ops/t"));
    // A preview asks for no approval.
    let p = c.call(
        "add",
        json!({"folder": "ops", "title": "T", "description": "D.", "body": "B.", "preview": true}),
    );
    assert!(!p.is_error, "{}", p.json);
}

#[test]
fn invalid_input_is_reported_before_anything_is_written() {
    let w = World::new();
    let mut c = client(&w, Person::A);
    for folder in ["../x", "/abs", ".waymark"] {
        let r = c.call(
            "add",
            json!({"folder": folder, "title": "T", "description": "D.", "body": "B."}),
        );
        assert_eq!(r.code(), Some("invalid_arguments"), "{folder}");
    }
    let r = c.call("add", json!({"folder": "ops", "title": "T"}));
    assert_eq!(r.code(), Some("invalid_arguments"));
    // A description over the word limit.
    let long = "word ".repeat(51);
    let r = c.call(
        "add",
        json!({"folder": "ops", "title": "T", "description": long, "body": "B."}),
    );
    assert_eq!(r.code(), Some("validation_failed"));
    assert!(r.json["next"].is_string());
}
