//! End-to-end tests of verification (see `specs/features/verification/test.md`).

mod common;
use common::git::{git, subjects};
use common::{McpClient, Person, Spawn, World};
use serde_json::{json, Value};

fn world_with_verification() -> World {
    let w = World::new();
    set_shared(&w, "verification = false", "verification = true");
    w
}

fn set_shared(w: &World, from: &str, to: &str) {
    let path = w.a.join("agent-memory/.waymark/settings.toml");
    let old = std::fs::read_to_string(&path).unwrap();
    assert!(old.contains(from), "{from}");
    std::fs::write(&path, old.replacen(from, to, 1)).unwrap();
    git(&w.a, &["add", "-A"]);
    git(&w.a, &["commit", "-q", "-m", "test: settings"]);
    git(&w.a, &["push", "-q"]);
}

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

fn file(w: &World, id: &str) -> String {
    std::fs::read_to_string(w.a.join("agent-memory").join(format!("{id}.md"))).unwrap()
}

#[test]
fn tools_are_hidden_and_denied_when_off() {
    let w = World::new();
    let mut c = client(&w);
    let names = c.tool_names();
    for t in ["verify_document", "review", "reject_document"] {
        assert!(!names.contains(&t.to_string()), "{t}");
    }
    let r = c.call("verify_document", json!({"id": "x", "base": "y"}));
    assert_eq!(r.code(), Some("access_denied"));
    assert!(r.message().contains("Verification is off"));
    let r = c.call("review", json!({}));
    assert_eq!(r.code(), Some("access_denied"));
}

#[test]
fn add_sets_unverified_and_verify_sets_the_fields() {
    let w = world_with_verification();
    let mut c = client(&w);
    assert!(c.tool_names().contains(&"verify_document".to_string()));
    let added = add(&mut c, "Doc one");
    assert!(file(&w, "ops/doc-one").contains("trust: unverified"));
    let before = subjects(&w.remote).len();
    let v = c.call(
        "verify_document",
        json!({"id": "ops/doc-one", "base": added["version"]}),
    );
    assert!(!v.is_error, "{}", v.json);
    assert_eq!(v.json["trust"], "verified");
    assert_eq!(v.json["verified_by"], "Anna Example <anna@example.org>");
    let text = file(&w, "ops/doc-one");
    assert!(
        text.contains("trust: verified")
            && text.contains("verified_by: ")
            && text.contains("verified: 20")
    );
    assert_eq!(subjects(&w.remote).len(), before + 1);
    assert_eq!(subjects(&w.remote)[0], "waymark: update \"Doc one\"");
    assert!(git(&w.remote, &["log", "-1", "--format=%b"]).contains("Trust: verified"));
    // Twice, and an old base.
    let again = c.call(
        "verify_document",
        json!({"id": "ops/doc-one", "base": v.json["version"]}),
    );
    assert_eq!(again.code(), Some("validation_failed"));
    let stale = c.call(
        "verify_document",
        json!({"id": "ops/doc-one", "base": added["version"]}),
    );
    assert_eq!(stale.code(), Some("stale_base"));
}

#[test]
fn a_body_change_resets_trust_and_undo_of_verify_restores_the_document() {
    let w = world_with_verification();
    let mut c = client(&w);
    let added = add(&mut c, "Doc one");
    let before = file(&w, "ops/doc-one");
    let v = c.call(
        "verify_document",
        json!({"id": "ops/doc-one", "base": added["version"]}),
    );
    let u = c.call(
        "update_document",
        json!({"id": "ops/doc-one", "base": v.json["version"], "body": "Changed."}),
    );
    assert!(!u.is_error, "{}", u.json);
    let text = file(&w, "ops/doc-one");
    assert!(
        text.contains("trust: unverified")
            && !text.contains("verified_by")
            && !text.contains("verified:")
    );
    // Undo of the verify commit would conflict with the later update: undo the update first.
    assert!(!c.call("undo", json!({"commit": u.json["commit"]})).is_error);
    assert!(file(&w, "ops/doc-one").contains("trust: verified"));
    assert!(!c.call("undo", json!({"commit": v.json["commit"]})).is_error);
    assert_eq!(file(&w, "ops/doc-one"), before);
}

#[test]
fn verify_fails_for_a_disputed_or_deprecated_document() {
    let w = world_with_verification();
    let mut c = client(&w);
    let a = add(&mut c, "Doc one");
    let d = c.call(
        "dispute_document",
        json!({"id": "ops/doc-one", "base": a["version"], "reason": "Wrong."}),
    );
    let r = c.call(
        "verify_document",
        json!({"id": "ops/doc-one", "base": d.json["version"]}),
    );
    assert_eq!(r.code(), Some("validation_failed"));
    assert!(r.json["next"].as_str().unwrap().contains("clear"));
    let b = add(&mut c, "Doc two");
    let dep = c.call(
        "deprecate_document",
        json!({"id": "ops/doc-two", "base": b["version"], "reason": "Old."}),
    );
    let r = c.call(
        "verify_document",
        json!({"id": "ops/doc-two", "base": dep.json["version"]}),
    );
    assert_eq!(r.code(), Some("validation_failed"));
}

#[test]
fn reject_keeps_the_file_as_deprecated_and_clears_trust() {
    let w = world_with_verification();
    let mut c = client(&w);
    let a = add(&mut c, "Doc one");
    let v = c.call(
        "verify_document",
        json!({"id": "ops/doc-one", "base": a["version"]}),
    );
    let r = c.call(
        "reject_document",
        json!({"id": "ops/doc-one", "base": v.json["version"], "reason": "It is wrong."}),
    );
    assert!(!r.is_error, "{}", r.json);
    let text = file(&w, "ops/doc-one");
    assert!(text.contains("status: deprecated") && text.contains("trust: unverified"));
    assert!(!text.contains("verified_by") && !text.contains("verified:"));
    let msg = git(&w.remote, &["log", "-1", "--format=%B"]);
    assert!(
        msg.starts_with("waymark: deprecate \"Doc one\"\n\nReason: It is wrong.\nTrust: rejected"),
        "{msg}"
    );
    let again = c.call(
        "reject_document",
        json!({"id": "ops/doc-one", "base": r.json["version"], "reason": "x"}),
    );
    assert_eq!(again.code(), Some("validation_failed"));
}

#[test]
fn review_lists_disputed_first_then_oldest() {
    let w = world_with_verification();
    let mut c = client(&w);
    let a = add(&mut c, "Doc one");
    add(&mut c, "Doc two");
    add(&mut c, "Doc three");
    let two = c.call("show", json!({"id": "ops/doc-two"}));
    c.call(
        "dispute_document",
        json!({"id": "ops/doc-two", "base": two.json["version"], "reason": "Check this."}),
    );
    let v = c.call(
        "verify_document",
        json!({"id": "ops/doc-one", "base": a["version"]}),
    );
    assert!(!v.is_error);
    let r = c.call("review", json!({}));
    assert!(!r.is_error, "{}", r.json);
    let ids: Vec<&str> = r.json["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, vec!["ops/doc-two", "ops/doc-three"]);
    assert_eq!(r.json["total"], 2);
    assert_eq!(r.json["entries"][0]["dispute_reason"], "Check this.");
    assert_eq!(r.json["entries"][0]["disputed"], true);
    assert!(r.json["entries"][0]["version"].is_string());
    let limited = c.call("review", json!({"limit": 1}));
    assert_eq!(limited.json["entries"].as_array().unwrap().len(), 1);
    assert_eq!(limited.json["total"], 2);
}

#[test]
fn a_document_without_trust_is_in_review_and_the_folder_limit_applies() {
    let w = World::new();
    // A document that was written before verification: no trust field.
    std::fs::create_dir_all(w.a.join("agent-memory/eng")).unwrap();
    std::fs::write(
        w.a.join("agent-memory/eng/old.md"),
        "---\ntitle: Old one\ndescription: About old.\nstatus: active\ncreated: 2020-01-01\nupdated: 2020-01-01\ncreated_by: \"A <a@example.org>\"\nupdated_by: \"A <a@example.org>\"\n---\n\nx\n",
    )
    .unwrap();
    std::fs::create_dir_all(w.a.join("agent-memory/ops")).unwrap();
    std::fs::write(
        w.a.join("agent-memory/ops/other.md"),
        "---\ntitle: Other one\ndescription: About other.\nstatus: active\ncreated: 2020-01-01\nupdated: 2020-01-01\ncreated_by: \"A <a@example.org>\"\nupdated_by: \"A <a@example.org>\"\n---\n\nx\n",
    )
    .unwrap();
    git(&w.a, &["add", "-A"]);
    git(&w.a, &["commit", "-q", "-m", "test: docs"]);
    git(&w.a, &["push", "-q"]);
    set_shared(&w, "verification = false", "verification = true");
    let mut c = client(&w);
    let r = c.call("review", json!({}));
    assert_eq!(r.json["total"], 2);
    assert_eq!(r.json["entries"][0]["trust"], "unverified");
    let f = c.call("review", json!({"folder": "eng"}));
    assert_eq!(f.json["total"], 1);
    let mut limited = McpClient::for_person(&w, Person::A, Spawn::args(&["--folders", "ops/**"]));
    let r = limited.call("review", json!({}));
    assert_eq!(r.json["total"], 1);
    assert_eq!(r.json["entries"][0]["id"], "ops/other");
}

#[test]
fn dispute_and_deprecate_keep_a_verified_mark_and_turning_off_keeps_fields() {
    let w = world_with_verification();
    let mut c = client(&w);
    let a = add(&mut c, "Doc one");
    let v = c.call(
        "verify_document",
        json!({"id": "ops/doc-one", "base": a["version"]}),
    );
    let d = c.call(
        "dispute_document",
        json!({"id": "ops/doc-one", "base": v.json["version"], "reason": "Hmm."}),
    );
    assert!(file(&w, "ops/doc-one").contains("trust: verified"));
    c.call(
        "deprecate_document",
        json!({"id": "ops/doc-one", "base": d.json["version"], "reason": "Old."}),
    );
    assert!(file(&w, "ops/doc-one").contains("trust: verified"));
    // Turn verification off: fields stay, `add` stops setting trust.
    set_shared(&w, "verification = true", "verification = false");
    let mut c = client(&w);
    add(&mut c, "Doc two");
    assert!(!file(&w, "ops/doc-two").contains("trust"));
    assert!(file(&w, "ops/doc-one").contains("trust: verified"));
}
