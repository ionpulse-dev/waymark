//! End-to-end tests of `folders` (see `specs/features/folders/test.md`).

mod common;
use common::git::{git, subjects};
use common::memrepo::commit_doc;
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

fn paths(v: &Value) -> Vec<String> {
    v["folders"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["path"].as_str().unwrap().to_string())
        .collect()
}

fn entry<'a>(v: &'a Value, path: &str) -> &'a Value {
    v["folders"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["path"] == path)
        .unwrap_or_else(|| panic!("no folder {path} in {v}"))
}

fn on_disk(dir: &Path, rel: &str) -> bool {
    dir.join("agent-memory").join(rel).exists()
}

fn personal(w: &World, text: &str) {
    let p = w.root.join("home-a/.config/waymark/config.toml");
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, text).unwrap();
}

/// ops (1 document), ops/deploy (2), ops/deploy/old (1), work (1).
fn tree(w: &World) {
    commit_doc(&w.a, "ops/a", "Alpha", "A.");
    commit_doc(&w.a, "ops/deploy/b", "Beta", "B.");
    commit_doc(&w.a, "ops/deploy/c", "Gamma", "C.");
    commit_doc(&w.a, "ops/deploy/old/d", "Delta", "D.");
    commit_doc(&w.a, "work/e", "Epsilon", "E.");
}

#[test]
fn folders_lists_the_tree_with_counts_down_to_depth_2() {
    let w = World::new();
    tree(&w);
    let mut c = client(&w);
    let v = ok(c.call("folder_tree", json!({})));
    assert_eq!(paths(&v), vec!["ops", "ops/deploy", "work"]);
    assert_eq!(entry(&v, "ops")["documents"], 1);
    assert_eq!(entry(&v, "ops")["total"], 4);
    assert_eq!(entry(&v, "ops/deploy")["documents"], 2);
    assert_eq!(entry(&v, "ops/deploy")["total"], 3);
    assert_eq!(entry(&v, "ops/deploy")["has_more"], true);
    assert_eq!(entry(&v, "ops")["has_more"], false);
    assert_eq!(v["truncated"], false);
}

#[test]
fn depth_1_lists_the_top_level_and_marks_folders_with_more() {
    let w = World::new();
    tree(&w);
    let mut c = client(&w);
    let v = ok(c.call("folder_tree", json!({"depth": 1})));
    assert_eq!(paths(&v), vec!["ops", "work"]);
    assert_eq!(entry(&v, "ops")["has_more"], true);
    let v = ok(c.call("folder_tree", json!({"depth": 3})));
    assert!(paths(&v).contains(&"ops/deploy/old".to_string()));
}

#[test]
fn a_start_folder_lists_below_it_and_an_unknown_one_is_not_found() {
    let w = World::new();
    tree(&w);
    let mut c = client(&w);
    let v = ok(c.call("folder_tree", json!({"folder": "ops"})));
    assert_eq!(paths(&v), vec!["ops/deploy", "ops/deploy/old"]);
    assert_eq!(
        c.call("folder_tree", json!({"folder": "nope"})).code(),
        Some("not_found")
    );
    assert_eq!(
        c.call("folder_tree", json!({"depth": 0})).code(),
        Some("invalid_arguments")
    );
}

#[test]
fn the_folders_limit_hides_other_folders() {
    let w = World::new();
    tree(&w);
    personal(&w, "[limits]\nfolders = [\"work/**\"]\n");
    let mut c = client(&w);
    let v = ok(c.call("folder_tree", json!({})));
    assert_eq!(paths(&v), vec!["work"]);
}

#[test]
fn more_than_200_folders_are_truncated() {
    let w = World::new();
    for i in 0..205 {
        let dir = w.a.join(format!("agent-memory/f{i:03}"));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(".gitkeep"), "").unwrap();
    }
    git(&w.a, &["add", "-A"]);
    git(&w.a, &["commit", "-q", "-m", "test: many folders"]);
    git(&w.a, &["push", "-q"]);
    let mut c = client(&w);
    let v = ok(c.call("folder_tree", json!({})));
    assert_eq!(paths(&v).len(), 200);
    assert_eq!(v["truncated"], true);
    assert!(v["next"].as_str().unwrap().contains("folder"));
}

#[test]
fn add_empty_folder_creates_a_gitkeep_in_one_commit() {
    let w = World::new();
    let mut c = client(&w);
    let r = ok(c.call("create_folder", json!({"folder": "ops/runbooks"})));
    assert_eq!(r["pushed"], true);
    assert_eq!(r["folder"], "ops/runbooks");
    assert!(on_disk(&w.a, "ops/runbooks/.gitkeep"));
    assert_eq!(
        subjects(&w.remote)[0],
        "waymark: add-folder \"ops/runbooks\""
    );
    let v = ok(c.call("folder_tree", json!({"depth": 3})));
    assert_eq!(entry(&v, "ops/runbooks")["documents"], 0);
    assert!(paths(&v).contains(&"ops".to_string()));
    // A document in it is listed, and the .gitkeep is not counted.
    ok(c.call(
        "add_document",
        json!({"folder": "ops/runbooks", "title": "Restart", "description": "D.", "body": "B."}),
    ));
    let v = ok(c.call("folder_tree", json!({"depth": 3})));
    assert_eq!(entry(&v, "ops/runbooks")["documents"], 1);
    // The folder stays when its last document is deleted.
    let shown = ok(c.call("show", json!({"id": "ops/runbooks/restart"})));
    ok(c.call(
        "delete_document",
        json!({"id": "ops/runbooks/restart", "base": shown["version"]}),
    ));
    let v = ok(c.call("folder_tree", json!({"depth": 3})));
    assert_eq!(entry(&v, "ops/runbooks")["documents"], 0);
}

#[test]
fn add_empty_folder_refuses_an_existing_folder_a_bad_path_and_a_document_id() {
    let w = World::new();
    tree(&w);
    let mut c = client(&w);
    let head = subjects(&w.remote);
    assert_eq!(
        c.call("create_folder", json!({"folder": "ops"})).code(),
        Some("validation_failed")
    );
    assert_eq!(
        c.call("create_folder", json!({"folder": "ops/a"})).code(),
        Some("validation_failed")
    );
    for bad in ["Ops", "../x", "/abs", "a b", ".hidden", ""] {
        assert_eq!(
            c.call("create_folder", json!({"folder": bad})).code(),
            Some("invalid_arguments"),
            "{bad}"
        );
    }
    assert_eq!(subjects(&w.remote), head);
}

#[test]
fn add_empty_folder_follows_the_limits_and_preview() {
    let w = World::new();
    let mut c = client(&w);
    let p = ok(c.call("create_folder", json!({"folder": "ops/x", "preview": true})));
    assert_eq!(p["preview"], true);
    assert!(!on_disk(&w.a, "ops/x"));
    personal(&w, "[limits]\nfolders = [\"work/**\"]\n");
    assert_eq!(
        c.call("create_folder", json!({"folder": "ops/x"})).code(),
        Some("access_denied")
    );
    personal(&w, "[limits]\naccess = \"read-only\"\n");
    assert_eq!(
        c.call("create_folder", json!({"folder": "work/x"})).code(),
        Some("access_denied")
    );
    personal(&w, "[limits]\npaused = true\n");
    assert_eq!(
        c.call("create_folder", json!({"folder": "work/x"})).code(),
        Some("paused")
    );
}

#[test]
fn a_gitkeep_is_not_a_document() {
    let w = World::new();
    let mut c = client(&w);
    ok(c.call("create_folder", json!({"folder": "ops/deep/er"})));
    let l = ok(c.call("lint", json!({})));
    assert_eq!(l["errors"], 0, "{l}");
    assert_eq!(l["warnings"], 0, "{l}");
    let list = ok(c.call("list", json!({})));
    assert_eq!(list["total_matches"], 0);
    let stats = ok(c.call("stats", json!({"metric": "documents"})));
    assert_eq!(stats["total"], 0, "{stats}");
}

#[test]
fn remove_folder_removes_documents_and_gitkeeps_in_one_commit_and_undo_restores() {
    let w = World::new();
    tree(&w);
    let mut c = client(&w);
    ok(c.call("create_folder", json!({"folder": "ops/deploy/empty"})));
    let p = ok(c.call(
        "delete_folder",
        json!({"folder": "ops/deploy", "expect_documents": 3, "preview": true}),
    ));
    assert_eq!(p["documents"].as_array().unwrap().len(), 3);
    assert!(on_disk(&w.a, "ops/deploy/b.md"));
    let before = subjects(&w.remote).len();
    let r = ok(c.call(
        "delete_folder",
        json!({"folder": "ops/deploy", "expect_documents": 3}),
    ));
    assert_eq!(r["pushed"], true);
    assert_eq!(subjects(&w.remote).len(), before + 1);
    assert_eq!(
        subjects(&w.remote)[0],
        "waymark: remove-folder \"ops/deploy\""
    );
    assert!(!on_disk(&w.a, "ops/deploy"));
    let v = ok(c.call("folder_tree", json!({"depth": 3})));
    assert_eq!(paths(&v), vec!["ops", "work"]);
    // One undo brings it all back.
    let act = ok(c.call("activity", json!({})));
    let commit = act["entries"][0]["commit"].clone();
    ok(c.call("undo", json!({"commit": commit})));
    assert!(on_disk(&w.a, "ops/deploy/b.md"));
    assert!(on_disk(&w.a, "ops/deploy/empty/.gitkeep"));
}

#[test]
fn remove_folder_needs_the_right_count_and_removes_an_empty_folder_with_0() {
    let w = World::new();
    tree(&w);
    let mut c = client(&w);
    let head = subjects(&w.remote);
    let r = c.call(
        "delete_folder",
        json!({"folder": "ops/deploy", "expect_documents": 1}),
    );
    assert_eq!(r.code(), Some("validation_failed"), "{}", r.json);
    assert!(r.message().contains("3 documents"), "{}", r.message());
    assert_eq!(subjects(&w.remote), head);
    ok(c.call("create_folder", json!({"folder": "tmp"})));
    ok(c.call(
        "delete_folder",
        json!({"folder": "tmp", "expect_documents": 0}),
    ));
    assert!(!on_disk(&w.a, "tmp"));
    assert_eq!(
        c.call(
            "delete_folder",
            json!({"folder": "tmp", "expect_documents": 0})
        )
        .code(),
        Some("not_found")
    );
}

#[test]
fn remove_folder_refuses_links_from_outside_and_allows_links_inside() {
    let w = World::new();
    commit_doc(&w.a, "ops/in-a", "In A", "See [b](in-b.md).");
    commit_doc(&w.a, "ops/in-b", "In B", "See [a](in-a.md).");
    commit_doc(&w.a, "work/out", "Out", "See [a](../ops/in-a.md).");
    let mut c = client(&w);
    let r = c.call(
        "delete_folder",
        json!({"folder": "ops", "expect_documents": 2}),
    );
    assert_eq!(r.code(), Some("validation_failed"), "{}", r.json);
    let from = r.json["data"]["linked_from"].as_array().unwrap();
    assert!(from
        .iter()
        .any(|f| f["id"] == "work/out" && f["to"] == "ops/in-a"));
    assert!(on_disk(&w.a, "ops/in-a.md"));
    // Remove the outside link, then the folder goes, although its documents link to each other.
    let shown = ok(c.call("show", json!({"id": "work/out"})));
    ok(c.call(
        "update_document",
        json!({"id": "work/out", "base": shown["version"], "body": "No link."}),
    ));
    ok(c.call(
        "delete_folder",
        json!({"folder": "ops", "expect_documents": 2}),
    ));
    assert!(!on_disk(&w.a, "ops"));
}

#[test]
fn remove_folder_follows_the_limits_and_protects_the_root() {
    let w = World::new();
    tree(&w);
    let mut c = client(&w);
    assert_eq!(
        c.call(
            "delete_folder",
            json!({"folder": ".waymark", "expect_documents": 0})
        )
        .code(),
        Some("invalid_arguments")
    );
    assert_eq!(
        c.call(
            "delete_folder",
            json!({"folder": "", "expect_documents": 0})
        )
        .code(),
        Some("invalid_arguments")
    );
    personal(&w, "[limits]\nfolders = [\"work/**\"]\n");
    assert_eq!(
        c.call(
            "delete_folder",
            json!({"folder": "ops", "expect_documents": 4})
        )
        .code(),
        Some("access_denied")
    );
    personal(&w, "[limits]\ntools_denied = [\"delete_folder\"]\n");
    assert!(!c.tool_names().contains(&"delete_folder".to_string()));
    assert!(c.tool_names().contains(&"create_folder".to_string()));
}

#[test]
fn move_carries_the_gitkeep_of_a_folder() {
    let w = World::new();
    let mut c = client(&w);
    ok(c.call("create_folder", json!({"folder": "ops/old"})));
    ok(c.call("move", json!({"from": "ops/old/", "to": "ops/new/"})));
    assert!(on_disk(&w.a, "ops/new/.gitkeep"));
    assert!(!on_disk(&w.a, "ops/old"));
    let v = ok(c.call("folder_tree", json!({"depth": 3})));
    assert!(paths(&v).contains(&"ops/new".to_string()));
    assert!(!paths(&v).contains(&"ops/old".to_string()));
}

#[test]
fn the_folder_tools_have_the_right_hints() {
    let w = World::new();
    let mut c = client(&w);
    let tools = c.list_tools();
    let hint = |name: &str, key: &str| -> Value {
        tools
            .iter()
            .find(|t| t["name"] == name)
            .unwrap_or_else(|| panic!("no tool {name}"))["annotations"][key]
            .clone()
    };
    assert_eq!(hint("folder_tree", "readOnlyHint"), true);
    assert_eq!(hint("create_folder", "readOnlyHint"), false);
    assert_eq!(hint("delete_folder", "destructiveHint"), true);
    assert_eq!(hint("create_folder", "destructiveHint"), false);
}
