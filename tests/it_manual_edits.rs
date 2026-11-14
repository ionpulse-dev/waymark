//! End-to-end tests of `sync` and manual edits (see `specs/features/manual-edits/test.md`).

mod common;
use common::git::{git, subjects};
use common::memrepo::{commit_doc, write_doc};
use common::{McpClient, Person, Spawn, World};
use serde_json::{json, Value};

const ANNA: &str = "Anna Example <anna@example.org>";

fn client(w: &World) -> McpClient {
    McpClient::for_person(w, Person::A, Spawn::default())
}

fn text(title: &str, updated: &str, body: &str) -> String {
    format!(
        "---\ntitle: \"{title}\"\ndescription: About {title}.\nstatus: active\ncreated: 2020-01-01\nupdated: {updated}\ncreated_by: \"{ANNA}\"\nupdated_by: \"Old Author <old@example.org>\"\n---\n\n{body}\n"
    )
}

fn setup(w: &World) {
    commit_doc(&w.a, "ops/deploy", "How we deploy", "Original body.");
}

/// Bring clone B up to date, as a person would.
fn pull_b(w: &World) {
    git(&w.b, &["pull", "-q", "--rebase"]);
}

fn edit_deploy(w: &World, body: &str) {
    write_doc(
        &w.a,
        "ops/deploy",
        &text("How we deploy", "2020-01-02", body),
    );
}

fn ids(r: &Value, key: &str) -> Vec<String> {
    r[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["id"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn t1_an_edit_is_committed_and_pushed() {
    let w = World::new();
    setup(&w);
    edit_deploy(&w, "New body.");
    let mut c = client(&w);
    let r = c.call("sync", json!({}));
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(ids(&r.json, "committed"), vec!["ops/deploy"]);
    assert_eq!(r.json["committed"][0]["kind"], "changed");
    assert_eq!(r.json["pushed"], true);
    assert_eq!(subjects(&w.remote)[0], "waymark: sync \"How we deploy\"");
    assert_eq!(
        git(&w.remote, &["log", "-1", "--format=%an <%ae>"]).trim(),
        ANNA
    );
    assert!(git(&w.a, &["status", "--porcelain"]).trim().is_empty());
}

#[test]
fn t2_a_failing_edit_alone_fails_and_pushes_nothing() {
    let w = World::new();
    setup(&w);
    write_doc(&w.a, "ops/new", "no frontmatter\n");
    let before = subjects(&w.remote);
    let mut c = client(&w);
    let r = c.call("sync", json!({}));
    assert_eq!(r.code(), Some("validation_failed"));
    assert!(r.message().contains("ops/new"));
    assert!(r.json["next"].is_string());
    assert_eq!(subjects(&w.remote), before);
    assert_eq!(
        std::fs::read_to_string(w.a.join("agent-memory/ops/new.md")).unwrap(),
        "no frontmatter\n"
    );
}

#[test]
fn t3_a_good_edit_goes_and_a_bad_one_is_blocked() {
    let w = World::new();
    setup(&w);
    edit_deploy(&w, "New body.");
    write_doc(&w.a, "ops/bad", "no frontmatter\n");
    let mut c = client(&w);
    let r = c.call("sync", json!({}));
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(ids(&r.json, "committed"), vec!["ops/deploy"]);
    assert_eq!(r.json["blocked"][0]["id"], "ops/bad");
    assert_eq!(
        r.json["blocked"][0]["findings"][0]["rule"],
        "no-frontmatter"
    );
    assert_eq!(subjects(&w.remote)[0], "waymark: sync \"How we deploy\"");
    assert!(git(&w.a, &["status", "--porcelain"]).contains("ops/bad.md"));
}

#[test]
fn t4_manual_edits_are_committed_before_an_agent_write() {
    let w = World::new();
    setup(&w);
    edit_deploy(&w, "New body.");
    let mut c = client(&w);
    let r = c.call(
        "add_document",
        json!({"folder": "ops", "title": "Other doc", "description": "D.", "body": "B."}),
    );
    assert!(!r.is_error, "{}", r.json);
    let log = subjects(&w.a);
    assert_eq!(log[0], "waymark: add \"Other doc\"");
    assert_eq!(log[1], "waymark: sync \"How we deploy\"");
}

#[test]
fn t5_a_bad_edit_does_not_stop_another_write_and_gives_a_notice() {
    let w = World::new();
    setup(&w);
    write_doc(&w.a, "ops/bad", "no frontmatter\n");
    let mut c = client(&w);
    let r = c.call(
        "add_document",
        json!({"folder": "ops", "title": "Other doc", "description": "D.", "body": "B."}),
    );
    assert!(!r.is_error, "{}", r.json);
    assert!(r.json["notices"]
        .to_string()
        .contains("1 manual edit is not published. Run `sync`."));
}

#[test]
fn t6_a_write_to_a_document_with_a_blocked_edit_is_stale() {
    let w = World::new();
    setup(&w);
    let long = "one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen";
    write_doc(&w.a, "ops/deploy", &text(long, "2020-01-02", "x"));
    let version = git(
        &w.a,
        &[
            "log",
            "-1",
            "--format=%h",
            "--",
            "agent-memory/ops/deploy.md",
        ],
    );
    let mut c = client(&w);
    let r = c.call(
        "update_document",
        json!({"id": "ops/deploy", "base": version.trim(), "body": "y"}),
    );
    assert_eq!(r.code(), Some("stale_base"));
    assert!(r.json["next"].as_str().unwrap().contains("sync"));
}

#[test]
fn t8_read_only_only_pulls() {
    let w = World::new();
    setup(&w);
    edit_deploy(&w, "Pending.");
    pull_b(&w);
    commit_doc(&w.b, "ops/from-b", "From B", "x");
    let mut c = McpClient::for_person(&w, Person::A, Spawn::args(&["--access", "read-only"]));
    assert!(c.tool_names().contains(&"sync".to_string()));
    let before = subjects(&w.remote);
    let r = c.call("sync", json!({}));
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(r.json["committed"], json!([]));
    assert_eq!(r.json["pushed"], false);
    assert_eq!(r.json["pulled"], 1);
    assert!(w.a.join("agent-memory/ops/from-b.md").exists());
    assert!(git(&w.a, &["status", "--porcelain"]).contains("ops/deploy.md"));
    assert_eq!(subjects(&w.remote), before);
    assert!(r.json["notices"]
        .to_string()
        .contains("Writes are not allowed"));
}

#[test]
fn t9_updated_and_updated_by_are_set_and_the_body_is_kept() {
    let w = World::new();
    setup(&w);
    edit_deploy(&w, "Body with\n\n- a list\n");
    let mut c = client(&w);
    let r = c.call("sync", json!({}));
    assert!(!r.is_error, "{}", r.json);
    let t = std::fs::read_to_string(w.a.join("agent-memory/ops/deploy.md")).unwrap();
    let today = chrono_today();
    assert!(t.contains(&format!("updated: {today}\n")), "{t}");
    assert!(
        t.contains(&format!("updated_by: \"{ANNA}\""))
            || t.contains(&format!("updated_by: {ANNA}")),
        "{t}"
    );
    assert!(t.ends_with("\n\nBody with\n\n- a list\n\n"), "{t:?}");
}

fn chrono_today() -> String {
    String::from_utf8(
        std::process::Command::new("date")
            .arg("+%Y-%m-%d")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
    .trim()
    .to_string()
}

#[test]
fn t10_a_hand_move_is_followed_by_show() {
    let w = World::new();
    setup(&w);
    std::fs::create_dir_all(w.a.join("agent-memory/guides")).unwrap();
    std::fs::rename(
        w.a.join("agent-memory/ops/deploy.md"),
        w.a.join("agent-memory/guides/deploy.md"),
    )
    .unwrap();
    let mut c = client(&w);
    let r = c.call("sync", json!({}));
    assert!(!r.is_error, "{}", r.json);
    let kinds: Vec<&str> = r.json["committed"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["kind"].as_str().unwrap())
        .collect();
    assert!(
        kinds.contains(&"deleted") && kinds.contains(&"new"),
        "{kinds:?}"
    );
    let s = c.call("show", json!({"id": "ops/deploy"}));
    assert!(!s.is_error, "{}", s.json);
    assert_eq!(s.json["id"], "guides/deploy");
    assert_eq!(s.json["moved_from"], "ops/deploy");
}

#[test]
fn t11_t13_other_files_are_skipped() {
    let w = World::new();
    setup(&w);
    std::fs::create_dir_all(w.a.join("agent-memory/images")).unwrap();
    std::fs::write(w.a.join("agent-memory/images/diagram.png"), b"png").unwrap();
    let settings = w.a.join("agent-memory/.waymark/settings.toml");
    let old = std::fs::read_to_string(&settings).unwrap();
    std::fs::write(&settings, format!("{old}\n# edited by hand\n")).unwrap();
    edit_deploy(&w, "x");
    let mut c = client(&w);
    let r = c.call("sync", json!({}));
    assert!(!r.is_error, "{}", r.json);
    let skipped = r.json["skipped"].to_string();
    assert!(
        skipped.contains("diagram.png") && skipped.contains("settings.toml"),
        "{skipped}"
    );
    let files = git(&w.remote, &["show", "--name-only", "--format=", "HEAD"]);
    assert!(
        files.contains("ops/deploy.md")
            && !files.contains("diagram")
            && !files.contains("settings.toml"),
        "{files}"
    );
}

#[test]
fn t12_a_preview_changes_nothing() {
    let w = World::new();
    setup(&w);
    edit_deploy(&w, "x");
    let before = subjects(&w.remote);
    let local_before = subjects(&w.a);
    let mut c = client(&w);
    let r = c.call("sync", json!({"preview": true}));
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(ids(&r.json, "committed"), vec!["ops/deploy"]);
    assert!(r.json["committed"][0].get("commit").is_none());
    assert_eq!(r.json["pushed"], false);
    assert_eq!(subjects(&w.remote), before);
    assert_eq!(subjects(&w.a), local_before);
    assert!(git(&w.a, &["status", "--porcelain"]).contains("ops/deploy.md"));
}

#[test]
fn t14_a_conflict_keeps_both_versions() {
    let w = World::new();
    setup(&w);
    // B changes the document and pushes.
    pull_b(&w);
    write_doc(
        &w.b,
        "ops/deploy",
        &text("How we deploy", "2020-01-02", "B version"),
    );
    git(&w.b, &["add", "-A"]);
    git(&w.b, &["commit", "-q", "-m", "test: b edit"]);
    git(&w.b, &["push", "-q"]);
    edit_deploy(&w, "A version");
    let mut c = client(&w);
    let r = c.call("sync", json!({}));
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(
        r.json["conflicts"].as_array().unwrap().len(),
        1,
        "{}",
        r.json
    );
    assert!(r.json["notices"]
        .to_string()
        .contains("Both versions are kept"));
    let kept = std::fs::read_to_string(w.a.join("agent-memory/ops/deploy.md")).unwrap();
    assert!(kept.contains("B version"));
    let copy = r.json["conflicts"][0]["copy"].as_str().unwrap();
    assert!(
        std::fs::read_to_string(w.a.join(format!("agent-memory/{copy}.md")))
            .unwrap()
            .contains("A version")
    );
}

#[test]
fn t15_a_pending_delete_is_committed() {
    let w = World::new();
    setup(&w);
    std::fs::remove_file(w.a.join("agent-memory/ops/deploy.md")).unwrap();
    let mut c = client(&w);
    let r = c.call("sync", json!({}));
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(r.json["committed"][0]["kind"], "deleted");
    assert_eq!(subjects(&w.remote)[0], "waymark: sync \"How we deploy\"");
    assert!(!git(&w.remote, &["ls-tree", "-r", "--name-only", "HEAD"]).contains("deploy"));
}

#[test]
fn t16_lint_rules_apply_to_pending_edits() {
    let w = World::new();
    setup(&w);
    write_doc(
        &w.a,
        "ops/copy",
        &text("How we deploy", "2020-01-02", "dup")
            .replace("created: 2020-01-01", "created: 2026-12-01"),
    );
    let mut c = client(&w);
    let r = c.call("sync", json!({}));
    assert_eq!(r.code(), Some("validation_failed"), "{}", r.json);
    let r = c.call("sync", json!({"preview": true}));
    assert_eq!(
        r.json["blocked"][0]["findings"][0]["rule"],
        "duplicate-title"
    );
}

#[test]
fn t17_a_clean_tree_makes_no_commit() {
    let w = World::new();
    setup(&w);
    pull_b(&w);
    commit_doc(&w.b, "ops/from-b", "From B", "x");
    let before = subjects(&w.remote);
    let mut c = client(&w);
    let r = c.call("sync", json!({}));
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(r.json["committed"], json!([]));
    assert_eq!(r.json["pulled"], 1);
    assert_eq!(subjects(&w.remote), before);
    assert!(w.a.join("agent-memory/ops/from-b.md").exists());
}

#[test]
fn t18_no_remote_commits_locally_with_a_notice() {
    let w = World::new();
    setup(&w);
    git(&w.a, &["remote", "remove", "origin"]);
    edit_deploy(&w, "x");
    let mut c = client(&w);
    let r = c.call("sync", json!({}));
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(ids(&r.json, "committed"), vec!["ops/deploy"]);
    assert_eq!(r.json["pushed"], false);
    assert!(r.json["notices"].to_string().contains("no remote"));
    assert_eq!(subjects(&w.a)[0], "waymark: sync \"How we deploy\"");
}

#[test]
fn t19_a_preview_write_does_not_commit_pending_edits() {
    let w = World::new();
    setup(&w);
    edit_deploy(&w, "x");
    let before = subjects(&w.a);
    let mut c = client(&w);
    let r = c.call(
        "add_document",
        json!({"folder": "ops", "title": "Other doc", "description": "D.", "body": "B.", "preview": true}),
    );
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(subjects(&w.a), before);
    assert!(git(&w.a, &["status", "--porcelain"]).contains("ops/deploy.md"));
}

#[test]
fn a_new_broken_body_link_in_a_manual_edit_is_blocked() {
    let w = World::new();
    setup(&w);
    edit_deploy(&w, "See [x](gone.md).");
    let head = subjects(&w.remote);
    let mut c = client(&w);
    let r = c.call("sync", json!({}));
    assert_eq!(r.code(), Some("validation_failed"), "{}", r.json);
    assert!(r.message().contains("ops/gone.md"), "{}", r.message());
    assert_eq!(subjects(&w.remote), head);
    // The person fixes the link and the edit goes.
    edit_deploy(&w, "No link now.");
    let r = c.call("sync", json!({}));
    assert_eq!(ids(&r.json, "committed"), vec!["ops/deploy"], "{}", r.json);
}

#[test]
fn an_old_broken_link_does_not_block_a_manual_edit() {
    let w = World::new();
    commit_doc(&w.a, "ops/deploy", "How we deploy", "See [x](gone.md).");
    edit_deploy(&w, "See [x](gone.md) and more text.");
    let mut c = client(&w);
    let r = c.call("sync", json!({}));
    assert_eq!(ids(&r.json, "committed"), vec!["ops/deploy"], "{}", r.json);
}

#[test]
fn deleting_a_linked_document_by_hand_waits_until_the_links_are_gone() {
    let w = World::new();
    commit_doc(&w.a, "ops/target", "Target", "T.");
    commit_doc(&w.a, "ops/linker", "Linker", "See [t](target.md).");
    std::fs::remove_file(w.a.join("agent-memory/ops/target.md")).unwrap();
    let mut c = client(&w);
    let r = c.call("sync", json!({}));
    assert_eq!(r.code(), Some("validation_failed"), "{}", r.json);
    assert!(r.message().contains("ops/linker"), "{}", r.message());
    // Both documents go together when the link goes too.
    write_doc(
        &w.a,
        "ops/linker",
        &text("Linker", "2020-01-02", "No link now."),
    );
    let r = c.call("sync", json!({}));
    assert!(
        ids(&r.json, "committed").contains(&"ops/target".to_string()),
        "{}",
        r.json
    );
}
