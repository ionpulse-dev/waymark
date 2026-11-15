//! A problem that only an update fixes must reach the person: in `initialize`, in the description
//! of `status`, and in the error of every tool (see `specs/foundation/mcp-server`, section
//! "Updates of the MCP entry", and `memory-repository`, section "Versions and updates").

mod common;
use common::{McpClient, Person, Spawn, World};
use serde_json::json;

fn instructions(c: &McpClient) -> String {
    c.initialize_result["instructions"]
        .as_str()
        .unwrap_or("")
        .to_string()
}

fn status_description(c: &mut McpClient) -> String {
    c.list_tools()
        .iter()
        .find(|t| t["name"] == "status")
        .map(|t| t["description"].as_str().unwrap_or("").to_string())
        .expect("status is listed")
}

#[test]
fn an_unknown_argument_reaches_the_person_and_blocks_the_tools() {
    let w = World::new();
    let mut c = McpClient::for_person(&w, Person::A, Spawn::args(&["--future-flag", "1"]));
    // initialize: the text names the problem and the fix.
    let ins = instructions(&c);
    assert!(ins.contains("WAYMARK NEEDS AN UPDATE"), "{ins}");
    assert!(ins.contains("--future-flag"), "{ins}");
    assert!(ins.contains("restart your agent tool"), "{ins}");
    // tools/list: only status, and its description carries the problem.
    assert_eq!(c.tool_names(), vec!["status".to_string()]);
    let d = status_description(&mut c);
    assert!(
        d.contains("WAYMARK NEEDS AN UPDATE") && d.contains(waymark::platform::update_step()),
        "{d}"
    );
    assert!(
        d.contains("claude mcp add --transport stdio --scope user waymark -- waymark"),
        "{d}"
    );
    // every other tool: the same message.
    let r = c.call("search", json!({}));
    assert_eq!(r.code(), Some("setup_problem"), "{}", r.json);
    assert!(r.message().contains("--future-flag"), "{}", r.json);
    assert!(r.json["next"]
        .as_str()
        .unwrap()
        .contains(waymark::platform::update_step()));
    // status: works, is not healthy, and has a failing check `entry`.
    let s = c.call("status", json!({}));
    assert!(!s.is_error, "{}", s.json);
    assert_eq!(s.json["healthy"], false);
    let entry = s.json["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|k| k["name"] == "entry")
        .expect("a check entry");
    assert_eq!(entry["ok"], false);
}

#[test]
fn a_wrong_value_does_not_guess_a_memory_repository() {
    let w = World::new();
    let mut c = McpClient::for_person(&w, Person::A, Spawn::args(&["--max-results", "many"]));
    assert!(instructions(&c).contains("WAYMARK NEEDS AN UPDATE"));
    let r = c.call("search", json!({}));
    assert_eq!(r.code(), Some("setup_problem"), "{}", r.json);
    assert!(r.message().contains("wrong argument"), "{}", r.json);
}

#[test]
fn a_memory_repository_with_a_higher_schema_reaches_the_person_too() {
    let w = World::new();
    let path = w.a.join("agent-memory/.waymark/settings.toml");
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::write(&path, text.replace("schema = 1", "schema = 2")).unwrap();
    let mut c = McpClient::for_person(&w, Person::A, Spawn::default());
    let ins = instructions(&c);
    assert!(ins.contains("WAYMARK NEEDS AN UPDATE"), "{ins}");
    assert!(ins.contains(waymark::platform::update_sentence()), "{ins}");
    assert_eq!(c.tool_names(), vec!["status".to_string()]);
    assert!(status_description(&mut c).contains("needs a newer waymark"));
    let r = c.call("search", json!({"query": "x"}));
    assert_eq!(r.code(), Some("setup_problem"));
    assert!(r.message().contains("needs a newer waymark"));
}

#[test]
fn a_healthy_setup_shows_no_warning() {
    let w = World::new();
    let mut c = McpClient::for_person(&w, Person::A, Spawn::default());
    assert!(!instructions(&c).contains("NEEDS AN UPDATE"));
    assert!(c.tool_names().len() > 5);
    assert!(!status_description(&mut c).contains("NEEDS AN UPDATE"));
}
