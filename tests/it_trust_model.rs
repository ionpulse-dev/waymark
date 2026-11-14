//! End-to-end tests of the trust model through the real binary (see `specs/foundation/trust-model/test.md`).
//!
//! The effective limits show in the `settings` field of every result, so `status` is enough to see them.
//! Tests that need a write tool (the commit author, a blocked write) are in `write-documents`.

mod common;
use common::{McpClient, Person, Spawn, World};
use serde_json::{json, Value};

fn write(path: &std::path::Path, text: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

fn personal(w: &World, text: &str) {
    write(&w.root.join("home-a/.config/waymark/config.toml"), text);
}

fn system(w: &World, text: &str) {
    write(&w.root.join("home-a/system-config.toml"), text);
}

fn shared(w: &World, limits: &str) {
    let path = w.a.join("agent-memory/.waymark/settings.toml");
    let text = std::fs::read_to_string(&path).unwrap();
    write(&path, &format!("{text}\n{limits}\n"));
}

fn settings(w: &World, args: &[&str]) -> Value {
    let mut c = McpClient::for_person(w, Person::A, Spawn::args(args));
    c.call("status", json!({})).json["settings"].clone()
}

#[test]
fn defaults_apply() {
    let w = World::new();
    let s = settings(&w, &[]);
    assert_eq!(s["access"], "read-write");
    assert_eq!(s["paused"], false);
    assert_eq!(s["max_results"], 5);
    assert_eq!(s["max_tokens"], 800);
    assert_eq!(s["max_documents_opened"], 20);
}

#[test]
fn shared_read_only_wins() {
    let w = World::new();
    // The template has a [limits] section already: add the key below it.
    let path = w.a.join("agent-memory/.waymark/settings.toml");
    let text = std::fs::read_to_string(&path).unwrap();
    write(
        &path,
        &text.replace("[limits]", "[limits]\naccess = \"read-only\""),
    );
    personal(&w, "[limits]\naccess = \"read-write\"\n");
    assert_eq!(
        settings(&w, &["--access", "read-write"])["access"],
        "read-only"
    );
}

#[test]
fn numeric_caps_take_smallest() {
    let w = World::new();
    let path = w.a.join("agent-memory/.waymark/settings.toml");
    let text = std::fs::read_to_string(&path).unwrap();
    write(
        &path,
        &text.replace("[limits]", "[limits]\nmax_results = 5"),
    );
    personal(&w, "[limits]\nmax_results = 3\n");
    assert_eq!(settings(&w, &[])["max_results"], 3);
    assert_eq!(settings(&w, &["--max-results", "2"])["max_results"], 2);
}

#[test]
fn system_file_wins() {
    let w = World::new();
    system(&w, "[limits]\naccess = \"read-only\"\n");
    personal(&w, "[limits]\naccess = \"read-write\"\n");
    assert_eq!(
        settings(&w, &["--access", "read-write"])["access"],
        "read-only"
    );
}

#[test]
fn paused_shows_in_settings_and_status_still_works() {
    let w = World::new();
    personal(&w, "[limits]\npaused = true\n");
    let mut c = McpClient::for_person(&w, Person::A, Spawn::default());
    let r = c.call("status", json!({}));
    assert!(!r.is_error);
    assert_eq!(r.json["settings"]["paused"], true);
}

#[test]
fn no_control_tool_listed() {
    let w = World::new();
    let mut c = McpClient::for_person(&w, Person::A, Spawn::default());
    for n in c.tool_names() {
        for bad in [
            "limit",
            "access",
            "pause",
            "resume",
            "approval",
            "login",
            "credential",
        ] {
            assert!(!n.contains(bad), "tool {n} looks like a control tool");
        }
    }
}

#[test]
fn the_effective_settings_change_when_the_personal_file_changes_between_two_calls() {
    let w = World::new();
    let mut c = McpClient::for_person(&w, Person::A, Spawn::default());
    assert_eq!(
        c.call("status", json!({})).json["settings"]["access"],
        "read-write"
    );
    personal(&w, "[limits]\naccess = \"read-only\"\n");
    assert_eq!(
        c.call("status", json!({})).json["settings"]["access"],
        "read-only"
    );
    personal(&w, "[limits]\naccess = \"read-write\"\n");
    assert_eq!(
        c.call("status", json!({})).json["settings"]["access"],
        "read-write"
    );
}

#[test]
fn every_result_shows_the_effective_limits() {
    let w = World::new();
    personal(&w, "[limits]\nmax_tokens = 300\n");
    let mut c = McpClient::for_person(&w, Person::A, Spawn::default());
    let ok = c.call("status", json!({}));
    let err = c.call("status", json!({"bad": 1}));
    assert_eq!(ok.json["settings"]["max_tokens"], 300);
    assert_eq!(err.json["settings"]["max_tokens"], 300);
}

#[test]
fn a_denied_tool_list_and_folders_show_in_settings() {
    let w = World::new();
    shared(&w, "");
    let s = settings(&w, &["--no-delete", "--folders", "engineering/**"]);
    assert_eq!(s["tools_denied"], json!(["delete_document"]));
    assert_eq!(s["folders"], json!(["engineering/**"]));
}
