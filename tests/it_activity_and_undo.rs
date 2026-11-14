//! End-to-end tests of `activity` and `undo` (see `specs/features/activity-and-undo/test.md`).

mod common;
use common::git::{git, subjects};
use common::{McpClient, Person, Spawn, World};
use serde_json::{json, Value};

fn client(w: &World, p: Person) -> McpClient {
    McpClient::for_person(w, p, Spawn::default())
}

fn add(c: &mut McpClient, folder: &str, title: &str, body: &str) -> Value {
    let r = c.call(
        "add",
        json!({"folder": folder, "title": title, "description": format!("About {title}."), "body": body}),
    );
    assert!(!r.is_error, "{}", r.json);
    r.json
}

fn entries(c: &mut McpClient, args: Value) -> Vec<Value> {
    let r = c.call("activity", args);
    assert!(!r.is_error, "{}", r.json);
    r.json["entries"].as_array().unwrap().clone()
}

fn file(w: &World, id: &str) -> String {
    std::fs::read_to_string(w.a.join("knowledge").join(format!("{id}.md"))).unwrap()
}

#[test]
fn activity_lists_newest_first_and_only_my_writes() {
    let w = World::new();
    let mut a = client(&w, Person::A);
    let mut b = client(&w, Person::B);
    add(&mut a, "ops", "First document", "x");
    add(&mut b, "ops", "Ben document", "x");
    add(&mut a, "ops", "Second document", "x");
    // A plain Git commit does not show.
    std::fs::write(w.a.join("knowledge/ops/plain.md"), "---\ntitle: P\n---\n").unwrap();
    git(&w.a, &["add", "-A"]);
    git(&w.a, &["commit", "-q", "-m", "plain commit"]);
    let e = entries(&mut a, json!({}));
    let titles: Vec<&str> = e.iter().map(|x| x["title"].as_str().unwrap()).collect();
    assert_eq!(titles, vec!["Second document", "First document"]);
    assert_eq!(e[0]["verb"], "add");
    assert_eq!(e[0]["folder"], "ops");
    assert_eq!(e[0]["ids"], json!(["ops/second-document"]));
    assert_eq!(e[0]["files"], 1);
    assert_eq!(e[0]["undone"], false);
    assert!(e[0]["time"].as_str().unwrap().contains('T'));
}

#[test]
fn undo_reverts_and_pushes() {
    let w = World::new();
    let mut a = client(&w, Person::A);
    let added = add(&mut a, "ops", "Doc one", "x");
    let r = a.call("undo", json!({"commit": added["commit"]}));
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(
        r.json["changes"],
        json!([{"id": "ops/doc-one", "action": "removed"}])
    );
    assert_eq!(r.json["pushed"], true);
    assert!(!w.a.join("knowledge/ops/doc-one.md").exists());
    let log = subjects(&w.remote);
    assert_eq!(log[0], "waymark: undo \"Doc one\"");
    let body = git(&w.remote, &["log", "-1", "--format=%b"]);
    assert_eq!(
        body.trim(),
        format!("Reverts: {}", added["commit"].as_str().unwrap())
    );
    assert_eq!(
        git(&w.remote, &["log", "-1", "--format=%ae"]).trim(),
        "anna@example.org"
    );
    // The entry is marked as undone, the undo itself is listed first.
    let e = entries(&mut a, json!({}));
    assert_eq!(e[0]["verb"], "undo");
    assert_eq!(e[1]["undone"], true);
    // A second undo fails.
    let again = a.call("undo", json!({"commit": added["commit"]}));
    assert_eq!(again.code(), Some("validation_failed"));
    assert!(again.message().contains("already undone"));
}

#[test]
fn undo_restores_the_text_of_update_dispute_deprecate_and_delete() {
    let w = World::new();
    let mut a = client(&w, Person::A);
    let added = add(&mut a, "ops", "Doc one", "Original body.");
    let id = "ops/doc-one";
    let original = file(&w, id);

    let u = a.call(
        "update_document",
        json!({"id": id, "base": added["version"], "body": "Changed body."}),
    );
    assert!(file(&w, id).contains("Changed body."));
    assert!(!a.call("undo", json!({"commit": u.json["commit"]})).is_error);
    assert_eq!(file(&w, id), original);

    let base = a.call("show", json!({"id": id})).json["version"].clone();
    let d = a.call(
        "dispute",
        json!({"id": id, "base": base, "reason": "Wrong."}),
    );
    assert!(file(&w, id).contains("disputed: true"));
    assert!(!a.call("undo", json!({"commit": d.json["commit"]})).is_error);
    assert_eq!(file(&w, id), original);

    let base = a.call("show", json!({"id": id})).json["version"].clone();
    let dep = a.call(
        "deprecate",
        json!({"id": id, "base": base, "reason": "Old."}),
    );
    assert!(file(&w, id).contains("status: deprecated"));
    assert!(
        !a.call("undo", json!({"commit": dep.json["commit"]}))
            .is_error
    );
    assert_eq!(file(&w, id), original);

    let base = a.call("show", json!({"id": id})).json["version"].clone();
    let del = a.call("delete", json!({"id": id, "base": base}));
    assert!(!w.a.join("knowledge/ops/doc-one.md").exists());
    let r = a.call("undo", json!({"commit": del.json["commit"]}));
    assert_eq!(r.json["changes"], json!([{"id": id, "action": "restored"}]));
    assert_eq!(file(&w, id), original);
}

#[test]
fn undo_of_an_undo_is_a_redo() {
    let w = World::new();
    let mut a = client(&w, Person::A);
    let added = add(&mut a, "ops", "Doc one", "x");
    let u = a.call("undo", json!({"commit": added["commit"]}));
    assert!(!w.a.join("knowledge/ops/doc-one.md").exists());
    let redo = a.call("undo", json!({"commit": u.json["commit"]}));
    assert!(!redo.is_error, "{}", redo.json);
    assert!(w.a.join("knowledge/ops/doc-one.md").exists());
    let e = entries(&mut a, json!({}));
    let first = e.iter().find(|x| x["commit"] == added["commit"]).unwrap();
    assert_eq!(first["undone"], false);
}

#[test]
fn undo_with_a_later_change_to_another_part_reverts_cleanly() {
    let w = World::new();
    let mut a = client(&w, Person::A);
    let body = "Line one.\n\nLine two.\n\nLine three.\n\nLine four.\n\nLine five.\n\nLine six.\n\nLine seven.\n\nLine eight.";
    let added = add(&mut a, "ops", "Doc one", body);
    let id = "ops/doc-one";
    // A changes the first line, then B changes the last line.
    let u = a.call(
        "update_document",
        json!({"id": id, "base": added["version"], "body": body.replace("Line one.", "Line ONE.")}),
    );
    assert!(!u.is_error, "{}", u.json);
    let mut b = client(&w, Person::B);
    let shown = b.call("show", json!({"id": id}));
    let ub = b.call("update_document", json!({"id": id, "base": shown.json["version"], "body": body.replace("Line one.", "Line ONE.").replace("Line eight.", "Line EIGHT.")}));
    assert!(!ub.is_error, "{}", ub.json);
    // A undoes the first update: Git reverts the first line and keeps B's change.
    git(&w.a, &["pull", "-q"]);
    let e = entries(&mut a, json!({}));
    assert_eq!(e[0]["later_changes"], true);
    let r = a.call("undo", json!({"commit": u.json["commit"]}));
    assert!(!r.is_error, "{}", r.json);
    let text = file(&w, id);
    assert!(
        text.contains("Line one.") && text.contains("Line EIGHT."),
        "{text}"
    );
}

#[test]
fn undo_with_a_conflicting_later_change_is_stale_and_changes_nothing() {
    let w = World::new();
    let mut a = client(&w, Person::A);
    let added = add(&mut a, "ops", "Doc one", "Original line.");
    let id = "ops/doc-one";
    let u = a.call(
        "update_document",
        json!({"id": id, "base": added["version"], "body": "A line."}),
    );
    let mut b = client(&w, Person::B);
    let shown = b.call("show", json!({"id": id}));
    b.call(
        "update_document",
        json!({"id": id, "base": shown.json["version"], "body": "B line."}),
    );
    git(&w.a, &["pull", "-q"]);
    let before = subjects(&w.remote);
    let local_before = subjects(&w.a);
    let r = a.call("undo", json!({"commit": u.json["commit"]}));
    assert_eq!(r.code(), Some("stale_base"), "{}", r.json);
    assert!(r.json["next"].as_str().unwrap().contains("update_document"));
    assert!(r.message().contains("Ben"));
    assert_eq!(subjects(&w.remote), before);
    assert_eq!(subjects(&w.a), local_before);
    assert!(git(&w.a, &["status", "--porcelain"]).trim().is_empty());
    assert!(file(&w, id).contains("B line."));
}

#[test]
fn undo_of_another_persons_commit_is_not_found() {
    let w = World::new();
    let mut a = client(&w, Person::A);
    let mut b = client(&w, Person::B);
    let ben = add(&mut b, "ops", "Ben document", "x");
    git(&w.a, &["pull", "-q"]);
    let r = a.call("undo", json!({"commit": ben["commit"]}));
    assert_eq!(r.code(), Some("not_found"));
    assert!(r.json["next"].as_str().unwrap().contains("activity"));
    let bad = a.call("undo", json!({"commit": "xyz"}));
    assert_eq!(bad.code(), Some("invalid_arguments"));
}

#[test]
fn read_only_denies_undo() {
    let w = World::new();
    let mut a = client(&w, Person::A);
    let added = add(&mut a, "ops", "Doc one", "x");
    let mut ro = McpClient::for_person(&w, Person::A, Spawn::args(&["--access", "read-only"]));
    assert!(ro.tool_names().contains(&"activity".to_string()));
    assert!(!ro.tool_names().contains(&"undo".to_string()));
    let r = ro.call("undo", json!({"commit": added["commit"]}));
    assert_eq!(r.code(), Some("access_denied"));
}

#[test]
fn undo_of_a_move_is_one_commit() {
    let w = World::new();
    let mut a = client(&w, Person::A);
    let target = add(&mut a, "ops", "Target doc", "x");
    add(&mut a, "dev", "Linker", "See [[ops/target-doc]].");
    let mv = a.call(
        "move",
        json!({"from": "ops/target-doc", "to": "archive/target-doc"}),
    );
    assert!(!mv.is_error, "{}", mv.json);
    let _ = target;
    let e = entries(&mut a, json!({}));
    assert_eq!(e[0]["verb"], "move");
    assert_eq!(e[0]["files"], 3, "{e:?}");
    let before = subjects(&w.remote).len();
    let r = a.call("undo", json!({"commit": e[0]["commit"]}));
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(subjects(&w.remote).len(), before + 1);
    assert!(w.a.join("knowledge/ops/target-doc.md").exists());
    assert!(!w.a.join("knowledge/archive/target-doc.md").exists());
    assert!(file(&w, "dev/linker").contains("[[ops/target-doc]]"));
    let body = git(&w.remote, &["log", "-1", "--format=%b"]);
    assert_eq!(
        body.split_whitespace().count(),
        3,
        "Reverts: and two hashes: {body}"
    );
}

#[test]
fn paging_with_limit_and_before() {
    let w = World::new();
    let mut a = client(&w, Person::A);
    for i in 0..12 {
        add(&mut a, "ops", &format!("Document number {i} here"), "x");
    }
    let r = a.call("activity", json!({"limit": 5}));
    let first: Vec<String> = r.json["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["title"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(first.len(), 5);
    let next = r.json["next_before"].as_str().unwrap().to_string();
    let r2 = a.call("activity", json!({"limit": 5, "before": next}));
    let second: Vec<String> = r2.json["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["title"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(second.len(), 5);
    assert_eq!(first[0], "Document number 11 here");
    assert_eq!(second[0], "Document number 6 here");
    let r3 = a.call(
        "activity",
        json!({"limit": 5, "before": r2.json["next_before"]}),
    );
    assert_eq!(r3.json["entries"].as_array().unwrap().len(), 2);
    assert!(r3.json["next_before"].is_null());
    // `since` in the future leaves everything out.
    assert!(entries(&mut a, json!({"since": "2999-01-01"})).is_empty());
    assert_eq!(
        entries(&mut a, json!({"since": "2000-01-01", "limit": 50})).len(),
        12
    );
}

#[test]
fn undo_preview_commits_nothing() {
    let w = World::new();
    let mut a = client(&w, Person::A);
    let added = add(&mut a, "ops", "Doc one", "x");
    let before = subjects(&w.remote);
    let r = a.call("undo", json!({"commit": added["commit"], "preview": true}));
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(
        r.json["changes"],
        json!([{"id": "ops/doc-one", "action": "removed"}])
    );
    assert!(r.json["commit"].is_null() && r.json["pushed"].is_null());
    assert!(w.a.join("knowledge/ops/doc-one.md").exists());
    assert_eq!(subjects(&w.remote), before);
    assert!(git(&w.a, &["status", "--porcelain"]).trim().is_empty());
}

#[test]
fn pending_manual_edits_on_the_same_file_give_stale_base() {
    let w = World::new();
    let mut a = client(&w, Person::A);
    let added = add(&mut a, "ops", "Doc one", "x");
    // A manual edit that the checks block (no frontmatter) stays pending.
    std::fs::write(w.a.join("knowledge/ops/doc-one.md"), "no frontmatter\n").unwrap();
    let r = a.call("undo", json!({"commit": added["commit"]}));
    assert_eq!(r.code(), Some("stale_base"), "{}", r.json);
    assert!(r.json["next"].as_str().unwrap().contains("sync"));
}

#[test]
fn undo_outside_the_folders_limit_is_denied() {
    let w = World::new();
    let mut a = client(&w, Person::A);
    let added = add(&mut a, "ops", "Doc one", "x");
    let mut c = McpClient::for_person(&w, Person::A, Spawn::args(&["--folders", "dev/**"]));
    let r = c.call("undo", json!({"commit": added["commit"]}));
    // The entry is hidden by the limit: the activity of the person shows only allowed folders.
    assert!(
        matches!(r.code(), Some("access_denied") | Some("not_found")),
        "{}",
        r.json
    );
    assert!(w.a.join("knowledge/ops/doc-one.md").exists());
}
