//! End-to-end tests of notices (see `specs/features/notifications-and-review/test.md`).
//! Person A runs the server that asks. Person B writes. Every client is a new session.

mod common;
use common::git::git;
use common::{McpClient, Person, Spawn, World};
use serde_json::{json, Value};
use std::time::Duration;

fn client(w: &World, p: Person) -> McpClient {
    McpClient::for_person(
        w,
        p,
        Spawn {
            env: vec![("WAYMARK_FETCH_MAX_AGE".into(), "0".into())],
            ..Spawn::default()
        },
    )
}

fn pause() {
    std::thread::sleep(Duration::from_millis(1100));
}

fn add(c: &mut McpClient, title: &str) -> (String, String) {
    let r = c.call(
        "add_document",
        json!({"folder": "ops", "title": title, "description": "About it.", "body": "Text."}),
    );
    assert!(!r.is_error, "{}", r.json);
    (
        r.json["id"].as_str().unwrap().to_string(),
        r.json["version"].as_str().unwrap().to_string(),
    )
}

fn version(c: &mut McpClient, id: &str) -> String {
    c.call("show", json!({"id": id})).json["version"]
        .as_str()
        .unwrap()
        .to_string()
}

fn notices(v: &Value) -> Vec<String> {
    v["notices"]
        .as_array()
        .map(|a| a.iter().map(|n| n.as_str().unwrap().to_string()).collect())
        .unwrap_or_default()
}

fn search(c: &mut McpClient) -> Value {
    let r = c.call("search", json!({"query": "about"}));
    assert!(!r.is_error, "{}", r.json);
    r.json
}

/// A wrote a document and stored the time. B disputes it. Returns the ID.
fn disputed_world() -> (World, String) {
    let w = World::new();
    let mut a = client(&w, Person::A);
    let (id, _) = add(&mut a, "Retry policy");
    a.call("updates", json!({}));
    drop(a);
    pause();
    let mut b = client(&w, Person::B);
    let base = version(&mut b, &id);
    let d = b.call(
        "dispute_document",
        json!({"id": id, "base": base, "reason": "Wrong."}),
    );
    assert!(!d.is_error, "{}", d.json);
    (w, id)
}

fn has(v: &Value, word: &str, id: &str) -> bool {
    notices(v)
        .iter()
        .any(|n| n.contains(word) && n.contains(&format!("`{id}`")))
}

#[test]
fn t1_t4_a_dispute_gives_one_notice_per_session() {
    let (w, id) = disputed_world();
    let mut a = client(&w, Person::A);
    let r = search(&mut a);
    assert!(has(&r, "disputed", &id), "{r}");
    assert!(notices(&r)[0].contains("Run `updates`"));
    // The same session: no repeat.
    assert!(notices(&search(&mut a)).is_empty());
    let l = a.call("search", json!({}));
    assert!(notices(&l.json).is_empty());
    // T11: status lists it again.
    let s = a.call("status", json!({}));
    assert!(has(&s.json, "disputed", &id), "{}", s.json);
}

#[test]
fn t2_a_change_by_someone_else_gives_a_notice() {
    let w = World::new();
    let mut a = client(&w, Person::A);
    let (id, _) = add(&mut a, "Deploy steps");
    a.call("updates", json!({}));
    drop(a);
    pause();
    let mut b = client(&w, Person::B);
    let base = version(&mut b, &id);
    let u = b.call(
        "update_document",
        json!({"id": id, "base": base, "body": "New text."}),
    );
    assert!(!u.is_error, "{}", u.json);
    let mut a = client(&w, Person::A);
    let r = search(&mut a);
    assert!(has(&r, "changed", &id), "{r}");
}

#[test]
fn t3_a_failed_fetch_says_the_clone_may_be_behind() {
    let w = World::new();
    git(
        &w.a,
        &["remote", "set-url", "origin", "file:///does/not/exist.git"],
    );
    let mut a = client(&w, Person::A);
    let r = search(&mut a);
    assert_eq!(
        notices(&r),
        vec!["The last update from the remote failed. The clone may be behind."]
    );
    let s = a.call("status", json!({}));
    assert!(notices(&s.json)
        .iter()
        .any(|n| n.contains("clone may be behind")));
}

#[test]
fn t5_no_problems_means_no_notices_key() {
    let w = World::new();
    let mut a = client(&w, Person::A);
    let r = search(&mut a);
    assert!(r.get("notices").is_none(), "{r}");
}

#[test]
fn t6_updates_without_since_clears_the_notice() {
    let (w, id) = disputed_world();
    let mut a = client(&w, Person::A);
    let u = a.call("updates", json!({}));
    assert!(!notices(&u.json).iter().any(|n| n.contains("disputed")));
    assert!(u.json["changes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c["id"] == id));
    assert!(notices(&search(&mut a)).is_empty());
    drop(a);
    let mut a = client(&w, Person::A);
    assert!(notices(&search(&mut a)).is_empty());
}

#[test]
fn t7_a_notice_stops_after_three_sessions() {
    let (w, id) = disputed_world();
    for session in 1..=3 {
        let mut a = client(&w, Person::A);
        assert!(has(&search(&mut a), "disputed", &id), "session {session}");
    }
    let mut a = client(&w, Person::A);
    assert!(notices(&search(&mut a)).is_empty());
}

#[test]
fn t8_a_dispute_that_was_cleared_gives_no_notice() {
    let w = World::new();
    let mut a = client(&w, Person::A);
    let (id, _) = add(&mut a, "Cleared one");
    a.call("updates", json!({}));
    drop(a);
    pause();
    let mut b = client(&w, Person::B);
    let base = version(&mut b, &id);
    assert!(
        !b.call(
            "dispute_document",
            json!({"id": id, "base": base, "reason": "Wrong."})
        )
        .is_error
    );
    let base = version(&mut b, &id);
    let c = b.call(
        "dispute_document",
        json!({"id": id, "base": base, "clear": true}),
    );
    assert!(!c.is_error, "{}", c.json);
    let mut a = client(&w, Person::A);
    assert!(notices(&search(&mut a)).is_empty());
}

#[test]
fn t9_paused_is_said_once() {
    let w = World::new();
    let p = w.root.join("home-a/.config/waymark/config.toml");
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, "[limits]\npaused = true\n").unwrap();
    let mut a = client(&w, Person::A);
    let first = search(&mut a);
    assert_eq!(notices(&first), vec!["Writes are paused."]);
    assert!(notices(&search(&mut a)).is_empty());
}

#[test]
fn t10_my_own_change_gives_no_notice() {
    let w = World::new();
    let mut a = client(&w, Person::A);
    let (id, _) = add(&mut a, "Mine only");
    let base = version(&mut a, &id);
    a.call(
        "update_document",
        json!({"id": id, "base": base, "body": "Again."}),
    );
    drop(a);
    let mut a = client(&w, Person::A);
    assert!(notices(&search(&mut a)).is_empty());
}

#[test]
fn t12_more_than_three_documents_say_and_n_more() {
    let w = World::new();
    let mut a = client(&w, Person::A);
    let ids: Vec<String> = (1..=5)
        .map(|i| add(&mut a, &format!("Doc number {i}")).0)
        .collect();
    a.call("updates", json!({}));
    drop(a);
    pause();
    let mut b = client(&w, Person::B);
    for id in &ids {
        let base = version(&mut b, id);
        assert!(
            !b.call(
                "dispute_document",
                json!({"id": id, "base": base, "reason": "Wrong."})
            )
            .is_error
        );
    }
    let mut a = client(&w, Person::A);
    let r = search(&mut a);
    let n = notices(&r);
    assert_eq!(n.len(), 1, "{r}");
    assert!(n[0].contains("disputed 5 of your documents"));
    assert!(n[0].contains("and 2 more"), "{}", n[0]);
}

#[test]
fn t13_an_error_result_carries_the_notice() {
    let (w, id) = disputed_world();
    let mut a = client(&w, Person::A);
    let r = a.call("show", json!({"id": "ops/does-not-exist"}));
    assert!(r.is_error);
    assert!(has(&r.json, "disputed", &id), "{}", r.json);
}

#[test]
fn t14_a_broken_state_file_does_not_fail_a_call() {
    let (w, _) = disputed_world();
    let dir = w.root.join("home-a/state");
    for e in std::fs::read_dir(&dir).unwrap() {
        std::fs::write(e.unwrap().path(), "not json").unwrap();
    }
    let mut a = client(&w, Person::A);
    let r = a.call("search", json!({"query": "about"}));
    assert!(!r.is_error, "{}", r.json);
}
