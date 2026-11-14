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
    // About 4 bytes per token: 20000 bytes are about 5000 tokens for the whole list.
    assert!(total < 20_000, "the tool list is {total} bytes");
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
