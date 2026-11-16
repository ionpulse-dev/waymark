//! End-to-end tests of write approval (see `specs/features/write-approval/test.md`).

mod common;
use common::git::subjects;
use common::memrepo::commit_doc;
use common::{McpClient, Person, Spawn, World};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

type Seen = Arc<Mutex<Vec<(Value, usize)>>>;

fn personal(w: &World, text: &str) {
    let p = w.root.join("home-a/.config/waymark/config.toml");
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, text).unwrap();
}

fn require(w: &World, list: &str) {
    personal(w, &format!("[approval]\nrequired_for = {list}\n"));
}

/// A client that answers every question with `answer`. `seen` gets each request and the number
/// of commits on the remote at that moment.
fn asking(w: &World, answer: Value, delay_ms: u64, seen: &Seen) -> McpClient {
    let remote = w.remote.clone();
    let seen = seen.clone();
    let hook: common::mcp::Hook = Box::new(move |req: &Value| {
        seen.lock()
            .unwrap()
            .push((req.clone(), subjects(&remote).len()));
        std::thread::sleep(Duration::from_millis(delay_ms));
        answer.clone()
    });
    McpClient::for_person(
        w,
        Person::A,
        Spawn {
            elicitation: true,
            hook: Some(hook),
            ..Spawn::default()
        },
    )
}

fn yes() -> Value {
    json!({"action": "accept", "content": {"approve": true}})
}

fn add(c: &mut McpClient, title: &str) -> common::Reply {
    c.call(
        "add_document",
        json!({"folder": "ops", "title": title, "description": format!("About {title}."), "body": "The body text."}),
    )
}

fn exists(w: &World, id: &str) -> bool {
    w.a.join("agent-memory").join(format!("{id}.md")).exists()
}

fn requests(seen: &Seen) -> usize {
    seen.lock()
        .unwrap()
        .iter()
        .filter(|(r, _)| r["method"] == "elicitation/create")
        .count()
}

#[test]
fn the_request_has_the_text_and_the_schema() {
    let w = World::new();
    require(&w, "[\"add\"]");
    let seen: Seen = Default::default();
    let mut c = asking(&w, yes(), 0, &seen);
    let before = subjects(&w.remote).len();
    let r = add(&mut c, "Doc one");
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(r.json["approval"], "approved");
    let (req, at_request) = seen.lock().unwrap()[0].clone();
    assert_eq!(at_request, before, "asked before the commit");
    let p = &req["params"];
    assert_eq!(p["mode"], "form");
    let m = p["message"].as_str().unwrap();
    assert!(
        m.contains("Tool: add") && m.contains("Author: Anna Example"),
        "{m}"
    );
    assert!(m.contains("--- Body ---\nThe body text."), "{m}");
    assert_eq!(p["requestedSchema"]["required"], json!(["approve"]));
    assert_eq!(
        p["requestedSchema"]["properties"]["approve"]["type"],
        "boolean"
    );
    assert!(exists(&w, "ops/doc-one"));
}

#[test]
fn delete_asks_before_commit() {
    let w = World::new();
    commit_doc(&w.a, "ops/old", "Old doc", "Old body.");
    require(&w, "[\"delete_document\"]");
    let seen: Seen = Default::default();
    let mut c = asking(&w, yes(), 0, &seen);
    let version = common::git::git(
        &w.a,
        &["log", "-1", "--format=%h", "--", "agent-memory/ops/old.md"],
    );
    let r = c.call(
        "delete_document",
        json!({"id": "ops/old", "base": version.trim()}),
    );
    assert!(!r.is_error, "{}", r.json);
    let m = seen.lock().unwrap()[0].0["params"]["message"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(m.contains("Old doc") && m.contains("Old body."), "{m}");
    assert!(!exists(&w, "ops/old"));
}

fn denied_with(answer: Value, message: &str) {
    let w = World::new();
    require(&w, "[\"add\"]");
    let seen: Seen = Default::default();
    let mut c = asking(&w, answer, 0, &seen);
    let (a, r) = (subjects(&w.a), subjects(&w.remote));
    let reply = add(&mut c, "Doc one");
    assert_eq!(reply.code(), Some("approval_denied"), "{}", reply.json);
    assert_eq!(reply.message(), message);
    assert_eq!(reply.json["next"], "Nothing was written.");
    assert_eq!((subjects(&w.a), subjects(&w.remote)), (a, r));
    assert!(!exists(&w, "ops/doc-one"));
    let status = common::git::git(&w.a, &["status", "--porcelain"]);
    assert!(status.trim().is_empty(), "{status}");
}

#[test]
fn decline_denies() {
    denied_with(json!({"action": "decline"}), "The person said no.");
}

#[test]
fn cancel_denies() {
    denied_with(
        json!({"action": "cancel"}),
        "The person closed the question.",
    );
}

#[test]
fn accept_without_approve_denies() {
    for content in [
        json!({"approve": false}),
        json!({}),
        json!({"approve": "yes"}),
    ] {
        denied_with(
            json!({"action": "accept", "content": content}),
            "The person did not approve.",
        );
    }
}

fn unavailable_with(caps: Option<Value>, elicitation: bool, hook: Option<Value>) {
    let w = World::new();
    require(&w, "[\"add\"]");
    let seen: Seen = Default::default();
    let s2 = seen.clone();
    let reply = hook.clone();
    let hook_fn: common::mcp::Hook = Box::new(move |req: &Value| {
        s2.lock().unwrap().push((req.clone(), 0));
        reply.clone().unwrap_or(json!({}))
    });
    let mut c = McpClient::for_person(
        &w,
        Person::A,
        Spawn {
            elicitation,
            elicitation_caps: caps,
            hook: Some(hook_fn),
            ..Spawn::default()
        },
    );
    let before = subjects(&w.remote);
    let r = add(&mut c, "Doc one");
    assert_eq!(r.code(), Some("approval_unavailable"), "{}", r.json);
    assert_eq!(subjects(&w.remote), before);
    if hook.is_none() {
        assert_eq!(requests(&seen), 0, "no request is sent");
    }
}

#[test]
fn no_capability_is_unavailable() {
    unavailable_with(None, false, None);
}

#[test]
fn url_only_capability_is_unavailable() {
    unavailable_with(Some(json!({"url": {}})), false, None);
}

#[test]
fn a_client_error_is_unavailable() {
    unavailable_with(None, true, Some(json!({"__error": true})));
}

#[test]
fn form_capability_gets_the_request() {
    let w = World::new();
    require(&w, "[\"add\"]");
    let seen: Seen = Default::default();
    let s2 = seen.clone();
    let hook: common::mcp::Hook = Box::new(move |req: &Value| {
        s2.lock().unwrap().push((req.clone(), 0));
        yes()
    });
    let mut c = McpClient::for_person(
        &w,
        Person::A,
        Spawn {
            elicitation_caps: Some(json!({"form": {}})),
            hook: Some(hook),
            ..Spawn::default()
        },
    );
    assert!(!add(&mut c, "Doc one").is_error);
    assert_eq!(requests(&seen), 1);
}

#[test]
fn timeout_denies_and_frees_the_lock() {
    let w = World::new();
    require(&w, "[\"add\"]");
    let seen: Seen = Default::default();
    let n = Arc::new(Mutex::new(0));
    let s2 = seen.clone();
    let hook: common::mcp::Hook = Box::new(move |req: &Value| {
        s2.lock().unwrap().push((req.clone(), 0));
        let mut k = n.lock().unwrap();
        *k += 1;
        if *k == 1 {
            std::thread::sleep(Duration::from_secs(4));
        }
        yes()
    });
    let mut c = McpClient::for_person(
        &w,
        Person::A,
        Spawn {
            elicitation: true,
            hook: Some(hook),
            env: vec![("WAYMARK_APPROVAL_TIMEOUT_SECONDS".into(), "2".into())],
            ..Spawn::default()
        },
    );
    let r = add(&mut c, "Doc one");
    assert_eq!(r.code(), Some("approval_denied"), "{}", r.json);
    assert_eq!(r.message(), "No answer in 2 minutes.");
    assert!(!c.notification_params("notifications/cancelled").is_empty());
    let r = add(&mut c, "Doc two");
    assert!(!r.is_error, "{}", r.json);
}

#[test]
fn progress_messages_come_while_waiting() {
    let w = World::new();
    require(&w, "[\"add\"]");
    let seen: Seen = Default::default();
    let mut c = {
        let remote = w.remote.clone();
        let s2 = seen.clone();
        let hook: common::mcp::Hook = Box::new(move |req: &Value| {
            s2.lock()
                .unwrap()
                .push((req.clone(), subjects(&remote).len()));
            std::thread::sleep(Duration::from_secs(3));
            yes()
        });
        McpClient::for_person(
            &w,
            Person::A,
            Spawn {
                elicitation: true,
                hook: Some(hook),
                env: vec![("WAYMARK_APPROVAL_TIMEOUT_SECONDS".into(), "8".into())],
                ..Spawn::default()
            },
        )
    };
    let r = c.call_with_progress(
        "add_document",
        json!({"folder": "ops", "title": "Doc one", "description": "D.", "body": "B."}),
    );
    assert!(!r.is_error, "{}", r.json);
    let p = c.notification_params("notifications/progress");
    assert!(!p.is_empty());
    assert_eq!(p[0]["message"], "Waiting for approval");
}

#[test]
fn unlisted_tool_and_read_tools_never_ask() {
    let w = World::new();
    require(&w, "[\"delete\"]");
    let seen: Seen = Default::default();
    let mut c = asking(&w, yes(), 0, &seen);
    assert!(!add(&mut c, "Doc one").is_error);
    assert_eq!(requests(&seen), 0);
    require(&w, "[\"write\"]");
    assert!(!c.call("status", json!({})).is_error);
    assert!(!c.call("search", json!({})).is_error);
    assert!(!c.call("search", json!({"query": "doc"})).is_error);
    assert_eq!(requests(&seen), 0);
}

#[test]
fn write_covers_all_write_tools_and_each_write_asks_again() {
    let w = World::new();
    require(&w, "[\"write\"]");
    let seen: Seen = Default::default();
    let mut c = asking(&w, yes(), 0, &seen);
    assert!(!add(&mut c, "Doc one").is_error);
    assert!(!add(&mut c, "Doc two").is_error);
    assert_eq!(requests(&seen), 2);
}

#[test]
fn preview_never_asks() {
    let w = World::new();
    require(&w, "[\"add\"]");
    let seen: Seen = Default::default();
    let mut c = asking(&w, yes(), 0, &seen);
    let r = c.call(
        "add_document",
        json!({"folder": "ops", "title": "T", "description": "D.", "body": "B.", "preview": true}),
    );
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(r.json["approval"], "required");
    assert_eq!(requests(&seen), 0);
}

#[test]
fn pause_during_approval() {
    for (text, code) in [
        ("[limits]\npaused = true\n", "paused"),
        ("[limits]\naccess = \"read-only\"\n", "access_denied"),
    ] {
        let w = World::new();
        require(&w, "[\"add\"]");
        let path: PathBuf = w.root.join("home-a/.config/waymark/config.toml");
        let remote = w.remote.clone();
        let hook: common::mcp::Hook = Box::new(move |_| {
            let _ = &remote;
            std::fs::write(
                &path,
                format!("[approval]\nrequired_for = [\"add_document\"]\n{text}"),
            )
            .unwrap();
            yes()
        });
        let mut c = McpClient::for_person(
            &w,
            Person::A,
            Spawn {
                elicitation: true,
                hook: Some(hook),
                ..Spawn::default()
            },
        );
        let before = subjects(&w.remote);
        let r = add(&mut c, "Doc one");
        assert_eq!(r.code(), Some(code), "{}", r.json);
        assert_eq!(subjects(&w.remote), before);
        assert!(!exists(&w, "ops/doc-one"));
    }
}

#[test]
fn the_shared_and_personal_rules_are_a_union() {
    let w = World::new();
    let shared = w.a.join("agent-memory/.waymark/settings.toml");
    let old = std::fs::read_to_string(&shared).unwrap();
    std::fs::write(
        &shared,
        old.replacen("[approval]\n", "[approval]\nrequired_for = [\"add\"]\n", 1),
    )
    .unwrap();
    common::git::git(&w.a, &["add", "-A"]);
    common::git::git(&w.a, &["commit", "-q", "-m", "test: settings"]);
    common::git::git(&w.a, &["push", "-q"]);
    require(&w, "[\"delete\"]");
    let seen: Seen = Default::default();
    let mut c = asking(&w, yes(), 0, &seen);
    let r = add(&mut c, "Doc one");
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(
        r.json["settings"]["approval"]["required_for"],
        json!(["add_document", "delete_document"])
    );
    assert_eq!(requests(&seen), 1);
}

#[test]
fn console_is_unavailable_in_local_mode() {
    let w = World::new();
    personal(
        &w,
        "[approval]\nrequired_for = [\"add\"]\nvia = \"console\"\n",
    );
    let seen: Seen = Default::default();
    let mut c = asking(&w, yes(), 0, &seen);
    let r = add(&mut c, "Doc one");
    assert_eq!(r.code(), Some("approval_unavailable"), "{}", r.json);
    assert!(r.message().contains("console"));
    assert_eq!(requests(&seen), 0);
}

#[test]
fn sync_asks_before_it_commits_manual_edits() {
    let w = World::new();
    require(&w, "[\"sync\"]");
    let seen: Seen = Default::default();
    let mut c = asking(&w, yes(), 0, &seen);
    common::memrepo::write_doc(
        &w.a,
        "ops/manual",
        &common::memrepo::doc("Manual", "Hand made."),
    );
    let r = c.call("sync", json!({}));
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(requests(&seen), 1);
    let m = seen.lock().unwrap()[0].0["params"]["message"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(m.contains("Tool: sync") && m.contains("ops/manual"), "{m}");
}

#[test]
fn verify_asks_approval() {
    let w = World::new();
    let shared = w.a.join("agent-memory/.waymark/settings.toml");
    let old = std::fs::read_to_string(&shared).unwrap();
    std::fs::write(
        &shared,
        old.replacen("verification = false", "verification = true", 1)
            .replacen(
                "[approval]\n",
                "[approval]\nrequired_for = [\"verify\"]\n",
                1,
            ),
    )
    .unwrap();
    common::git::git(&w.a, &["add", "-A"]);
    common::git::git(&w.a, &["commit", "-q", "-m", "test: settings"]);
    common::git::git(&w.a, &["push", "-q"]);
    let seen: Seen = Default::default();
    let mut c = asking(&w, yes(), 0, &seen);
    let added = add(&mut c, "Doc one");
    assert_eq!(requests(&seen), 0);
    let v = c.call(
        "verify_document",
        json!({"id": "ops/doc-one", "base": added.json["version"]}),
    );
    assert!(!v.is_error, "{}", v.json);
    assert_eq!(requests(&seen), 1);
    assert_eq!(v.json["approval"], "approved");
}

#[test]
fn sync_asks_when_add_document_needs_approval() {
    // See `specs/features/document-names`: `sync` applies the approval rule of the matching tool.
    let w = World::new();
    require(&w, "[\"add_document\"]");
    let seen: Seen = Default::default();
    let mut c = asking(&w, yes(), 0, &seen);
    common::memrepo::write_doc(
        &w.a,
        "ops/manual",
        &common::memrepo::doc("Manual", "Hand made."),
    );
    let r = c.call("sync", json!({}));
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(requests(&seen), 1);
    let m = seen.lock().unwrap()[0].0["params"]["message"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(
        m.contains("Tool: add_document") && m.contains("ops/manual"),
        "{m}"
    );
}

#[test]
fn a_write_holds_back_an_edit_that_needs_approval() {
    let w = World::new();
    commit_doc(&w.a, "ops/deploy", "How we deploy", "Old.");
    require(&w, "[\"add_document\"]");
    common::memrepo::write_doc(
        &w.a,
        "ops/manual",
        &common::memrepo::doc("Manual", "Hand made."),
    );
    let mut c = McpClient::for_person(&w, Person::A, Spawn::default());
    let base = c.call("show", json!({"id": "ops/deploy"})).json["version"]
        .as_str()
        .unwrap()
        .to_string();
    let r = c.call(
        "update_document",
        json!({"id": "ops/deploy", "base": base, "body": "New."}),
    );
    assert!(!r.is_error, "{}", r.json);
    assert!(
        r.json["notices"]
            .to_string()
            .contains("1 manual edit is not published. Run `sync`."),
        "{}",
        r.json
    );
    assert!(!subjects(&w.remote)
        .iter()
        .any(|s| s.contains("\"Manual\"")));
    assert!(exists(&w, "ops/manual"));
}
