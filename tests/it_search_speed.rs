//! T21 of `specs/features/search-and-read/test.md`: the search speed. It is alone in this file, so
//! that the other tests of the search do not run in parallel with it and slow it down.

mod common;
use common::memrepo::{doc, write_doc};
use common::{McpClient, Person, Spawn, World};
use serde_json::{json, Value};
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
