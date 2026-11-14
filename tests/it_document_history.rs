//! End-to-end tests of `history` and `show` with a version
//! (see `specs/features/document-history/test.md`).

mod common;
use common::{McpClient, Person, Spawn, World};
use serde_json::{json, Value};

fn client(w: &World) -> McpClient {
    McpClient::for_person(w, Person::A, Spawn::default())
}

fn ok(r: common::Reply) -> Value {
    assert!(!r.is_error, "{}", r.json);
    r.json
}

/// A document with three changes. Returns its ID.
fn three_versions(c: &mut McpClient) -> String {
    let r = ok(c.call(
        "add_document",
        json!({"folder": "ops", "title": "Runbook", "description": "About the runbook.", "body": "first text"}),
    ));
    let id = r["id"].as_str().unwrap().to_string();
    for text in ["second text", "third text"] {
        let base = ok(c.call("show", json!({"id": id})))["version"]
            .as_str()
            .unwrap()
            .to_string();
        ok(c.call(
            "update_document",
            json!({"id": id, "base": base, "body": text}),
        ));
    }
    id
}

fn versions(v: &Value) -> Vec<String> {
    v["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["version"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn history_lists_the_versions_newest_first() {
    let w = World::new();
    let mut c = client(&w);
    let id = three_versions(&mut c);
    let h = ok(c.call("history", json!({"id": id})));
    let entries = h["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0]["verb"], "update");
    assert_eq!(entries[2]["verb"], "add");
    assert_eq!(entries[0]["author"], "Anna Example <anna@example.org>");
    assert_eq!(entries[0]["subject"], "waymark: update \"Runbook\"");
    assert!(entries[0]["time"].as_str().unwrap().starts_with("20"));
    assert!(h["next_before"].is_null());
    let current = ok(c.call("show", json!({"id": id})));
    assert_eq!(entries[0]["version"], current["version"]);
}

#[test]
fn history_pages_with_before() {
    let w = World::new();
    let mut c = client(&w);
    let id = three_versions(&mut c);
    let p1 = ok(c.call("history", json!({"id": id, "limit": 2})));
    assert_eq!(p1["entries"].as_array().unwrap().len(), 2);
    assert!(p1["next_before"].is_string());
    let p2 = ok(c.call(
        "history",
        json!({"id": id, "limit": 2, "before": p1["next_before"]}),
    ));
    assert_eq!(p2["entries"].as_array().unwrap().len(), 1);
    assert!(p2["next_before"].is_null());
    let mut all = versions(&p1);
    all.extend(versions(&p2));
    all.dedup();
    assert_eq!(all.len(), 3, "no repeats");
    let r = c.call("history", json!({"id": id, "limit": 0}));
    assert_eq!(r.code(), Some("invalid_arguments"));
}

#[test]
fn history_follows_a_move() {
    let w = World::new();
    let mut c = client(&w);
    let id = three_versions(&mut c);
    ok(c.call("move", json!({"from": id, "to": "platform/runbook"})));
    let h = ok(c.call("history", json!({"id": "platform/runbook"})));
    let verbs: Vec<&str> = h["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["verb"].as_str().unwrap_or(""))
        .collect();
    assert_eq!(verbs, ["move", "update", "update", "add"], "{h}");
}

#[test]
fn show_with_an_old_version_returns_the_old_text() {
    let w = World::new();
    let mut c = client(&w);
    let id = three_versions(&mut c);
    let h = ok(c.call("history", json!({"id": id})));
    let v = versions(&h);
    let old = ok(c.call("show", json!({"id": id, "version": v[2]})));
    assert_eq!(old["body"], "first text");
    assert_eq!(old["is_current"], false);
    assert_eq!(old["version"], v[2]);
    assert_eq!(old["current_version"], v[0]);
    assert_eq!(old["title"], "Runbook");
    assert!(old["notices"].to_string().contains("older version"));
    let mid = ok(c.call("show", json!({"id": id, "version": v[1]})));
    assert_eq!(mid["body"], "second text");
}

#[test]
fn show_with_the_current_version_is_the_current_document() {
    let w = World::new();
    let mut c = client(&w);
    let id = three_versions(&mut c);
    let plain = ok(c.call("show", json!({"id": id})));
    let same = ok(c.call("show", json!({"id": id, "version": plain["version"]})));
    assert_eq!(same["is_current"], true);
    assert_eq!(same["body"], "third text");
    for key in [
        "title",
        "description",
        "version",
        "status",
        "body",
        "backlinks_total",
    ] {
        assert_eq!(plain[key], same[key], "{key}");
    }
}

#[test]
fn history_and_show_refuse_unknown_hidden_and_foreign_versions() {
    let w = World::new();
    let mut c = client(&w);
    let id = three_versions(&mut c);
    let other = ok(c.call(
        "add_document",
        json!({"folder": "ops", "title": "Other", "description": "About other.", "body": "x"}),
    ));
    let other_version = other["version"].as_str().unwrap().to_string();
    assert_eq!(
        c.call("history", json!({"id": "ops/nope"})).code(),
        Some("not_found")
    );
    assert_eq!(
        c.call("history", json!({"id": "../x"})).code(),
        Some("invalid_arguments")
    );
    assert_eq!(
        c.call("history", json!({"id": id, "before": other_version}))
            .code(),
        Some("invalid_arguments")
    );
    assert_eq!(
        c.call("show", json!({"id": id, "version": other_version}))
            .code(),
        Some("invalid_arguments")
    );
    assert_eq!(
        c.call("show", json!({"id": id, "version": "zzzzzzz"}))
            .code(),
        Some("invalid_arguments")
    );
    // A hidden document is not found.
    let p = w.root.join("home-a/.config/waymark/config.toml");
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(&p, "[limits]\nfolders = [\"work/**\"]\n").unwrap();
    let mut c2 = client(&w);
    assert_eq!(
        c2.call("history", json!({"id": id})).code(),
        Some("not_found")
    );
}

#[test]
fn show_with_a_version_counts_the_document_once() {
    let w = World::new();
    let mut c = McpClient::for_person(&w, Person::A, Spawn::args(&["--max-documents-opened", "1"]));
    let id = three_versions(&mut c);
    // `add_document` and `update_document` do not open documents. `show` opens this one.
    let h = ok(c.call("history", json!({"id": id})));
    let v = versions(&h);
    ok(c.call("show", json!({"id": id})));
    ok(c.call("show", json!({"id": id, "version": v[2]})));
    ok(c.call("show", json!({"id": id, "version": v[1]})));
    let other = ok(c.call(
        "add_document",
        json!({"folder": "ops", "title": "Other", "description": "About other.", "body": "x"}),
    ));
    let r = c.call("show", json!({"id": other["id"]}));
    assert_eq!(
        r.code(),
        Some("access_denied"),
        "the budget of 1 is used up"
    );
}
