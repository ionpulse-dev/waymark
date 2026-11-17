//! End-to-end tests of document names, the rules of `sync`, and short wikilinks
//! (see `specs/features/document-names/test.md`).

mod common;
use common::git::{git, subjects};
use common::memrepo::{commit_doc, doc, write_doc, AUTHOR, TODAY};
use common::{McpClient, Person, Spawn, World};
use serde_json::{json, Value};

fn client(w: &World) -> McpClient {
    McpClient::for_person(w, Person::A, Spawn::default())
}

fn personal(w: &World, text: &str) {
    let p = w.root.join("home-a/.config/waymark/config.toml");
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, text).unwrap();
}

fn file(w: &World, id: &str) -> String {
    std::fs::read_to_string(w.a.join("agent-memory").join(format!("{id}.md"))).unwrap()
}

fn exists(w: &World, id: &str) -> bool {
    w.a.join("agent-memory").join(format!("{id}.md")).exists()
}

fn set_verification_on(w: &World) {
    let path = w.a.join("agent-memory/.waymark/settings.toml");
    let old = std::fs::read_to_string(&path).unwrap();
    assert!(old.contains("verification = false"));
    std::fs::write(
        &path,
        old.replacen("verification = false", "verification = true", 1),
    )
    .unwrap();
    git(&w.a, &["add", "-A"]);
    git(&w.a, &["commit", "-q", "-m", "test: settings"]);
    git(&w.a, &["push", "-q"]);
}

fn add(c: &mut McpClient, folder: &str, title: &str, extra: Value) -> common::Reply {
    let mut args = json!({
        "folder": folder,
        "title": title,
        "description": format!("About {title}."),
        "body": "The body text."
    });
    if let (Some(a), Some(e)) = (args.as_object_mut(), extra.as_object()) {
        for (k, v) in e {
            a.insert(k.clone(), v.clone());
        }
    }
    c.call("add_document", args)
}

fn committed_ids(r: &Value) -> Vec<String> {
    r["committed"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["id"].as_str().unwrap().to_string())
        .collect()
}

fn blocked_rules(r: &Value, id: &str) -> Vec<String> {
    r["blocked"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|b| b["id"] == id)
        .flat_map(|b| b["findings"].as_array().unwrap().clone())
        .map(|f| f["rule"].as_str().unwrap().to_string())
        .collect()
}

// ---- the name of add_document ----

#[test]
fn a_name_sets_the_id() {
    let w = World::new();
    let mut c = client(&w);
    let r = add(
        &mut c,
        "people",
        "How to name your folder in people",
        json!({"name": "readme"}),
    );
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(r.json["id"], "people/readme");
    assert_eq!(r.json["title"], "How to name your folder in people");
    assert!(file(&w, "people/readme").contains("title: How to name your folder in people"));
}

#[test]
fn a_taken_name_fails() {
    let w = World::new();
    commit_doc(&w.a, "people/readme", "People readme", "x");
    let mut c = client(&w);
    let r = add(&mut c, "people", "Another text", json!({"name": "readme"}));
    assert_eq!(r.code(), Some("validation_failed"), "{}", r.json);
    assert!(r.message().contains("people/readme"), "{}", r.message());
    assert!(!exists(&w, "people/readme-"));
}

#[test]
fn a_bad_name_fails() {
    let w = World::new();
    let mut c = client(&w);
    let long = "a".repeat(61);
    for bad in ["Readme", "a/b", "readme.md", "-a", "a b", long.as_str()] {
        let r = add(&mut c, "people", "Some title", json!({"name": bad}));
        assert_eq!(r.code(), Some("invalid_arguments"), "{bad}: {}", r.json);
    }
}

#[test]
fn without_a_name_the_title_gives_the_name() {
    let w = World::new();
    let mut c = client(&w);
    let r = add(&mut c, "ops", "Retry policy for payment calls", json!({}));
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(r.json["id"], "ops/retry-policy-for-payment-calls");
}

// ---- short wikilinks ----

#[test]
fn a_second_readme_rewrites_short_links() {
    let w = World::new();
    commit_doc(&w.a, "people/readme", "People readme", "x");
    commit_doc(
        &w.a,
        "ops/overview",
        "Overview",
        "See [[readme]] and [[readme|the rules]].",
    );
    let mut c = client(&w);
    let r = add(
        &mut c,
        "projects",
        "Projects readme",
        json!({"name": "readme"}),
    );
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(
        r.json["links_updated"][0]["id"], "ops/overview",
        "{}",
        r.json
    );
    let t = file(&w, "ops/overview");
    assert!(
        t.contains("See [[people/readme]] and [[people/readme|the rules]]."),
        "{t}"
    );
    // One commit for the new document and the fix.
    let files = git(&w.remote, &["show", "--name-only", "--format=", "HEAD"]);
    assert!(
        files.contains("projects/readme.md") && files.contains("ops/overview.md"),
        "{files}"
    );
    // The first readme keeps its backlink.
    let s = c.call("show", json!({"id": "people/readme"}));
    assert!(
        s.json["backlinks"].to_string().contains("ops/overview"),
        "{}",
        s.json
    );
}

#[test]
fn a_move_to_a_taken_file_name_rewrites_short_links() {
    let w = World::new();
    commit_doc(&w.a, "people/readme", "People readme", "x");
    commit_doc(&w.a, "ops/notes", "Notes", "y");
    commit_doc(&w.a, "ops/overview", "Overview", "See [[readme]].");
    let mut c = client(&w);
    let r = c.call(
        "move",
        json!({"from": "ops/notes", "to": "projects/readme"}),
    );
    assert!(!r.is_error, "{}", r.json);
    assert!(file(&w, "ops/overview").contains("See [[people/readme]]."));
}

// ---- the rules of the write tools in sync ----

#[test]
fn sync_blocks_a_near_duplicate_title() {
    let w = World::new();
    commit_doc(&w.a, "ops/retry", "Retry policy for payment calls", "x");
    write_doc(
        &w.a,
        "ops/retry-copy",
        &doc("The retry policy for payment calls", "y"),
    );
    let mut c = client(&w);
    let r = c.call("sync", json!({"preview": true}));
    assert!(!r.is_error, "{}", r.json);
    assert!(
        blocked_rules(&r.json, "ops/retry-copy").contains(&"near-duplicate-title".to_string()),
        "{}",
        r.json
    );
}

#[test]
fn sync_blocks_a_file_outside_the_folders() {
    let w = World::new();
    personal(&w, "[limits]\nfolders = [\"ops/**\"]\n");
    write_doc(&w.a, "people/readme", &doc("People readme", "x"));
    let mut c = client(&w);
    let r = c.call("sync", json!({"preview": true}));
    assert!(
        blocked_rules(&r.json, "people/readme").contains(&"outside-folders".to_string()),
        "{}",
        r.json
    );
}

#[test]
fn sync_follows_switched_off_tools() {
    let w = World::new();
    commit_doc(&w.a, "ops/deploy", "How we deploy", "Old.");
    personal(&w, "[limits]\ntools_denied = [\"add_document\"]\n");
    write_doc(&w.a, "ops/deploy", &doc("How we deploy", "New."));
    write_doc(&w.a, "ops/new", &doc("A new hand document", "x"));
    let mut c = client(&w);
    let r = c.call("sync", json!({}));
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(committed_ids(&r.json), vec!["ops/deploy"]);
    assert!(
        blocked_rules(&r.json, "ops/new").contains(&"tool-switched-off".to_string()),
        "{}",
        r.json
    );
}

#[test]
fn sync_resets_trust_on_a_changed_verified_document() {
    let w = World::new();
    set_verification_on(&w);
    let verified = doc("How we deploy", "Old.").replace(
        "---\n\n",
        &format!("trust: verified\nverified_by: \"{AUTHOR}\"\nverified: {TODAY}\n---\n\n"),
    );
    write_doc(&w.a, "ops/deploy", &verified);
    git(&w.a, &["add", "-A"]);
    git(&w.a, &["commit", "-q", "-m", "test: verified"]);
    git(&w.a, &["push", "-q"]);
    write_doc(&w.a, "ops/deploy", &verified.replace("Old.", "New."));
    let mut c = client(&w);
    let r = c.call("sync", json!({}));
    assert!(!r.is_error, "{}", r.json);
    let t = file(&w, "ops/deploy");
    assert!(t.contains("trust: unverified"), "{t}");
    assert!(!t.contains("verified_by"), "{t}");
}

#[test]
fn sync_sets_a_new_file_to_unverified() {
    let w = World::new();
    set_verification_on(&w);
    let text = doc("A new hand document", "x").replace(
        "---\n\n",
        &format!("trust: verified\nverified_by: \"{AUTHOR}\"\nverified: {TODAY}\n---\n\n"),
    );
    write_doc(&w.a, "ops/new", &text);
    let mut c = client(&w);
    let r = c.call("sync", json!({}));
    assert!(!r.is_error, "{}", r.json);
    let t = file(&w, "ops/new");
    assert!(t.contains("trust: unverified"), "{t}");
    assert!(!t.contains("verified_by"), "{t}");
}

#[test]
fn sync_makes_one_commit_for_each_edit() {
    let w = World::new();
    commit_doc(&w.a, "ops/deploy", "How we deploy", "Old.");
    write_doc(&w.a, "ops/deploy", &doc("How we deploy", "New."));
    write_doc(&w.a, "ops/new", &doc("A new hand document", "x"));
    let before = subjects(&w.remote).len();
    let mut c = client(&w);
    let r = c.call("sync", json!({}));
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(subjects(&w.remote).len(), before + 2);
    let c0 = r.json["committed"][0]["commit"]
        .as_str()
        .unwrap()
        .to_string();
    let c1 = r.json["committed"][1]["commit"]
        .as_str()
        .unwrap()
        .to_string();
    assert_ne!(c0, c1);
    let bodies = git(&w.remote, &["log", "-2", "--format=%B"]);
    assert!(
        bodies.contains("Action: update") && bodies.contains("Action: add"),
        "{bodies}"
    );
    let a = c.call("activity", json!({}));
    assert_eq!(a.json["entries"].as_array().unwrap().len(), 2, "{}", a.json);
}

#[test]
fn a_rename_by_hand_is_published_as_a_move() {
    let w = World::new();
    commit_doc(&w.a, "ops/deploy", "How we deploy", "Deploy steps.");
    commit_doc(&w.a, "ops/linker", "Linker", "See [d](deploy.md).");
    std::fs::create_dir_all(w.a.join("agent-memory/guides")).unwrap();
    std::fs::rename(
        w.a.join("agent-memory/ops/deploy.md"),
        w.a.join("agent-memory/guides/deploy.md"),
    )
    .unwrap();
    let mut c = client(&w);
    let r = c.call("sync", json!({}));
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(
        r.json["committed"].as_array().unwrap().len(),
        1,
        "{}",
        r.json
    );
    assert_eq!(r.json["committed"][0]["kind"], "moved");
    assert_eq!(r.json["committed"][0]["from"], "ops/deploy");
    assert!(file(&w, "ops/linker").contains("[d](../guides/deploy.md)"));
    let s = c.call("show", json!({"id": "ops/deploy"}));
    assert!(!s.is_error, "{}", s.json);
    assert_eq!(s.json["id"], "guides/deploy");
    assert!(git(&w.a, &["status", "--porcelain"]).trim().is_empty());
}

#[test]
fn the_sync_description_names_manual_edits() {
    let w = World::new();
    let mut c = client(&w);
    let tools = c.list_tools();
    let sync = tools.iter().find(|t| t["name"] == "sync").unwrap();
    let d = sync["description"].as_str().unwrap();
    assert!(
        d.contains("manual edits") && d.contains("add_document"),
        "{d}"
    );
    for word in ["should", "only when", "please"] {
        assert!(!d.to_lowercase().contains(word), "{d}");
    }
}

#[test]
fn sync_does_not_let_a_hand_edit_verify() {
    let w = World::new();
    set_verification_on(&w);
    let unverified =
        doc("How we deploy", "Steps.").replace("---\n\n", "trust: unverified\n---\n\n");
    write_doc(&w.a, "ops/deploy", &unverified);
    git(&w.a, &["add", "-A"]);
    git(&w.a, &["commit", "-q", "-m", "test: unverified"]);
    git(&w.a, &["push", "-q"]);
    let by_hand = unverified.replace(
        "trust: unverified\n",
        &format!("trust: verified\nverified_by: \"{AUTHOR}\"\nverified: {TODAY}\n"),
    );
    write_doc(&w.a, "ops/deploy", &by_hand);
    let mut c = client(&w);
    let r = c.call("sync", json!({}));
    assert!(!r.is_error, "{}", r.json);
    let t = file(&w, "ops/deploy");
    assert!(t.contains("trust: unverified"), "{t}");
    assert!(!t.contains("verified_by"), "{t}");
}

#[test]
fn two_conflicting_hand_edits_keep_both_versions() {
    let w = World::new();
    commit_doc(&w.a, "ops/one", "First document", "A1.");
    commit_doc(&w.a, "ops/two", "Second document", "A2.");
    // B changes both documents and pushes.
    git(&w.b, &["pull", "-q", "--rebase"]);
    write_doc(&w.b, "ops/one", &doc("First document", "B version one."));
    write_doc(&w.b, "ops/two", &doc("Second document", "B version two."));
    git(&w.b, &["add", "-A"]);
    git(&w.b, &["commit", "-q", "-m", "test: b edits"]);
    git(&w.b, &["push", "-q"]);
    // A changes both by hand: two sync commits.
    write_doc(&w.a, "ops/one", &doc("First document", "A version one."));
    write_doc(&w.a, "ops/two", &doc("Second document", "A version two."));
    let mut c = client(&w);
    let r = c.call("sync", json!({}));
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(
        r.json["conflicts"].as_array().unwrap().len(),
        2,
        "{}",
        r.json
    );
    assert!(file(&w, "ops/one").contains("B version one."));
    assert!(file(&w, "ops/two").contains("B version two."));
    assert!(git(&w.a, &["status", "--porcelain"]).trim().is_empty());
}

#[test]
fn a_hand_rename_is_one_activity_entry_and_undo_takes_back_both() {
    let w = World::new();
    commit_doc(&w.a, "ops/deploy", "How we deploy", "Deploy steps.");
    commit_doc(&w.a, "ops/linker", "Linker", "See [d](deploy.md).");
    std::fs::create_dir_all(w.a.join("agent-memory/guides")).unwrap();
    std::fs::rename(
        w.a.join("agent-memory/ops/deploy.md"),
        w.a.join("agent-memory/guides/deploy.md"),
    )
    .unwrap();
    let mut c = client(&w);
    let r = c.call("sync", json!({}));
    assert!(!r.is_error, "{}", r.json);
    let a = c.call("activity", json!({}));
    let entries = a.json["entries"].as_array().unwrap().clone();
    assert_eq!(entries.len(), 1, "{}", a.json);
    let commit = entries[0]["commit"].as_str().unwrap().to_string();
    let u = c.call("undo", json!({"commit": commit}));
    assert!(!u.is_error, "{}", u.json);
    assert!(exists(&w, "ops/deploy"));
    assert!(!exists(&w, "guides/deploy"));
    assert!(file(&w, "ops/linker").contains("[d](deploy.md)"));
}

#[test]
fn a_read_only_preview_does_not_report_switched_off_tools() {
    let w = World::new();
    write_doc(&w.a, "ops/new", &doc("A new hand document", "x"));
    let mut c = McpClient::for_person(&w, Person::A, Spawn::args(&["--access", "read-only"]));
    let r = c.call("sync", json!({"preview": true}));
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(committed_ids(&r.json), vec!["ops/new"], "{}", r.json);
    assert!(!r.json.to_string().contains("tool-switched-off"));
}

#[test]
fn a_hand_rename_stays_one_entry_after_a_pull_with_rebase() {
    let w = World::new();
    commit_doc(&w.a, "ops/deploy", "How we deploy", "Deploy steps.");
    commit_doc(&w.a, "ops/linker", "Linker", "See [d](deploy.md).");
    // B pushes a commit, so A's sync rebases its two commits.
    git(&w.b, &["pull", "-q", "--rebase"]);
    commit_doc(&w.b, "ops/from-b", "From B", "x");
    std::fs::create_dir_all(w.a.join("agent-memory/guides")).unwrap();
    std::fs::rename(
        w.a.join("agent-memory/ops/deploy.md"),
        w.a.join("agent-memory/guides/deploy.md"),
    )
    .unwrap();
    let mut c = client(&w);
    let r = c.call("sync", json!({}));
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(r.json["pulled"], 1, "{}", r.json);
    let a = c.call("activity", json!({}));
    let entries = a.json["entries"].as_array().unwrap().clone();
    assert_eq!(entries.len(), 1, "{}", a.json);
    let commit = entries[0]["commit"].as_str().unwrap().to_string();
    let u = c.call("undo", json!({"commit": commit}));
    assert!(!u.is_error, "{}", u.json);
    assert!(exists(&w, "ops/deploy"));
    assert!(file(&w, "ops/linker").contains("[d](deploy.md)"));
}

#[test]
fn a_delete_waits_while_a_blocked_edit_still_links_in_its_committed_text() {
    let w = World::new();
    commit_doc(&w.a, "ops/a", "Target document", "T.");
    commit_doc(&w.a, "ops/b", "Linking document", "See [a](a.md).");
    commit_doc(&w.a, "ops/retry", "Retry policy for payment calls", "x");
    // The person deletes ops/a and edits ops/b: the link goes, but the new title of ops/b
    // is a near-duplicate, so ops/b is blocked and its committed text still links.
    std::fs::remove_file(w.a.join("agent-memory/ops/a.md")).unwrap();
    write_doc(
        &w.a,
        "ops/b",
        &doc("The retry policy for payment calls", "No link now."),
    );
    let mut c = client(&w);
    let r = c.call("sync", json!({"preview": true}));
    assert!(!r.is_error, "{}", r.json);
    assert!(committed_ids(&r.json).is_empty(), "{}", r.json);
    assert!(
        blocked_rules(&r.json, "ops/a").contains(&"broken-link".to_string()),
        "{}",
        r.json
    );
}
