//! End-to-end tests of the filters and the paging of `search` and `list`
//! (see `specs/features/search-filters/test.md`).

mod common;
use common::git::git;
use common::memrepo::{doc, write_doc};
use common::{McpClient, Person, Spawn, World};
use serde_json::{json, Value};
use std::path::Path;

fn client(w: &World) -> McpClient {
    McpClient::for_person(w, Person::A, Spawn::default())
}

fn ok(r: common::Reply) -> Value {
    assert!(!r.is_error, "{}", r.json);
    r.json
}

fn ids(v: &Value) -> Vec<String> {
    v["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["id"].as_str().unwrap().to_string())
        .collect()
}

/// A document with its own author and `updated` date.
fn put(dir: &Path, id: &str, title: &str, author: &str, updated: &str, extra: &str) {
    let text = doc(title, "shared words")
        .replace(common::memrepo::AUTHOR, author)
        .replace(
            &format!("updated: {}", common::memrepo::TODAY),
            &format!("updated: {updated}"),
        )
        .replace("---\n\nshared", &format!("{extra}---\n\nshared"));
    write_doc(dir, id, &text);
}

fn commit_all(w: &World) {
    git(&w.a, &["add", "-A"]);
    git(&w.a, &["commit", "-q", "-m", "test: documents"]);
    git(&w.a, &["push", "-q"]);
}

const BOB: &str = "Bob Builder <bob@example.org>";
const ANNA: &str = "Anna Example <anna@example.org>";

fn mixed(w: &World) {
    put(&w.a, "ops/a", "Alpha", ANNA, "2026-11-01", "");
    put(&w.a, "ops/b", "Beta", BOB, "2026-11-05", "");
    put(
        &w.a,
        "ops/c",
        "Gamma",
        BOB,
        "2026-11-10",
        "disputed: true\ndispute_reason: Wrong.\n",
    );
    put(&w.a, "ops/d", "Delta", ANNA, "2026-11-12", "");
    commit_all(w);
}

#[test]
fn search_pages_with_offset_and_next_offset() {
    let w = World::new();
    mixed(&w);
    let mut c = McpClient::for_person(&w, Person::A, Spawn::args(&["--max-results", "2"]));
    let p1 = ok(c.call("search", json!({"query": "shared", "folders": ["ops"]})));
    assert_eq!(p1["total_matches"], 4);
    assert_eq!(p1["truncated"], true);
    assert_eq!(p1["next_offset"], 2);
    let p2 = ok(c.call(
        "search",
        json!({"query": "shared", "folders": ["ops"], "offset": p1["next_offset"]}),
    ));
    assert_eq!(p2["truncated"], false);
    assert!(p2.get("next_offset").is_none());
    let mut all = ids(&p1);
    all.extend(ids(&p2));
    all.sort();
    all.dedup();
    assert_eq!(all.len(), 4, "no repeats: {all:?}");
    let past = ok(c.call("search", json!({"query": "shared", "offset": 99})));
    assert!(ids(&past).is_empty());
    assert_eq!(past["truncated"], false);
}

#[test]
fn list_and_search_filter_by_author() {
    let w = World::new();
    mixed(&w);
    let mut c = client(&w);
    for tool in ["list", "search"] {
        for who in ["bob", "BUILDER", "bob@example", "Bob Builder"] {
            let mut args = json!({"author": who, "folders": ["ops"]});
            if tool == "search" {
                args["query"] = json!("shared");
            }
            let mut got = ids(&ok(c.call(tool, args)));
            got.sort();
            assert_eq!(got, ["ops/b", "ops/c"], "{tool} {who}");
        }
    }
}

#[test]
fn list_and_search_filter_by_the_updated_range() {
    let w = World::new();
    mixed(&w);
    let mut c = client(&w);
    for tool in ["list", "search"] {
        let mut args = json!({"updated_after": "2026-11-05", "updated_before": "2026-11-10", "folders": ["ops"]});
        if tool == "search" {
            args["query"] = json!("shared");
        }
        let mut got = ids(&ok(c.call(tool, args)));
        got.sort();
        assert_eq!(got, ["ops/b", "ops/c"], "{tool}");
        let mut args = json!({"updated_after": "2026-11-11", "folders": ["ops"]});
        if tool == "search" {
            args["query"] = json!("shared");
        }
        assert_eq!(ids(&ok(c.call(tool, args))), ["ops/d"], "{tool}");
    }
}

#[test]
fn search_filters_disputed_documents() {
    let w = World::new();
    mixed(&w);
    let mut c = client(&w);
    let v = ok(c.call("search", json!({"query": "shared", "disputed": true})));
    assert_eq!(ids(&v), ["ops/c"]);
    let v = ok(c.call(
        "search",
        json!({"query": "shared", "disputed": false, "folders": ["ops"], "limit": 5}),
    ));
    assert_eq!(v["total_matches"], 4);
}

#[test]
fn bad_filter_arguments_are_invalid_arguments() {
    let w = World::new();
    mixed(&w);
    let mut c = client(&w);
    for (tool, args) in [
        (
            "search",
            json!({"query": "shared", "updated_after": "2026-13-01"}),
        ),
        (
            "search",
            json!({"query": "shared", "updated_before": "yesterday"}),
        ),
        ("search", json!({"query": "shared", "offset": -1})),
        (
            "list",
            json!({"updated_after": "2026-11-10", "updated_before": "2026-11-01"}),
        ),
        ("list", json!({"updated_after": "11/10/2026"})),
    ] {
        let r = c.call(tool, args.clone());
        assert_eq!(r.code(), Some("invalid_arguments"), "{tool} {args}");
        assert!(r.json["next"].is_string(), "{}", r.json);
    }
}

#[test]
fn filters_never_reveal_documents_outside_the_folders_limit() {
    let w = World::new();
    mixed(&w);
    put(&w.a, "work/x", "Hidden", BOB, "2026-11-06", "");
    commit_all(&w);
    let p = w.root.join("home-a/.config/waymark/config.toml");
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, "[limits]\nfolders = [\"ops/**\"]\n").unwrap();
    let mut c = client(&w);
    let v = ok(c.call("list", json!({"author": "bob"})));
    assert_eq!(ids(&v), ["ops/b", "ops/c"]);
    let v = ok(c.call(
        "search",
        json!({"query": "shared", "author": "bob", "updated_after": "2026-11-06"}),
    ));
    assert_eq!(ids(&v), ["ops/c"]);
}
