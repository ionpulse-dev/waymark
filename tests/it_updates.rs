//! End-to-end tests of `updates` (see `specs/features/updates/test.md`).
//! Person A runs the server that asks. Person B writes.

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

/// Add a document. Returns (id, version).
fn add(c: &mut McpClient, folder: &str, title: &str) -> (String, String) {
    let r = c.call(
        "add",
        json!({"folder": folder, "title": title, "description": "About it.", "body": "Text."}),
    );
    assert!(!r.is_error, "{}", r.json);
    (
        r.json["id"].as_str().unwrap().to_string(),
        r.json["version"].as_str().unwrap().to_string(),
    )
}

fn pause() {
    // Commit times have a resolution of one second.
    std::thread::sleep(Duration::from_millis(1100));
}

fn updates(c: &mut McpClient, args: Value) -> Value {
    let r = c.call("updates", args);
    assert!(!r.is_error, "{}", r.json);
    r.json
}

fn entry<'a>(v: &'a Value, id: &str) -> &'a Value {
    v["changes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == id)
        .unwrap_or_else(|| panic!("{id} not in {v}"))
}

fn ids(v: &Value) -> Vec<String> {
    v["changes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["id"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn t1_a_dispute_of_my_document_is_listed_with_its_reason() {
    let w = World::new();
    let mut a = client(&w, Person::A);
    let (id, _) = add(&mut a, "backend", "Retry policy");
    let first = updates(&mut a, json!({}));
    assert_eq!(first["marked"], true);
    pause();
    let mut b = client(&w, Person::B);
    let v = b.call("show", json!({"id": id}));
    let base = v.json["version"].as_str().unwrap().to_string();
    let d = b.call(
        "dispute",
        json!({"id": id, "base": base, "reason": "The retry limit changed in v3."}),
    );
    assert!(!d.is_error, "{}", d.json);
    let u = updates(&mut a, json!({}));
    let e = entry(&u, &id);
    assert_eq!(e["kind"], "disputed");
    assert_eq!(e["mine"], true);
    assert_eq!(e["reason"], "The retry limit changed in v3.");
    assert!(e["by"].as_str().unwrap().contains("ben@example.org"));
    assert!(u["message"]
        .as_str()
        .unwrap()
        .contains("1 of your document"));
    assert!(u["message"].as_str().unwrap().contains("did not check"));
}

#[test]
fn t2_t3_the_second_call_covers_only_the_time_since_the_first() {
    let w = World::new();
    let mut a = client(&w, Person::A);
    let mut b = client(&w, Person::B);
    let (one, _) = add(&mut b, "ops", "First doc");
    pause();
    let u = updates(&mut a, json!({}));
    assert_eq!(ids(&u), vec![one.clone()]);
    pause();
    let (two, _) = add(&mut b, "ops", "Second doc");
    pause();
    // A call with `since` only reads.
    let past = updates(&mut a, json!({"since": "2020-01-01"}));
    assert_eq!(past["marked"], false);
    assert_eq!(past["total"], 2);
    let u = updates(&mut a, json!({}));
    assert_eq!(ids(&u), vec![two]);
    assert_eq!(u["marked"], true);
}

#[test]
fn t4_t5_folders_and_the_folders_limit() {
    let w = World::new();
    let mut b = client(&w, Person::B);
    add(&mut b, "ops", "Ops doc");
    add(&mut b, "backend", "Backend doc");
    let mut a = client(&w, Person::A);
    let u = updates(&mut a, json!({"since": "2020-01-01", "folders": ["ops"]}));
    assert_eq!(ids(&u), vec!["ops/ops-doc"]);
    drop(a);
    let p = w.root.join("home-a/.config/waymark/config.toml");
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, "[limits]\nfolders = [\"ops/**\"]\n").unwrap();
    let mut a = client(&w, Person::A);
    let u = updates(&mut a, json!({"since": "2020-01-01"}));
    assert_eq!(ids(&u), vec!["ops/ops-doc"]);
}

#[test]
fn t6_my_own_changes_are_not_listed() {
    let w = World::new();
    let mut a = client(&w, Person::A);
    add(&mut a, "ops", "Mine");
    let u = updates(&mut a, json!({"since": "2020-01-01"}));
    assert_eq!(u["total"], 0);
}

#[test]
fn t7_a_move_is_listed_with_from() {
    let w = World::new();
    let mut b = client(&w, Person::B);
    add(&mut b, "ops", "Undo steps");
    let mut a = client(&w, Person::A);
    let first = updates(&mut a, json!({}));
    assert_eq!(first["total"], 1);
    pause();
    let m = b.call(
        "move",
        json!({"from": "ops/undo-steps", "to": "ops/rollback"}),
    );
    assert!(!m.is_error, "{}", m.json);
    let u = updates(&mut a, json!({}));
    let e = entry(&u, "ops/rollback");
    assert_eq!(e["kind"], "moved");
    assert_eq!(e["from"], "ops/undo-steps");
}

#[test]
fn t8_a_new_document_by_someone_else_is_new() {
    let w = World::new();
    let mut b = client(&w, Person::B);
    let (id, _) = add(&mut b, "ops", "Fresh");
    let mut a = client(&w, Person::A);
    let u = updates(&mut a, json!({"since": "2020-01-01"}));
    let e = entry(&u, &id);
    assert_eq!(e["kind"], "new");
    assert_eq!(e["mine"], false);
}

#[test]
fn t9_a_bad_since_is_invalid_arguments() {
    let w = World::new();
    let mut a = client(&w, Person::A);
    for since in ["yesterday", "2999-01-01"] {
        let r = a.call("updates", json!({"since": since}));
        assert_eq!(r.code(), Some("invalid_arguments"), "{since}");
        assert!(r.json["next"].as_str().is_some());
    }
}

#[test]
fn t10_a_change_on_the_remote_is_fetched_first() {
    let w = World::new();
    let mut a = client(&w, Person::A);
    let mut b = client(&w, Person::B);
    add(&mut b, "ops", "From the remote");
    // A has not fetched yet.
    assert!(!w.a.join("agent-memory/ops/from-the-remote.md").exists());
    let u = updates(&mut a, json!({"since": "2020-01-01"}));
    assert_eq!(ids(&u), vec!["ops/from-the-remote"]);
}

#[test]
fn t11_a_deprecation_is_listed() {
    let w = World::new();
    let mut a = client(&w, Person::A);
    let (id, _) = add(&mut a, "ops", "Old way");
    updates(&mut a, json!({}));
    pause();
    let mut b = client(&w, Person::B);
    let v = b.call("show", json!({"id": id}));
    let base = v.json["version"].as_str().unwrap().to_string();
    let d = b.call(
        "deprecate",
        json!({"id": id, "base": base, "reason": "No longer true."}),
    );
    assert!(!d.is_error, "{}", d.json);
    let u = updates(&mut a, json!({}));
    assert_eq!(entry(&u, &id)["kind"], "deprecated");
}

#[test]
fn t12_the_list_is_cut_at_50() {
    let w = World::new();
    // Many files in one commit by B, written by hand.
    for i in 0..60 {
        let text = format!(
            "---\ntitle: Doc {i}\ndescription: About {i}.\nstatus: active\ncreated: 2026-11-01\nupdated: 2026-11-01\ncreated_by: \"Ben Example <ben@example.org>\"\nupdated_by: \"Ben Example <ben@example.org>\"\n---\n\nText.\n"
        );
        common::memrepo::write_doc(&w.b, &format!("bulk/doc-{i}"), &text);
    }
    git(&w.b, &["add", "-A"]);
    git(
        &w.b,
        &[
            "-c",
            "user.name=Ben Example",
            "-c",
            "user.email=ben@example.org",
            "commit",
            "-q",
            "-m",
            "bulk",
        ],
    );
    git(&w.b, &["push", "-q"]);
    let mut a = client(&w, Person::A);
    let u = updates(&mut a, json!({"since": "2020-01-01"}));
    assert_eq!(u["total"], 60);
    assert_eq!(u["truncated"], true);
    assert_eq!(u["changes"].as_array().unwrap().len(), 50);
}

#[test]
fn t13_a_generated_index_is_not_listed() {
    let w = World::new();
    let text = "---\ntitle: Index of ops\ndescription: All of ops.\nstatus: active\ncreated: 2026-11-01\nupdated: 2026-11-01\ncreated_by: \"Ben Example <ben@example.org>\"\nupdated_by: \"Ben Example <ben@example.org>\"\ntags:\n  - index\n---\n\nList.\n";
    common::memrepo::write_doc(&w.b, "ops/index", text);
    git(&w.b, &["add", "-A"]);
    git(
        &w.b,
        &[
            "-c",
            "user.name=Ben Example",
            "-c",
            "user.email=ben@example.org",
            "commit",
            "-q",
            "-m",
            "index",
        ],
    );
    git(&w.b, &["push", "-q"]);
    let mut a = client(&w, Person::A);
    let u = updates(&mut a, json!({"since": "2020-01-01"}));
    assert_eq!(u["total"], 0);
}

#[test]
fn t14_the_state_file_keeps_other_keys() {
    let w = World::new();
    let mut a = client(&w, Person::A);
    let memory_repository_id = {
        let s = a.call("status", json!({}));
        s.json["memory_repository"]["id"]
            .as_str()
            .unwrap()
            .to_string()
    };
    let file = w
        .root
        .join("home-a/state")
        .join(format!("{memory_repository_id}.json"));
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(&file, "{\"other\": \"stays\"}").unwrap();
    updates(&mut a, json!({}));
    let v: Value = serde_json::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();
    assert_eq!(v["other"], "stays");
    assert!(v["last_updates"].as_str().unwrap().ends_with('Z'));
}

#[test]
fn t15_a_young_repository_lists_every_document_as_new() {
    let w = World::new();
    let mut b = client(&w, Person::B);
    add(&mut b, "ops", "One");
    add(&mut b, "ops", "Two");
    let mut a = client(&w, Person::A);
    // No stored time and no since: seven days back. Everything is younger.
    let u = updates(&mut a, json!({}));
    assert_eq!(u["total"], 2);
    for c in u["changes"].as_array().unwrap() {
        assert_eq!(c["kind"], "new");
    }
}
