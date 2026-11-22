//! End-to-end tests of `lint` (see `specs/features/quality-checks/test.md`).

mod common;
use common::git::git;
use common::memrepo::{commit_doc, doc, write_doc};
use common::{McpClient, Person, Spawn, World};
use serde_json::{json, Value};
use std::path::Path;

fn client(w: &World) -> McpClient {
    McpClient::for_person(w, Person::A, Spawn::default())
}

fn doc_with(title: &str, created: &str, author: &str, body: &str) -> String {
    format!(
        "---\ntitle: \"{title}\"\ndescription: About {title}.\nstatus: active\ncreated: {created}\nupdated: {created}\ncreated_by: \"{author}\"\nupdated_by: \"{author}\"\n---\n\n{body}\n"
    )
}

const ANNA: &str = "Anna Example <anna@example.org>";

fn commit_text(dir: &Path, id: &str, text: &str) {
    write_doc(dir, id, text);
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", &format!("test: {id}")]);
    git(dir, &["push", "-q"]);
}

fn set_shared(w: &World, from: &str, to: &str) {
    let path = w.a.join("agent-memory/.waymark/settings.toml");
    let old = std::fs::read_to_string(&path).unwrap();
    assert!(old.contains(from), "{from}");
    std::fs::write(&path, old.replacen(from, to, 1)).unwrap();
    git(&w.a, &["add", "-A"]);
    git(&w.a, &["commit", "-q", "-m", "test: settings"]);
    git(&w.a, &["push", "-q"]);
}

fn rules(r: &Value, rule: &str) -> Vec<Value> {
    r["findings"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| f["rule"] == rule)
        .cloned()
        .collect()
}

#[test]
fn t1_a_broken_body_link_is_a_warning() {
    let w = World::new();
    commit_text(
        &w.a,
        "ops/a",
        &doc_with("A", "2026-11-01", ANNA, "See [x](missing-file.md)."),
    );
    let mut c = client(&w);
    let r = c.call("lint", json!({}));
    assert!(!r.is_error, "{}", r.json);
    let f = rules(&r.json, "broken-link");
    assert_eq!(f.len(), 1, "{}", r.json);
    assert_eq!(f[0]["severity"], "warning");
    assert_eq!(f[0]["path"], "ops/a.md");
    assert!(f[0]["message"].as_str().unwrap().contains("missing-file"));
}

#[test]
fn t2_a_broken_property_link_is_an_error() {
    let w = World::new();
    let text = doc_with("A", "2026-11-01", ANNA, "x")
        .replace("---\n\nx", "related:\n  - \"[[ops/missing]]\"\n---\n\nx");
    commit_text(&w.a, "ops/a", &text);
    let mut c = client(&w);
    let r = c.call("lint", json!({}));
    let f = rules(&r.json, "broken-link");
    assert_eq!(f.len(), 1, "{}", r.json);
    assert_eq!(f[0]["severity"], "error");
    assert!(r.json["errors"].as_u64().unwrap() >= 1);
}

#[test]
fn t3_a_custom_pattern_blocks_a_write_and_lint_reports_it() {
    let w = World::new();
    commit_doc(
        &w.a,
        "ops/a",
        "Has customer id",
        "Customer CUST-123456 here.",
    );
    set_shared(
        &w,
        "# extra_secret_patterns = []",
        "extra_secret_patterns = [\"CUST-[0-9]{6}\"]",
    );
    let mut c = client(&w);
    let add = c.call(
        "add_document",
        json!({"folder": "ops", "title": "Another", "description": "D.", "body": "CUST-654321"}),
    );
    assert_eq!(add.code(), Some("validation_failed"));
    assert!(!add.json.to_string().contains("CUST-654321"));
    let r = c.call("lint", json!({}));
    assert_eq!(rules(&r.json, "secret").len(), 1, "{}", r.json);
}

#[test]
fn t4_a_long_title_is_reported() {
    let w = World::new();
    let title = "one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen";
    commit_text(&w.a, "ops/long", &doc_with(title, "2026-11-01", ANNA, "x"));
    let mut c = client(&w);
    let r = c.call("lint", json!({}));
    assert_eq!(rules(&r.json, "title-too-long").len(), 1, "{}", r.json);
}

#[test]
fn t5_an_author_that_differs_from_git_history_is_a_warning() {
    let w = World::new();
    // Committed by "Test Setup", but the file names another author.
    commit_text(
        &w.a,
        "ops/a",
        &doc_with("A", "2026-11-01", "Other Person <other@example.org>", "x"),
    );
    // Not committed: no history, no check.
    write_doc(
        &w.a,
        "ops/b",
        &doc_with("B", "2026-11-01", "Other Person <other@example.org>", "x"),
    );
    let mut c = client(&w);
    let r = c.call("lint", json!({}));
    let f = rules(&r.json, "author-mismatch");
    assert!(
        f.iter()
            .all(|x| x["severity"] == "warning" && x["path"] == "ops/a.md"),
        "{}",
        r.json
    );
    assert_eq!(f.len(), 2, "created_by and updated_by: {}", r.json);
}

#[test]
fn t6_the_sample_vault_has_no_findings() {
    let w = World::new();
    let vault = Path::new(env!("CARGO_MANIFEST_DIR")).join("standard/conformance/vault");
    let status = std::process::Command::new("cp")
        .args([
            "-R",
            &format!("{}/.", vault.display()),
            w.a.join("agent-memory").to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success());
    // Keep the settings of the world, and the exclude list of the vault (it excludes the README).
    let exclude = std::fs::read_to_string(vault.join(".waymark/exclude")).unwrap();
    std::fs::write(w.a.join("agent-memory/.waymark/exclude"), exclude).unwrap();
    let mut c = client(&w);
    let r = c.call("lint", json!({}));
    assert_eq!(r.json["errors"], 0, "{}", r.json);
    assert_eq!(r.json["findings"], json!([]), "{}", r.json);
}

#[test]
fn t7_callouts_embeds_highlights_and_comments_are_accepted() {
    let w = World::new();
    let body = "> [!note] A callout\n> Text.\n\n![[some-image.png]]\n\nA ==highlight== and %%a comment%% here.";
    write_doc(
        &w.a,
        "ops/rich",
        &doc_with("Rich", "2026-11-01", ANNA, body),
    );
    let mut c = client(&w);
    let r = c.call("lint", json!({}));
    assert_eq!(r.json["findings"], json!([]), "{}", r.json);
}

#[test]
fn t8_a_private_key_blocks_a_write_and_is_never_repeated() {
    let w = World::new();
    let mut c = client(&w);
    let r = c.call(
        "add_document",
        json!({"folder": "ops", "title": "Keys", "description": "D.",
            "body": "-----BEGIN RSA PRIVATE KEY-----\nMIIEvQSECRETKEYTEXT\n-----END RSA PRIVATE KEY-----"}),
    );
    assert_eq!(r.code(), Some("validation_failed"));
    assert!(!r.json.to_string().contains("SECRETKEYTEXT"));
    assert!(r.message().contains("private key"));
}

#[test]
fn t9_the_newer_of_two_equal_titles_has_the_finding() {
    let w = World::new();
    commit_text(
        &w.a,
        "ops/older",
        &doc_with("How we deploy", "2026-10-01", ANNA, "x"),
    );
    commit_text(
        &w.a,
        "ops/newer",
        &doc_with("how we DEPLOY", "2026-11-01", ANNA, "x"),
    );
    let mut c = client(&w);
    let r = c.call("lint", json!({}));
    let f = rules(&r.json, "duplicate-title");
    assert_eq!(f.len(), 1, "{}", r.json);
    assert_eq!(f[0]["path"], "ops/newer.md");
    assert!(f[0]["message"].as_str().unwrap().contains("ops/older"));
}

#[test]
fn t10_an_iban_is_financial_data_and_a_wrong_one_is_not() {
    let w = World::new();
    commit_text(
        &w.a,
        "ops/iban",
        &doc_with(
            "Bank",
            "2026-11-01",
            ANNA,
            "Pay to DE89 3704 0044 0532 0130 00.",
        ),
    );
    commit_text(
        &w.a,
        "ops/fake",
        &doc_with(
            "Fake bank",
            "2026-11-01",
            ANNA,
            "Pay to DE88 3704 0044 0532 0130 00.",
        ),
    );
    let mut c = client(&w);
    let r = c.call("lint", json!({}));
    let f = rules(&r.json, "financial-data");
    assert_eq!(f.len(), 1, "{}", r.json);
    assert_eq!(f[0]["path"], "ops/iban.md");
    assert_eq!(f[0]["severity"], "warning");
}

#[test]
fn t12_an_author_from_another_domain_is_an_error() {
    let w = World::new();
    set_shared(
        &w,
        "[documents]",
        "[people]\nemail_domains = [\"example.org\"]\n\n[documents]",
    );
    commit_text(
        &w.a,
        "ops/a",
        &doc_with("A", "2026-11-01", "Jo <jo@other.test>", "x"),
    );
    let mut c = client(&w);
    let r = c.call("lint", json!({}));
    let f = rules(&r.json, "author-not-allowed");
    assert!(!f.is_empty(), "{}", r.json);
    assert!(f.iter().all(|x| x["severity"] == "error"));
}

#[test]
fn t17_there_is_no_index_tool_and_no_stale_check() {
    let w = World::new();
    let mut c = client(&w);
    assert!(!c.tool_names().contains(&"index".to_string()));
    assert!(c.call_raw("index", json!({})).get("error").is_some());
    // An `index.md` with the tag `index` is a normal document: lint never reports `index-stale`.
    commit_doc(&w.a, "backend/api", "The api", "x");
    let r = c.call("lint", json!({}));
    assert!(rules(&r.json, "index-stale").is_empty(), "{}", r.json);
    drop(c);
    let mut c = McpClient::for_person(&w, Person::A, Spawn::args(&["--access", "read-only"]));
    let names = c.tool_names();
    assert!(names.contains(&"lint".to_string()));
    assert!(!names.contains(&"index".to_string()));
}

#[test]
fn t20_a_placeholder_is_not_a_secret_but_a_value_is() {
    let w = World::new();
    commit_text(
        &w.a,
        "ops/ok",
        &doc_with(
            "Placeholder",
            "2026-11-01",
            ANNA,
            "Set `password = <password>` first.",
        ),
    );
    commit_text(
        &w.a,
        "ops/bad",
        &doc_with(
            "Real value",
            "2026-11-01",
            ANNA,
            "Set password = hunter2hunter2 here.",
        ),
    );
    let mut c = client(&w);
    let r = c.call("lint", json!({}));
    let f = rules(&r.json, "secret");
    assert_eq!(f.len(), 1, "{}", r.json);
    assert_eq!(f[0]["path"], "ops/bad.md");
    assert!(!r.json.to_string().contains("hunter2"));
}

#[test]
fn t21_a_bad_extra_pattern_is_a_setup_problem() {
    let w = World::new();
    set_shared(
        &w,
        "# extra_secret_patterns = []",
        "extra_secret_patterns = [\"[\"]",
    );
    let mut c = client(&w);
    let r = c.call("lint", json!({}));
    assert_eq!(r.code(), Some("setup_problem"));
    assert!(r.json["next"].as_str().unwrap().contains("settings.toml"));
    assert!(r.message().contains('['));
}

#[test]
fn t22_the_result_has_at_most_100_findings_errors_first() {
    let w = World::new();
    for i in 0..150 {
        write_doc(&w.a, &format!("bad/b{i:03}"), "no frontmatter here\n");
    }
    write_doc(
        &w.a,
        "aaa/warn",
        &doc_with("Warn", "2026-11-01", ANNA, "[x](gone.md)"),
    );
    let mut c = client(&w);
    let r = c.call("lint", json!({}));
    let findings = r.json["findings"].as_array().unwrap();
    assert_eq!(findings.len(), 100);
    assert_eq!(r.json["truncated"], true);
    assert!(findings.iter().all(|f| f["severity"] == "error"));
    assert!(r.json["errors"].as_u64().unwrap() >= 150);
    assert_eq!(r.json["warnings"], 1);
}

#[test]
fn t23_the_folders_argument_narrows_lint() {
    let w = World::new();
    write_doc(&w.a, "ops/a", "no frontmatter\n");
    write_doc(&w.a, "dev/a", "no frontmatter\n");
    let mut c = client(&w);
    let r = c.call("lint", json!({"folders": ["ops"]}));
    assert_eq!(r.json["checked"], 1);
    assert!(r.json["findings"]
        .as_array()
        .unwrap()
        .iter()
        .all(|f| f["path"] == "ops/a.md"));
}

#[test]
fn t24_a_secret_in_a_code_block_is_a_finding() {
    let w = World::new();
    commit_text(
        &w.a,
        "ops/code",
        &doc_with(
            "Code",
            "2026-11-01",
            ANNA,
            "```\nexport API_KEY=abcd1234efgh5678\n```",
        ),
    );
    let mut c = client(&w);
    let r = c.call("lint", json!({}));
    assert_eq!(rules(&r.json, "secret").len(), 1, "{}", r.json);
}

#[test]
fn t25_an_iban_does_not_block_a_write_but_a_secret_does() {
    let w = World::new();
    let mut c = client(&w);
    let r = c.call(
        "add_document",
        json!({"folder": "ops", "title": "Bank details", "description": "D.",
            "body": "Pay to DE89 3704 0044 0532 0130 00."}),
    );
    assert!(!r.is_error, "{}", r.json);
    let notices = r.json["notices"].to_string();
    assert!(
        notices.contains("IBAN") && notices.contains("line 1"),
        "{}",
        r.json
    );
    let r = c.call(
        "add_document",
        json!({"folder": "ops", "title": "Keys", "description": "D.",
            "body": "Pay to DE89 3704 0044 0532 0130 00.\n-----BEGIN RSA PRIVATE KEY-----\nMIIEvQSECRETKEYTEXT\n-----END RSA PRIVATE KEY-----"}),
    );
    assert_eq!(r.code(), Some("validation_failed"));
    assert!(!r.json.to_string().contains("SECRETKEYTEXT"));
}

#[test]
fn a_clean_memory_repository_has_no_findings() {
    let w = World::new();
    let mut c = client(&w);
    let r = c.call("lint", json!({}));
    assert_eq!(r.json["findings"], json!([]));
    assert_eq!(r.json["errors"], 0);
    let _ = doc("x", "y");
}
