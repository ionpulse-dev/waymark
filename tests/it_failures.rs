//! Failure behavior (task G6 of the plan). Each failure gives a clear message and a `next` sentence.

mod common;
use common::{McpClient, Person, Spawn, World};
use serde_json::json;

fn has_next(r: &common::Reply) -> bool {
    r.json["next"].as_str().is_some_and(|n| !n.is_empty())
}

#[test]
fn a_missing_root_is_a_setup_problem_with_a_next_step() {
    let w = World::new();
    let mut c = McpClient::for_person(&w, Person::A, Spawn::args(&[]));
    std::fs::remove_dir_all(w.a.join("agent-memory/.waymark")).unwrap();
    let r = c.call("search", json!({}));
    assert!(r.is_error, "{}", r.json);
    assert!(has_next(&r), "{}", r.json);
}

#[test]
fn a_corrupt_document_does_not_break_the_others() {
    let w = World::new();
    let mut c = McpClient::for_person(&w, Person::A, Spawn::args(&[]));
    std::fs::write(w.a.join("agent-memory/broken.md"), "---\n: : [\n---\nx").unwrap();
    let r = c.call("search", json!({}));
    assert!(!r.is_error, "{}", r.json);
    let s = c.call("search", json!({"query": "anything"}));
    assert!(!s.is_error, "{}", s.json);
}

#[test]
fn a_rejected_push_says_what_to_do() {
    let w = World::new();
    let hook = w.remote.join("hooks/pre-receive");
    std::fs::write(&hook, "#!/bin/sh\necho no >&2\nexit 1\n").unwrap();
    std::process::Command::new("chmod")
        .args(["+x"])
        .arg(&hook)
        .status()
        .unwrap();
    let mut c = McpClient::for_person(&w, Person::A, Spawn::args(&[]));
    let r = c.call(
        "add_document",
        json!({"folder": "ops", "title": "Pushed", "description": "D.", "body": "B."}),
    );
    let text = r.json.to_string();
    assert!(text.contains("push") || text.contains("Push"), "{text}");
    assert!(
        has_next(&r) || text.contains("next") || text.contains("notices"),
        "{text}"
    );
}

#[test]
fn no_git_on_the_path_gives_a_clear_message() {
    let w = World::new();
    let mut s = Spawn::args(&[]);
    s.env = vec![("PATH".into(), "/nonexistent".into())];
    let mut c = McpClient::for_person(&w, Person::A, s);
    let r = c.call("status", json!({}));
    let text = r.json.to_string();
    assert!(text.to_lowercase().contains("git"), "{text}");
    assert!(text.contains("next"), "{text}");
}
