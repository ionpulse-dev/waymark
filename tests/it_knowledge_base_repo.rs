//! End-to-end tests of the knowledge base layout (see `specs/foundation/knowledge-base-repo/test.md`).

mod common;
use common::git::git;
use common::{McpClient, Person, Spawn, World};
use serde_json::json;

fn check<'a>(r: &'a serde_json::Value, name: &str) -> &'a serde_json::Value {
    r["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == name)
        .unwrap()
}

#[test]
fn gitignore_hides_personal_obsidian_files() {
    let w = World::new();
    let obsidian = w.a.join(".obsidian");
    std::fs::write(obsidian.join("workspace.json"), "{}").unwrap();
    std::fs::write(obsidian.join("graph.json"), "{}").unwrap();
    assert_eq!(git(&w.a, &["status", "--porcelain"]), "");
    // The shared file stays in Git.
    assert!(git(&w.a, &["ls-files"]).contains(".obsidian/app.json"));
}

#[test]
fn init_writes_obsidian_defaults() {
    let w = World::new();
    let app: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(w.a.join(".obsidian/app.json")).unwrap())
            .unwrap();
    assert_eq!(app["useMarkdownLinks"], true);
    assert_eq!(app["newLinkFormat"], "relative");
}

#[test]
fn missing_id_is_setup_problem() {
    let w = World::new();
    let path = w.a.join("knowledge/.mkbase/waymark.toml");
    let text = std::fs::read_to_string(&path).unwrap();
    let without: String = text
        .lines()
        .filter(|l| !l.starts_with("id ="))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&path, without).unwrap();
    let mut c = McpClient::for_person(&w, Person::A, Spawn::default());
    let r = c.call("status", json!({}));
    assert_eq!(r.json["healthy"], false);
    let kb = check(&r.json, "knowledge_base");
    assert_eq!(kb["ok"], false);
    assert!(kb["detail"].as_str().unwrap().contains("id"));
}

#[test]
fn stricter_value_applies() {
    let w = World::new();
    let path = w.a.join("knowledge/.mkbase/waymark.toml");
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::write(
        &path,
        text.replace("[limits]", "[limits]\nmax_tokens = 400"),
    )
    .unwrap();
    let mut c = McpClient::for_person(&w, Person::A, Spawn::args(&["--max-tokens", "900"]));
    assert_eq!(
        c.call("status", json!({})).json["settings"]["max_tokens"],
        400
    );
}

#[test]
fn unknown_keys_give_a_notice_and_no_error() {
    let w = World::new();
    let path = w.a.join("knowledge/.mkbase/waymark.toml");
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::write(&path, format!("{text}\n[future]\nthing = 1\n")).unwrap();
    let mut c = McpClient::for_person(&w, Person::A, Spawn::default());
    let r = c.call("status", json!({}));
    assert!(!r.is_error);
    assert_eq!(r.json["healthy"], true);
    assert!(r.json["notices"]
        .as_array()
        .map(|n| !n.is_empty())
        .unwrap_or(false));
}

#[test]
fn the_layout_has_the_root_in_knowledge_and_no_owners_field() {
    let w = World::new();
    let kb = waymark::kb::KnowledgeBase::open(&w.a).unwrap();
    assert!(kb.root.ends_with("knowledge"));
    assert!(kb.documents().is_empty());
    let toml = std::fs::read_to_string(w.a.join("knowledge/.mkbase/waymark.toml")).unwrap();
    assert!(!toml.contains("owners"));
}

#[test]
fn a_second_person_gets_the_same_knowledge_base_id() {
    let w = World::new();
    let mut a = McpClient::for_person(&w, Person::A, Spawn::default());
    let mut b = McpClient::for_person(&w, Person::B, Spawn::default());
    let ra = a.call("status", json!({}));
    let rb = b.call("status", json!({}));
    assert_eq!(
        ra.json["knowledge_base"]["id"],
        rb.json["knowledge_base"]["id"]
    );
}
