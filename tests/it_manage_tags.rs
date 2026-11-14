//! End-to-end tests of `list_tags` and `rename_tag` (see `specs/features/manage-tags/test.md`).

mod common;
use common::git::{git, subjects};
use common::{McpClient, Person, Spawn, World};
use serde_json::{json, Value};
use std::path::Path;

fn client(w: &World) -> McpClient {
    McpClient::for_person(w, Person::A, Spawn::default())
}

fn ok(r: common::Reply) -> Value {
    assert!(!r.is_error, "{}", r.json);
    r.json
}

fn add(c: &mut McpClient, folder: &str, title: &str, tags: &[&str]) -> String {
    let r = ok(c.call(
        "add_document",
        json!({"folder": folder, "title": title, "description": format!("About {title}."), "body": "x", "tags": tags}),
    ));
    r["id"].as_str().unwrap().to_string()
}

fn file(dir: &Path, id: &str) -> String {
    std::fs::read_to_string(dir.join("agent-memory").join(format!("{id}.md"))).unwrap()
}

fn counts(v: &Value) -> Vec<(String, u64)> {
    v["tags"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| {
            (
                t["tag"].as_str().unwrap().to_string(),
                t["documents"].as_u64().unwrap(),
            )
        })
        .collect()
}

#[test]
fn list_tags_counts_documents_and_pages() {
    let w = World::new();
    let mut c = client(&w);
    add(&mut c, "ops", "One", &["payments", "ops"]);
    add(&mut c, "ops", "Two", &["payments"]);
    add(&mut c, "ops", "Three", &["payments", "zeta"]);
    let v = ok(c.call("list_tags", json!({})));
    assert_eq!(
        counts(&v),
        [
            ("payments".to_string(), 3),
            ("ops".to_string(), 1),
            ("zeta".to_string(), 1)
        ]
    );
    assert_eq!(v["total_tags"], 3);
    assert_eq!(v["truncated"], false);
    let p1 = ok(c.call("list_tags", json!({"limit": 2})));
    assert_eq!(p1["truncated"], true);
    assert_eq!(p1["next_offset"], 2);
    let p2 = ok(c.call("list_tags", json!({"limit": 2, "offset": 2})));
    assert_eq!(counts(&p2), [("zeta".to_string(), 1)]);
    assert_eq!(
        c.call("list_tags", json!({"limit": 0})).code(),
        Some("invalid_arguments")
    );
    assert_eq!(
        c.call("list_tags", json!({"offset": -1})).code(),
        Some("invalid_arguments")
    );
    // Deprecated documents count only when asked for.
    let id = add(&mut c, "ops", "Four", &["old-tag"]);
    let base = ok(c.call("show", json!({"id": id})))["version"]
        .as_str()
        .unwrap()
        .to_string();
    ok(c.call(
        "deprecate_document",
        json!({"id": id, "base": base, "reason": "Old."}),
    ));
    assert!(!counts(&ok(c.call("list_tags", json!({}))))
        .iter()
        .any(|t| t.0 == "old-tag"));
    assert!(counts(&ok(c.call("list_tags", json!({"status": "any"}))))
        .iter()
        .any(|t| t.0 == "old-tag"));
}

#[test]
fn rename_tag_changes_all_documents_in_one_commit() {
    let w = World::new();
    let mut c = client(&w);
    let a = add(&mut c, "ops", "One", &["pay", "ops"]);
    let b = add(&mut c, "backend", "Two", &["pay"]);
    let d = add(&mut c, "ops", "Three", &["pay", "x"]);
    let other = add(&mut c, "ops", "Four", &["ops"]);
    let before = subjects(&w.remote).len();
    let r = ok(c.call("rename_tag", json!({"from": "pay", "to": "payments"})));
    assert_eq!(r["documents"], 3);
    assert_eq!(r["pushed"], true);
    assert_eq!(subjects(&w.remote).len(), before + 1, "one commit");
    assert_eq!(
        subjects(&w.remote)[0],
        "waymark: rename-tag \"pay to payments\""
    );
    for id in [&a, &b, &d] {
        let t = file(&w.a, id);
        assert!(t.contains("payments") && !t.contains("- pay\n"), "{t}");
        assert!(t.contains("updated_by: \"Anna Example <anna@example.org>\""));
    }
    assert!(!file(&w.a, &other).contains("payments"));
    let ids: Vec<&str> = r["ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i.as_str().unwrap())
        .collect();
    assert_eq!(ids, {
        let mut v = vec![a.as_str(), b.as_str(), d.as_str()];
        v.sort();
        v
    });
    let tags = counts(&ok(c.call("list_tags", json!({}))));
    assert!(tags.contains(&("payments".to_string(), 3)));
    assert!(!tags.iter().any(|t| t.0 == "pay"));
}

#[test]
fn rename_tag_merges_into_an_existing_tag() {
    let w = World::new();
    let mut c = client(&w);
    let both = add(&mut c, "ops", "Both", &["pay", "payments"]);
    let only = add(&mut c, "ops", "Only", &["pay"]);
    ok(c.call("rename_tag", json!({"from": "pay", "to": "payments"})));
    let t = file(&w.a, &both);
    assert_eq!(t.matches("payments").count(), 1, "{t}");
    assert!(!t.contains("- pay\n"));
    assert!(file(&w.a, &only).contains("payments"));
}

#[test]
fn rename_tag_refuses_an_unknown_tag() {
    let w = World::new();
    let mut c = client(&w);
    add(&mut c, "ops", "One", &["a"]);
    let r = c.call("rename_tag", json!({"from": "nope", "to": "b"}));
    assert_eq!(r.code(), Some("not_found"));
    assert!(r.json["next"].as_str().unwrap().contains("list_tags"));
}

#[test]
fn rename_tag_refuses_bad_tags() {
    let w = World::new();
    let mut c = client(&w);
    add(&mut c, "ops", "One", &["a"]);
    for (from, to) in [
        ("a", "a"),
        ("a", "#b"),
        ("a", "b c"),
        ("a", ""),
        ("", "b"),
        ("a b", "c"),
    ] {
        let r = c.call("rename_tag", json!({"from": from, "to": to}));
        assert_eq!(
            r.code(),
            Some("invalid_arguments"),
            "{from:?} {to:?}: {}",
            r.json
        );
        assert!(r.json["next"].is_string());
    }
}

#[test]
fn tag_tools_respect_the_folders_limit() {
    let w = World::new();
    let mut c = client(&w);
    let inside = add(&mut c, "ops", "Inside", &["pay"]);
    let outside = add(&mut c, "work", "Outside", &["pay"]);
    let p = w.root.join("home-a/.config/waymark/config.toml");
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(&p, "[limits]\nfolders = [\"ops/**\"]\n").unwrap();
    let mut c = client(&w);
    assert_eq!(
        counts(&ok(c.call("list_tags", json!({})))),
        [("pay".to_string(), 1)]
    );
    let r = ok(c.call("rename_tag", json!({"from": "pay", "to": "payments"})));
    assert_eq!(r["documents"], 1);
    assert!(file(&w.a, &inside).contains("payments"));
    assert!(!file(&w.a, &outside).contains("payments"));
    // Nothing visible has the tag any more.
    assert_eq!(
        c.call("rename_tag", json!({"from": "pay", "to": "x"}))
            .code(),
        Some("not_found")
    );
}

#[test]
fn rename_tag_preview_writes_nothing() {
    let w = World::new();
    let mut c = client(&w);
    let id = add(&mut c, "ops", "One", &["pay"]);
    let before = file(&w.a, &id);
    let log = subjects(&w.remote);
    let r = ok(c.call(
        "rename_tag",
        json!({"from": "pay", "to": "payments", "preview": true}),
    ));
    assert_eq!(r["preview"], true);
    assert_eq!(r["documents"], 1);
    assert_eq!(r["changes"][0]["id"], id);
    assert_eq!(file(&w.a, &id), before);
    assert_eq!(subjects(&w.remote), log);
}

#[test]
fn rename_tag_follows_the_limits_and_undo_reverts_it() {
    let w = World::new();
    let mut c = client(&w);
    let id = add(&mut c, "ops", "One", &["pay"]);
    let p = w.root.join("home-a/.config/waymark/config.toml");
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    for (limits, code) in [
        ("[limits]\naccess = \"read-only\"\n", "access_denied"),
        ("[limits]\npaused = true\n", "paused"),
        (
            "[limits]\ntools_denied = [\"rename_tag\"]\n",
            "access_denied",
        ),
    ] {
        std::fs::write(&p, limits).unwrap();
        let mut c2 = client(&w);
        let r = c2.call("rename_tag", json!({"from": "pay", "to": "payments"}));
        assert_eq!(r.code(), Some(code), "{limits}");
    }
    std::fs::remove_file(&p).unwrap();
    let mut c = client(&w);
    let r = ok(c.call("rename_tag", json!({"from": "pay", "to": "payments"})));
    assert!(file(&w.a, &id).contains("payments"));
    let u = ok(c.call("undo", json!({"commit": r["commit"]})));
    assert!(u["reverted"].is_array());
    let t = file(&w.a, &id);
    assert!(t.contains("pay") && !t.contains("payments"), "{t}");
    let _ = git;
}
