//! Context budget (task G5 of the plan). The tool list and the server instructions are sent into
//! the context of the agent at every session. Keep them small.

mod common;
use common::{McpClient, Person, Spawn, World};
use serde_json::json;

#[test]
fn tool_descriptions_and_instructions_stay_small() {
    let w = World::new();
    let mut c = McpClient::for_person(&w, Person::A, Spawn::args(&[]));
    let tools = c.list_tools();
    let total: usize = tools.iter().map(|t| t.to_string().len()).sum();
    eprintln!("tools: {} entries, {} bytes", tools.len(), total);
    for t in &tools {
        let n = t.to_string().len();
        assert!(n < 3000, "{} is {n} bytes", t["name"]);
    }
    // About 4 bytes per token: 28000 bytes are about 7000 tokens for the whole list.
    assert!(total < 28_000, "the tool list is {total} bytes");
}

#[test]
fn the_tool_list_is_smaller_than_in_0_4_0() {
    // Version 0.4.0 listed 24 tools in 20223 bytes (see `specs/features/lean-tools`).
    let w = World::new();
    let mut c = McpClient::for_person(&w, Person::A, Spawn::args(&[]));
    let tools = c.list_tools();
    let total: usize = tools.iter().map(|t| t.to_string().len()).sum();
    eprintln!("tools: {} entries, {total} bytes", tools.len());
    assert_eq!(tools.len(), 23);
    assert!(total < 20_223, "the tool list is {total} bytes");
}

#[test]
fn search_results_are_capped() {
    let w = World::new();
    let mut c = McpClient::for_person(&w, Person::A, Spawn::args(&[]));
    let r = c.call("search", json!({"query": "the", "limit": 1000}));
    let n = r.json.to_string().len();
    eprintln!("search result: {n} bytes");
    assert!(n < 60_000, "{n}");
}
