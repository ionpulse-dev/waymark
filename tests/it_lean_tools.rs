//! End-to-end tests of the lean tools (see `specs/features/lean-tools/test.md`).

mod common;
use common::git::git;
use common::memrepo::commit_doc;
use common::{McpClient, Person, Spawn, World};
use serde_json::{json, Value};
use std::path::Path;

fn client(w: &World) -> McpClient {
    McpClient::for_person(
        w,
        Person::A,
        Spawn {
            env: vec![("WAYMARK_FETCH_MAX_AGE".into(), "0".into())],
            ..Spawn::default()
        },
    )
}

fn personal(w: &World, text: &str) {
    let p = w.root.join("home-a/.config/waymark/config.toml");
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, text).unwrap();
}

fn add_with(c: &mut McpClient, folder: &str, title: &str, body: &str, tags: &[&str]) -> String {
    let r = c.call(
        "add_document",
        json!({"folder": folder, "title": title, "description": format!("About {title}."), "body": body, "tags": tags}),
    );
    assert!(!r.is_error, "{}", r.json);
    r.json["id"].as_str().unwrap().to_string()
}

fn version(c: &mut McpClient, id: &str) -> String {
    let r = c.call("show", json!({"id": id}));
    assert!(!r.is_error, "{}", r.json);
    r.json["version"].as_str().unwrap().to_string()
}

fn file(dir: &Path, id: &str) -> String {
    std::fs::read_to_string(dir.join("agent-memory").join(format!("{id}.md"))).unwrap()
}

fn ids(v: &Value) -> Vec<String> {
    v["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["id"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn list_is_gone_and_search_needs_no_query() {
    let w = World::new();
    let mut c = client(&w);
    let tools = c.list_tools();
    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(!names.contains(&"list"), "{names:?}");
    let search = tools.iter().find(|t| t["name"] == "search").unwrap();
    let required = search["inputSchema"]["required"].as_array();
    assert!(
        required.is_none_or(|r| r.is_empty()),
        "search still requires {required:?}"
    );
}

#[test]
fn search_without_a_query_lists_by_id() {
    let w = World::new();
    let mut c = client(&w);
    add_with(&mut c, "ops", "Zeta process", "x", &[]);
    add_with(&mut c, "ops", "Alpha process", "x", &[]);
    add_with(&mut c, "dev", "Other area", "x", &[]);
    let r = c.call("search", json!({"folders": ["ops"]}));
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(ids(&r.json), ["ops/alpha-process", "ops/zeta-process"]);
    assert_eq!(r.json["total_matches"], 2);
    // Level 1 by default: a title, and no description or snippet.
    let first = &r.json["results"][0];
    assert_eq!(first["title"], "Alpha process");
    assert!(first.get("description").is_none() && first.get("snippet").is_none());
    assert!(r.message().contains("2 of 2 documents"), "{}", r.message());
    // Filters and paging work as with a query.
    let r = c.call("search", json!({"folders": ["ops"], "limit": 1}));
    assert_eq!(r.json["truncated"], true);
    assert_eq!(r.json["next_offset"], 1);
    let r = c.call("search", json!({"folders": ["ops"], "offset": 1}));
    assert_eq!(ids(&r.json), ["ops/zeta-process"]);
    // A query without words is still a mistake.
    let r = c.call("search", json!({"query": "  "}));
    assert_eq!(r.code(), Some("invalid_arguments"));
}

#[test]
fn search_without_a_query_hides_deprecated_documents() {
    let w = World::new();
    let mut c = client(&w);
    let id = add_with(&mut c, "ops", "Old thing", "x", &[]);
    let base = version(&mut c, &id);
    let r = c.call(
        "deprecate_document",
        json!({"id": id, "base": base, "reason": "Old."}),
    );
    assert!(!r.is_error, "{}", r.json);
    add_with(&mut c, "ops", "New thing", "x", &[]);
    let r = c.call("search", json!({"folders": ["ops"]}));
    assert_eq!(ids(&r.json), ["ops/new-thing"]);
    let r = c.call("search", json!({"folders": ["ops"], "status": "any"}));
    assert_eq!(ids(&r.json), ["ops/new-thing", "ops/old-thing"]);
}

#[test]
fn the_old_name_list_is_an_unknown_tool() {
    // The call alias of 0.5.x ended in 0.6.0 (see `decisions.md`).
    let w = World::new();
    let mut c = client(&w);
    assert!(c.call_raw("list", json!({}))["error"].is_object());
    assert!(c.call_raw("list", json!({"folders": ["ops"]}))["error"].is_object());
}

#[test]
fn list_in_a_limit_switches_off_search() {
    let w = World::new();
    personal(&w, "[limits]\ntools_denied = [\"list\"]\n");
    let mut c = client(&w);
    assert!(!c.tool_names().contains(&"search".to_string()));
    let r = c.call("search", json!({"query": "x"}));
    assert_eq!(r.code(), Some("access_denied"));
}

#[test]
fn the_old_names_of_0_4_0_are_unknown_tools() {
    let w = World::new();
    let mut c = client(&w);
    for old in [
        "add",
        "delete",
        "deprecate",
        "dispute",
        "verify",
        "reject",
        "add_empty_folder",
        "remove_folder",
        "folders",
    ] {
        assert!(
            c.call_raw(old, json!({}))["error"].is_object(),
            "{old} still works"
        );
    }
}

const BODY: &str = "Alpha beta gamma.\n\nDelta beta.\n\nOmega.";

fn doc_with_body(c: &mut McpClient) -> (String, String) {
    let id = add_with(c, "ops", "Editable", BODY, &["one", "two"]);
    let base = version(c, &id);
    (id, base)
}

#[test]
fn replace_changes_one_passage() {
    let w = World::new();
    let mut c = client(&w);
    let (id, base) = doc_with_body(&mut c);
    let r = c.call(
        "update_document",
        json!({"id": id, "base": base, "replace": {"old": "Alpha beta", "new": "ALPHA BETA"}}),
    );
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(r.json["changed"], json!(["body"]));
    let text = file(&w.a, &id);
    assert!(
        text.ends_with("\nALPHA BETA gamma.\n\nDelta beta.\n\nOmega.\n"),
        "{text}"
    );
    // An empty `new` removes the passage.
    let base = version(&mut c, &id);
    let r = c.call(
        "update_document",
        json!({"id": id, "base": base, "replace": {"old": "\n\nOmega.", "new": ""}}),
    );
    assert!(!r.is_error, "{}", r.json);
    assert!(
        file(&w.a, &id).ends_with("\nALPHA BETA gamma.\n\nDelta beta.\n"),
        "{}",
        file(&w.a, &id)
    );
}

#[test]
fn replace_needs_exactly_one_match() {
    let w = World::new();
    let mut c = client(&w);
    let (id, base) = doc_with_body(&mut c);
    let before = file(&w.a, &id);
    for (old, expect) in [("missing passage", "does not occur"), ("beta", "2 times")] {
        let r = c.call(
            "update_document",
            json!({"id": id, "base": base, "replace": {"old": old, "new": "x"}}),
        );
        assert_eq!(r.code(), Some("validation_failed"), "{}", r.json);
        assert!(r.message().contains(expect), "{}", r.message());
        assert!(r.json["next"].is_string(), "{}", r.json);
    }
    let r = c.call(
        "update_document",
        json!({"id": id, "base": base, "replace": {"old": "", "new": "x"}}),
    );
    assert_eq!(r.code(), Some("invalid_arguments"));
    assert_eq!(file(&w.a, &id), before);
}

#[test]
fn append_adds_text_at_the_end() {
    let w = World::new();
    let mut c = client(&w);
    let (id, base) = doc_with_body(&mut c);
    let r = c.call(
        "update_document",
        json!({"id": id, "base": base, "append": "\nAppended paragraph."}),
    );
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(r.json["changed"], json!(["body"]));
    let text = file(&w.a, &id);
    assert!(text.ends_with("Omega.\n\nAppended paragraph.\n"), "{text}");
    assert!(text.contains("Alpha beta gamma.\n\nDelta beta."), "{text}");
}

#[test]
fn conflicting_body_and_tag_parameters_fail() {
    let w = World::new();
    let mut c = client(&w);
    let (id, base) = doc_with_body(&mut c);
    let before = file(&w.a, &id);
    for args in [
        json!({"body": "x", "append": "y"}),
        json!({"body": "x", "replace": {"old": "Omega", "new": "y"}}),
        json!({"append": "x", "replace": {"old": "Omega", "new": "y"}}),
        json!({"tags": ["a"], "add_tags": ["b"]}),
        json!({"tags": ["a"], "remove_tags": ["b"]}),
        json!({"add_tags": ["same"], "remove_tags": ["same"]}),
        json!({"add_tags": ["has space"]}),
    ] {
        let mut full = args.clone();
        full["id"] = json!(id);
        full["base"] = json!(base);
        let r = c.call("update_document", full);
        assert_eq!(r.code(), Some("invalid_arguments"), "{args}: {}", r.json);
        assert!(r.json["next"].is_string(), "{}", r.json);
    }
    assert_eq!(file(&w.a, &id), before);
}

#[test]
fn add_and_remove_tags() {
    let w = World::new();
    let mut c = client(&w);
    let (id, base) = doc_with_body(&mut c);
    let before = w.remote_commits();
    let r = c.call(
        "update_document",
        json!({"id": id, "base": base, "add_tags": ["three", "one"], "remove_tags": ["two", "absent"]}),
    );
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(r.json["changed"], json!(["tags"]));
    let shown = c.call("show", json!({"id": id}));
    assert_eq!(shown.json["tags"], json!(["one", "three"]));
    assert_eq!(w.remote_commits(), before + 1);
    // Nothing to change: no commit.
    let base = version(&mut c, &id);
    let r = c.call(
        "update_document",
        json!({"id": id, "base": base, "add_tags": ["one"], "remove_tags": ["absent"]}),
    );
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(r.message(), "No change.");
    assert_eq!(w.remote_commits(), before + 1);
    // Removing the last tags removes the property.
    let r = c.call(
        "update_document",
        json!({"id": id, "base": base, "remove_tags": ["one", "three"]}),
    );
    assert!(!r.is_error, "{}", r.json);
    assert!(!file(&w.a, &id).contains("tags:"), "{}", file(&w.a, &id));
}

trait Commits {
    fn remote_commits(&self) -> usize;
}

impl Commits for World {
    fn remote_commits(&self) -> usize {
        common::git::subjects(&self.remote).len()
    }
}

#[test]
fn replace_resets_trust_and_checks_links() {
    let w = World::new();
    let path = w.a.join("agent-memory/.waymark/settings.toml");
    let old = std::fs::read_to_string(&path).unwrap();
    std::fs::write(
        &path,
        old.replacen("verification = false", "verification = true", 1),
    )
    .unwrap();
    git(&w.a, &["add", "-A"]);
    git(&w.a, &["commit", "-q", "-m", "test: settings"]);
    git(&w.a, &["push", "-q"]);
    let mut c = client(&w);
    let (id, base) = doc_with_body(&mut c);
    let r = c.call("verify_document", json!({"id": id, "base": base}));
    assert!(!r.is_error, "{}", r.json);
    let base = version(&mut c, &id);
    assert!(file(&w.a, &id).contains("trust: verified"));
    // A new body link to a missing document fails.
    let r = c.call(
        "update_document",
        json!({"id": id, "base": base, "append": "See [gone](ops/gone.md)."}),
    );
    assert_eq!(r.code(), Some("validation_failed"), "{}", r.json);
    assert!(file(&w.a, &id).contains("trust: verified"));
    // A change of the body resets trust.
    let r = c.call(
        "update_document",
        json!({"id": id, "base": base, "replace": {"old": "Omega.", "new": "Omega!"}}),
    );
    assert!(!r.is_error, "{}", r.json);
    let text = file(&w.a, &id);
    assert!(
        text.contains("trust: unverified") && !text.contains("verified_by"),
        "{text}"
    );
}

#[test]
fn show_returns_the_deprecation_reason() {
    let w = World::new();
    let mut c = client(&w);
    let id = add_with(&mut c, "ops", "Retired", "x", &[]);
    // An active document has no such field.
    let r = c.call("show", json!({"id": id}));
    assert!(r.json.get("deprecation_reason").is_none(), "{}", r.json);
    let base = r.json["version"].as_str().unwrap().to_string();
    let r = c.call(
        "deprecate_document",
        json!({"id": id, "base": base, "reason": "The API was removed."}),
    );
    assert!(!r.is_error, "{}", r.json);
    let r = c.call("show", json!({"id": id}));
    assert_eq!(r.json["deprecation_reason"], "The API was removed.");
    // An older version has no such field.
    let history = c.call("history", json!({"id": id}));
    let oldest = history.json["entries"].as_array().unwrap().last().unwrap()["version"]
        .as_str()
        .unwrap()
        .to_string();
    let r = c.call("show", json!({"id": id, "version": oldest}));
    assert!(r.json.get("deprecation_reason").is_none(), "{}", r.json);
}

#[test]
fn a_restored_document_has_no_deprecation_reason() {
    let w = World::new();
    let mut c = client(&w);
    let id = add_with(&mut c, "ops", "Back again", "x", &[]);
    let base = version(&mut c, &id);
    c.call(
        "deprecate_document",
        json!({"id": id, "base": base, "reason": "Wrong."}),
    );
    let base = version(&mut c, &id);
    let r = c.call("restore_document", json!({"id": id, "base": base}));
    assert!(!r.is_error, "{}", r.json);
    let r = c.call("show", json!({"id": id}));
    assert_eq!(r.json["status"], "active");
    assert!(r.json.get("deprecation_reason").is_none(), "{}", r.json);
    // Deprecated again with another reason: the newest reason counts.
    let base = r.json["version"].as_str().unwrap().to_string();
    c.call(
        "deprecate_document",
        json!({"id": id, "base": base, "reason": "Wrong again."}),
    );
    let r = c.call("show", json!({"id": id}));
    assert_eq!(r.json["deprecation_reason"], "Wrong again.");
}

#[test]
fn a_deprecation_without_a_reason_gives_null() {
    let w = World::new();
    let mut c = client(&w);
    let old = add_with(&mut c, "ops", "Old way", "x", &[]);
    let r = c.call(
        "add_document",
        json!({"folder": "ops", "title": "New way", "description": "The new way.", "body": "y", "supersedes": old}),
    );
    assert!(!r.is_error, "{}", r.json);
    let r = c.call("show", json!({"id": old}));
    assert_eq!(r.json["status"], "deprecated");
    assert!(r.json["deprecation_reason"].is_null(), "{}", r.json);
    assert!(r
        .json
        .as_object()
        .unwrap()
        .contains_key("deprecation_reason"));
}

#[test]
fn updates_pages_with_offset() {
    let w = World::new();
    for i in 0..55 {
        commit_doc(
            &w.b,
            &format!("ops/doc-{i:02}"),
            &format!("Document {i:02}"),
            "x",
        );
    }
    let mut c = client(&w);
    let first = c.call("updates", json!({"since": "2020-01-01"}));
    assert!(!first.is_error, "{}", first.json);
    let total = first.json["total"].as_u64().unwrap();
    assert!(total >= 55, "{total}");
    assert_eq!(first.json["truncated"], true);
    assert_eq!(first.json["changes"].as_array().unwrap().len(), 50);
    assert_eq!(first.json["next_offset"], 50);
    assert!(first.json["until"].is_string());
    let second = c.call("updates", json!({"since": "2020-01-01", "offset": 50}));
    assert!(!second.is_error, "{}", second.json);
    assert_eq!(second.json["truncated"], false);
    assert!(second.json.get("next_offset").is_none());
    let mut all: Vec<String> = Vec::new();
    for page in [&first.json, &second.json] {
        for ch in page["changes"].as_array().unwrap() {
            all.push(ch["id"].as_str().unwrap().to_string());
        }
    }
    assert_eq!(all.len() as u64, total);
    let mut unique = all.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), all.len(), "an entry was repeated");
    // A call with `since` does not move the stored time.
    assert_eq!(second.json["marked"], false);
}

#[test]
fn updates_offset_needs_since() {
    let w = World::new();
    let mut c = client(&w);
    let r = c.call("updates", json!({"offset": 5}));
    assert_eq!(r.code(), Some("invalid_arguments"), "{}", r.json);
    assert!(r.json["next"].is_string());
    let r = c.call("updates", json!({"since": "2020-01-01", "offset": -1}));
    assert_eq!(r.code(), Some("invalid_arguments"));
}

/// The text of every string literal in a Rust source file.
fn literals(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut inside = false;
    let mut escaped = false;
    for ch in src.chars() {
        if inside {
            if escaped {
                cur.push(ch);
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                inside = false;
                out.push(std::mem::take(&mut cur));
            } else {
                cur.push(ch);
            }
        } else if ch == '"' {
            inside = true;
        }
    }
    out
}

fn rust_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            rust_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

#[test]
fn no_hint_names_a_retired_tool() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_files(&root, &mut files);
    assert!(files.len() > 50, "found only {} files", files.len());
    let retired = [
        "add",
        "delete",
        "deprecate",
        "dispute",
        "verify",
        "reject",
        "add_empty_folder",
        "remove_folder",
        "folders",
        "list",
    ];
    // The tools of this build, and the parameters whose names look like tool names.
    let real = [
        "add_document",
        "update_document",
        "delete_document",
        "deprecate_document",
        "restore_document",
        "dispute_document",
        "verify_document",
        "reject_document",
        "create_folder",
        "delete_folder",
        "folder_tree",
        "list_tags",
        "rename_tag",
        "add_tags",
        "remove_tags",
    ];
    let mut bad = Vec::new();
    for f in &files {
        let src = std::fs::read_to_string(f).unwrap();
        for lit in literals(&src) {
            for old in retired {
                for form in [
                    format!("call {old} "),
                    format!("call {old}."),
                    format!("use {old} "),
                    format!("use {old}."),
                    format!("with {old} "),
                    format!("with {old}."),
                    format!("{old} and clear"),
                    format!("or {old} to"),
                    format!("run {old}."),
                    format!("run {old} "),
                    format!("the tool \"{old}\" "),
                ] {
                    if lit.to_lowercase().contains(&form)
                        && !lit.contains("is now")
                        && !f.ends_with("tests.rs")
                    {
                        bad.push(format!("{}: \"{lit}\" has \"{form}\"", f.display()));
                    }
                }
            }
            // A word that looks like a tool of one document or folder must be a real tool.
            for word in lit.split(|c: char| !(c.is_ascii_lowercase() || c == '_')) {
                // The table of old names in limits lists the retired names on purpose.
                if f.ends_with("config/limits.rs") || word == "total_tags" {
                    continue;
                }
                let looks = word.ends_with("_document")
                    || word.ends_with("_folder")
                    || word.ends_with("_tags")
                    || word.ends_with("_tag");
                if looks && !real.contains(&word) {
                    bad.push(format!("{}: \"{lit}\" names \"{word}\"", f.display()));
                }
            }
        }
    }
    assert!(bad.is_empty(), "stale tool names:\n{}", bad.join("\n"));
}
