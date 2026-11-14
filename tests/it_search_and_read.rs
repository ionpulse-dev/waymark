//! End-to-end tests of the read tools (see `specs/features/search-and-read/test.md`).

mod common;
use common::git::git;
use common::kb::{commit_doc, doc, write_doc};
use common::{McpClient, Person, Spawn, World};
use serde_json::{json, Value};
use std::path::Path;
use std::time::Instant;

fn client(w: &World) -> McpClient {
    McpClient::for_person(w, Person::A, Spawn::default())
}

fn ids(r: &Value) -> Vec<String> {
    r["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["id"].as_str().unwrap().to_string())
        .collect()
}

fn commit_text(dir: &Path, id: &str, text: &str) {
    write_doc(dir, id, text);
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", &format!("test: {id}")]);
    git(dir, &["push", "-q"]);
}

fn with_props(title: &str, extra_props: &str, body: &str) -> String {
    format!(
        "---\ntitle: \"{title}\"\ndescription: About {title}.\nstatus: active\ncreated: 2026-11-01\nupdated: 2026-11-01\ncreated_by: \"Anna Example <anna@example.org>\"\nupdated_by: \"Anna Example <anna@example.org>\"\n{extra_props}---\n\n{body}\n"
    )
}

fn set_shared(w: &World, from: &str, to: &str) {
    let path = w.a.join("knowledge/.mkbase/waymark.toml");
    let old = std::fs::read_to_string(&path).unwrap();
    assert!(old.contains(from));
    std::fs::write(&path, old.replacen(from, to, 1)).unwrap();
    git(&w.a, &["add", "-A"]);
    git(&w.a, &["commit", "-q", "-m", "test: settings"]);
    git(&w.a, &["push", "-q"]);
}

#[test]
fn t1_no_match_is_an_empty_result() {
    let w = World::new();
    commit_doc(&w.a, "ops/deploy", "How we deploy", "Use the pipeline.");
    let mut c = client(&w);
    let r = c.call("search", json!({"query": "zebra"}));
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(r.json["results"], json!([]));
    assert_eq!(r.json["total_matches"], 0);
}

#[test]
fn t2_t3_the_result_is_capped_by_count_and_tokens() {
    let w = World::new();
    for i in 0..8 {
        commit_doc(
            &w.a,
            &format!("ops/doc-{i}"),
            &format!("Keys document {i}"),
            "keys",
        );
    }
    let mut c = client(&w);
    let r = c.call("search", json!({"query": "keys"}));
    assert_eq!(r.json["results"].as_array().unwrap().len(), 5);
    assert_eq!(r.json["truncated"], true);
    assert_eq!(r.json["total_matches"], 8);

    // Long descriptions: stay within 800 estimated tokens (the first entry always stays).
    let w = World::new();
    for i in 0..6 {
        let long = "word ".repeat(40);
        let text = format!(
            "---\ntitle: Long {i}\ndescription: {long}\nstatus: active\ncreated: 2026-11-01\nupdated: 2026-11-01\ncreated_by: \"A <a@example.org>\"\nupdated_by: \"A <a@example.org>\"\n---\n\nneedle\n"
        );
        commit_text(&w.a, &format!("ops/long-{i}"), &text);
    }
    let mut c = client(&w);
    let r = c.call("search", json!({"query": "needle", "level": 2}));
    let words: usize = r.json["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            e.as_object()
                .unwrap()
                .values()
                .filter_map(Value::as_str)
                .map(|s| s.split_whitespace().count())
                .sum::<usize>()
        })
        .sum();
    assert!(words * 3 / 2 <= 800 + 100, "{words}");
    assert!(r.json["results"].as_array().unwrap().len() < 6);
    assert_eq!(r.json["truncated"], true);
}

#[test]
fn t4_a_larger_limit_does_not_raise_the_cap() {
    let w = World::new();
    for i in 0..8 {
        commit_doc(
            &w.a,
            &format!("ops/doc-{i}"),
            &format!("Keys document {i}"),
            "keys",
        );
    }
    let mut c = client(&w);
    let r = c.call("search", json!({"query": "keys", "limit": 50}));
    assert_eq!(r.json["results"].as_array().unwrap().len(), 5);
    let p = w.root.join("home-a/.config/waymark/config.toml");
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, "[limits]\nmax_results = 3\n").unwrap();
    let mut c = client(&w);
    let r = c.call("search", json!({"query": "keys", "limit": 50}));
    assert_eq!(r.json["results"].as_array().unwrap().len(), 3);
}

#[test]
fn t5_a_title_match_ranks_before_a_body_match() {
    let w = World::new();
    commit_doc(
        &w.a,
        "ops/aaa-body",
        "Something else",
        "idempotency idempotency idempotency",
    );
    commit_doc(&w.a, "ops/zzz-title", "Idempotency keys", "text");
    let mut c = client(&w);
    let r = c.call("search", json!({"query": "idempotency"}));
    assert_eq!(ids(&r.json), vec!["ops/zzz-title", "ops/aaa-body"]);
    assert!(r.json["results"][0]["snippet"].is_string());
}

#[test]
fn t6_t7_the_read_budget() {
    let w = World::new();
    for n in ["a", "b", "c"] {
        commit_doc(&w.a, &format!("ops/{n}"), &format!("Doc {n}"), "x");
    }
    let mut c = McpClient::for_person(&w, Person::A, Spawn::args(&["--max-documents-opened", "2"]));
    assert!(!c.call("show", json!({"id": "ops/a"})).is_error);
    assert!(!c.call("show", json!({"id": "ops/b"})).is_error);
    let r = c.call("show", json!({"id": "ops/c"}));
    assert_eq!(r.code(), Some("access_denied"));
    assert!(r.message().contains("budget"));
    // A document that was opened already costs nothing.
    assert!(!c.call("show", json!({"id": "ops/a"})).is_error);
}

#[test]
fn t8_level_0_is_only_the_id() {
    let w = World::new();
    commit_doc(&w.a, "ops/a", "Doc a", "x");
    let mut c = client(&w);
    let r = c.call("list", json!({"level": 0}));
    assert_eq!(r.json["results"], json!([{"id": "ops/a"}]));
    let r = c.call("list", json!({}));
    assert_eq!(r.json["results"][0]["title"], "Doc a");
    assert!(r.json["results"][0].get("description").is_none());
}

#[test]
fn t9_t10_t11_backlinks_dispute_and_replacement() {
    let w = World::new();
    commit_text(&w.a, "ops/target", &with_props("Target", "", "x"));
    commit_text(
        &w.a,
        "ops/body-link",
        &with_props("Body link", "", "See [[ops/target]]."),
    );
    commit_text(
        &w.a,
        "ops/prop-link",
        &with_props("Prop link", "related:\n  - \"[[ops/target]]\"\n", "x"),
    );
    commit_text(
        &w.a,
        "ops/disputed",
        &with_props(
            "Disputed one",
            "disputed: true\ndispute_reason: The port is wrong.\n",
            "x",
        ),
    );
    commit_text(
        &w.a,
        "ops/old",
        &with_props("Old way", "", "x").replace("status: active", "status: deprecated"),
    );
    commit_text(
        &w.a,
        "ops/new",
        &with_props("New way", "supersedes:\n  - \"[[ops/old]]\"\n", "x"),
    );
    let mut c = client(&w);
    let r = c.call("show", json!({"id": "ops/target"}));
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(
        r.json["backlinks"],
        json!(["ops/body-link", "ops/prop-link"])
    );
    assert_eq!(r.json["backlinks_total"], 2);
    assert!(r.json["version"].is_string());
    // The last editor comes from Git, not from the frontmatter.
    assert_eq!(r.json["last_editor"], "Test Setup <setup@example.org>");
    assert_eq!(r.json["pending_edit"], false);
    let d = c.call("show", json!({"id": "ops/disputed"}));
    assert_eq!(d.json["disputed"], true);
    assert_eq!(d.json["dispute_reason"], "The port is wrong.");
    let o = c.call("show", json!({"id": "ops/old"}));
    assert_eq!(o.json["replaced_by"], json!(["ops/new"]));
    let n = c.call("show", json!({"id": "ops/new"}));
    assert_eq!(n.json["supersedes"], json!(["ops/old"]));
}

#[test]
fn t12_the_folders_limit_hides_documents() {
    let w = World::new();
    commit_doc(&w.a, "engineering/api", "Shared api", "keys");
    commit_doc(&w.a, "ops/deploy", "Shared deploy", "keys");
    set_shared(&w, "# folders = []", "folders = [\"engineering/**\"]");
    let mut c = client(&w);
    let r = c.call("search", json!({"query": "shared"}));
    assert_eq!(ids(&r.json), vec!["engineering/api"]);
    let s = c.call("show", json!({"id": "ops/deploy"}));
    assert_eq!(s.code(), Some("not_found"));
    let l = c.call("list", json!({}));
    assert_eq!(ids(&l.json), vec!["engineering/api"]);
}

#[test]
fn t13_t18_stats() {
    let w = World::new();
    commit_text(&w.a, "backend/a", &with_props("A", "", "one two three"));
    commit_text(&w.a, "backend/b", &with_props("B", "", "one two"));
    commit_text(
        &w.a,
        "ops/c",
        &with_props("C", "", "one").replace(
            "Anna Example <anna@example.org>",
            "Ben Example <ben@example.org>",
        ),
    );
    let mut c = client(&w);
    let r = c.call("stats", json!({"by": "author"}));
    assert!(!r.is_error, "{}", r.json);
    let groups = r.json["groups"].as_array().unwrap();
    assert_eq!(groups.len(), 2);
    let sum: u64 = groups.iter().map(|g| g["value"].as_u64().unwrap()).sum();
    assert_eq!(sum, r.json["total"].as_u64().unwrap());
    let d = c.call("stats", json!({"metric": "documents", "by": "folder"}));
    assert_eq!(d.json["total"], 3);
    assert_eq!(
        d.json["groups"],
        json!([
            {"key": "backend", "value": 2, "documents": 2},
            {"key": "ops", "value": 1, "documents": 1}
        ])
    );
    let f = c.call(
        "stats",
        json!({"metric": "documents", "folders": ["backend"]}),
    );
    assert_eq!(f.json["total"], 2);
}

#[test]
fn t14_a_stale_clone_fetches_before_it_reads() {
    let w = World::new();
    let mut c = McpClient::for_person(
        &w,
        Person::A,
        Spawn {
            env: vec![("WAYMARK_FETCH_MAX_AGE".into(), "0".into())],
            ..Spawn::default()
        },
    );
    assert_eq!(
        c.call("search", json!({"query": "remote"})).json["total_matches"],
        0
    );
    commit_doc(&w.b, "ops/from-b", "Remote document", "x");
    let r = c.call("search", json!({"query": "remote"}));
    assert_eq!(ids(&r.json), vec!["ops/from-b"]);
}

#[test]
fn t15_a_failed_fetch_answers_locally_with_a_notice() {
    let w = World::new();
    commit_doc(&w.a, "ops/local", "Local document", "x");
    git(
        &w.a,
        &["remote", "set-url", "origin", "/nonexistent/remote.git"],
    );
    let mut c = client(&w);
    let r = c.call("search", json!({"query": "local"}));
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(ids(&r.json), vec!["ops/local"]);
    assert!(r.json["notices"].to_string().contains("may be behind"));
}

#[test]
fn t16_a_moved_document_is_followed() {
    let w = World::new();
    commit_doc(&w.a, "ops/old-name", "Moved document", "x");
    git(
        &w.a,
        &[
            "mv",
            "knowledge/ops/old-name.md",
            "knowledge/ops/new-name.md",
        ],
    );
    git(&w.a, &["commit", "-q", "-m", "test: move"]);
    let mut c = client(&w);
    let r = c.call("show", json!({"id": "ops/old-name"}));
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(r.json["id"], "ops/new-name");
    assert_eq!(r.json["moved_from"], "ops/old-name");
    assert!(r.json["notices"].to_string().contains("moved"));
    let none = c.call("show", json!({"id": "ops/never-existed"}));
    assert_eq!(none.code(), Some("not_found"));
}

#[test]
fn t17_generated_index_files_are_skipped() {
    let w = World::new();
    commit_doc(&w.a, "ops/doc", "Normal document", "x");
    commit_text(
        &w.a,
        "ops/index",
        &with_props(
            "Index of ops",
            "tags:\n  - index\n",
            "Lists the uniqueword here.",
        ),
    );
    let mut c = client(&w);
    let r = c.call("search", json!({"query": "uniqueword"}));
    assert_eq!(r.json["results"], json!([]));
    assert!(!c.call("show", json!({"id": "ops/index"})).is_error);
    let l = c.call("list", json!({}));
    assert_eq!(ids(&l.json), vec!["ops/doc"]);
}

#[test]
fn t19_list_pages_through_all_documents() {
    let w = World::new();
    for i in 0..7 {
        commit_doc(&w.a, &format!("ops/d{i}"), &format!("Doc {i}"), "x");
    }
    let mut c = client(&w);
    let mut seen = Vec::new();
    let mut offset = 0;
    loop {
        let r = c.call("list", json!({"limit": 3, "offset": offset}));
        seen.extend(ids(&r.json));
        match r.json.get("next_offset").and_then(Value::as_u64) {
            Some(n) => offset = n,
            None => break,
        }
    }
    let expected: Vec<String> = (0..7).map(|i| format!("ops/d{i}")).collect();
    assert_eq!(seen, expected);
}

#[test]
fn t20_an_uncommitted_edit_shows() {
    let w = World::new();
    commit_doc(&w.a, "ops/doc", "Edited document", "old text");
    write_doc(&w.a, "ops/doc", &doc("Edited document", "freshword"));
    let mut c = client(&w);
    let r = c.call("search", json!({"query": "freshword"}));
    assert_eq!(ids(&r.json), vec!["ops/doc"]);
    let s = c.call("show", json!({"id": "ops/doc"}));
    assert_eq!(s.json["pending_edit"], true);
}

#[test]
fn t21_two_thousand_documents_are_searched_quickly() {
    let w = World::new();
    for i in 0..2000 {
        write_doc(
            &w.a,
            &format!("gen/d{i}"),
            &doc(&format!("Generated {i}"), "some text for the scan"),
        );
    }
    write_doc(&w.a, "gen/target", &doc("Target", "findmeneedle"));
    let mut c = client(&w);
    // The first call fetches. Time the second.
    c.call("search", json!({"query": "x"}));
    let start = Instant::now();
    let r = c.call("search", json!({"query": "findmeneedle"}));
    let ms = start.elapsed().as_millis();
    assert_eq!(ids(&r.json), vec!["gen/target"]);
    assert!(ms < 500, "took {ms} ms");
}

#[test]
fn t22_an_empty_query_is_invalid() {
    let w = World::new();
    let mut c = client(&w);
    let r = c.call("search", json!({"query": "   "}));
    assert_eq!(r.code(), Some("invalid_arguments"));
    assert!(r.json["next"].is_string());
    let r = c.call("search", json!({"query": "x", "level": 7}));
    assert_eq!(r.code(), Some("invalid_arguments"));
    let r = c.call("search", json!({"query": "x", "in": ["nothing"]}));
    assert_eq!(r.code(), Some("invalid_arguments"));
}

#[test]
fn the_read_tools_work_when_writes_are_paused_or_read_only() {
    let w = World::new();
    commit_doc(&w.a, "ops/doc", "Readable document", "x");
    let mut c = McpClient::for_person(&w, Person::A, Spawn::args(&["--access", "read-only"]));
    let names = c.tool_names();
    for t in ["search", "show", "list", "stats"] {
        assert!(names.contains(&t.to_string()), "{t}");
    }
    assert!(!c.call("search", json!({"query": "readable"})).is_error);
}
