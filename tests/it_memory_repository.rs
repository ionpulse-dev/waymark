//! End-to-end tests of the memory repository layout (see `specs/foundation/memory-repository/test.md`).

mod common;
use common::git::git;
use common::{McpClient, Person, Spawn, World};
use serde_json::json;

fn check<'a>(r: &'a serde_json::Value, name: &str) -> &'a serde_json::Value {
    r["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == name)
        .unwrap()
}

#[test]
fn gitignore_hides_personal_obsidian_files() {
    let w = World::new();
    let obsidian = w.a.join(".obsidian");
    std::fs::write(obsidian.join("workspace.json"), "{}").unwrap();
    std::fs::write(obsidian.join("graph.json"), "{}").unwrap();
    assert_eq!(git(&w.a, &["status", "--porcelain"]), "");
    // The shared file stays in Git.
    assert!(git(&w.a, &["ls-files"]).contains(".obsidian/app.json"));
}

#[test]
fn init_writes_obsidian_defaults() {
    let w = World::new();
    let app: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(w.a.join(".obsidian/app.json")).unwrap())
            .unwrap();
    assert_eq!(app["useMarkdownLinks"], true);
    assert_eq!(app["newLinkFormat"], "relative");
}

#[test]
fn missing_id_is_setup_problem() {
    let w = World::new();
    let path = w.a.join("agent-memory/.waymark/settings.toml");
    let text = std::fs::read_to_string(&path).unwrap();
    let without: String = text
        .lines()
        .filter(|l| !l.starts_with("id ="))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&path, without).unwrap();
    let mut c = McpClient::for_person(&w, Person::A, Spawn::default());
    let r = c.call("status", json!({}));
    assert_eq!(r.json["healthy"], false);
    let memrepo = check(&r.json, "clone");
    assert_eq!(memrepo["ok"], false);
    assert!(memrepo["detail"].as_str().unwrap().contains("id"));
}

#[test]
fn stricter_value_applies() {
    let w = World::new();
    let path = w.a.join("agent-memory/.waymark/settings.toml");
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::write(
        &path,
        text.replace("[limits]", "[limits]\nmax_tokens = 400"),
    )
    .unwrap();
    let mut c = McpClient::for_person(&w, Person::A, Spawn::args(&["--max-tokens", "900"]));
    assert_eq!(
        c.call("status", json!({})).json["settings"]["max_tokens"],
        400
    );
}

#[test]
fn unknown_keys_give_a_notice_and_no_error() {
    let w = World::new();
    let path = w.a.join("agent-memory/.waymark/settings.toml");
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::write(&path, format!("{text}\n[future]\nthing = 1\n")).unwrap();
    let mut c = McpClient::for_person(&w, Person::A, Spawn::default());
    let r = c.call("status", json!({}));
    assert!(!r.is_error);
    assert_eq!(r.json["healthy"], true);
    assert!(r.json["notices"]
        .as_array()
        .map(|n| !n.is_empty())
        .unwrap_or(false));
}

#[test]
fn the_layout_has_the_root_in_agent_memory_and_no_owners_field() {
    let w = World::new();
    let memrepo = waymark::memrepo::MemRepo::open(&w.a).unwrap();
    assert!(memrepo.root.ends_with("agent-memory"));
    assert!(memrepo.documents().is_empty());
    let toml = std::fs::read_to_string(w.a.join("agent-memory/.waymark/settings.toml")).unwrap();
    assert!(!toml.contains("owners"));
}

#[test]
fn a_second_person_gets_the_same_memory_repository_id() {
    let w = World::new();
    let mut a = McpClient::for_person(&w, Person::A, Spawn::default());
    let mut b = McpClient::for_person(&w, Person::B, Spawn::default());
    let ra = a.call("status", json!({}));
    let rb = b.call("status", json!({}));
    assert_eq!(
        ra.json["memory_repository"]["id"],
        rb.json["memory_repository"]["id"]
    );
}

#[test]
fn init_writes_the_schema() {
    let w = World::new();
    let text = std::fs::read_to_string(w.a.join("agent-memory/.waymark/settings.toml")).unwrap();
    assert!(text.lines().any(|l| l.trim() == "schema = 1"), "{text}");
}

#[test]
fn a_higher_schema_blocks_every_tool_but_status() {
    let w = World::new();
    let path = w.a.join("agent-memory/.waymark/settings.toml");
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::write(&path, text.replace("schema = 1", "schema = 2")).unwrap();
    let mut c = McpClient::for_person(&w, Person::A, Spawn::default());
    for (tool, args) in [
        ("search", json!({})),
        ("search", json!({"query": "x"})),
        (
            "add_document",
            json!({"folder": "ops", "title": "T", "description": "D.", "body": "B."}),
        ),
    ] {
        let r = c.call(tool, args);
        assert_eq!(r.code(), Some("setup_problem"), "{tool}: {}", r.json);
        assert!(
            r.message().contains("needs a newer waymark"),
            "{tool}: {}",
            r.json
        );
        assert!(r.json["next"]
            .as_str()
            .unwrap()
            .contains(waymark::platform::update_sentence()));
    }
    let s = c.call("status", json!({}));
    assert!(!s.is_error, "{}", s.json);
    assert_eq!(s.json["healthy"], false);
    assert!(
        s.json.to_string().contains("needs a newer waymark"),
        "{}",
        s.json
    );
    assert!(s
        .json
        .to_string()
        .contains(waymark::platform::update_sentence()));
}

#[test]
fn a_file_without_schema_works_as_schema_1() {
    let w = World::new();
    let path = w.a.join("agent-memory/.waymark/settings.toml");
    let text = std::fs::read_to_string(&path).unwrap();
    let without: String = text
        .lines()
        .filter(|l| !l.starts_with("schema"))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&path, without).unwrap();
    let mut c = McpClient::for_person(&w, Person::A, Spawn::default());
    let r = c.call("search", json!({}));
    assert!(!r.is_error, "{}", r.json);
}

fn remove_schema_and_commit(w: &World) {
    let path = w.a.join("agent-memory/.waymark/settings.toml");
    let text = std::fs::read_to_string(&path).unwrap();
    let without: String = text
        .lines()
        .filter(|l| !l.starts_with("schema"))
        .map(|l| format!("{l}\n"))
        .collect();
    std::fs::write(&path, without).unwrap();
    git(&w.a, &["add", "-A"]);
    git(
        &w.a,
        &[
            "-c",
            "user.name=Host",
            "-c",
            "user.email=host@corp.org",
            "commit",
            "-q",
            "-m",
            "old settings",
        ],
    );
    git(&w.a, &["push", "-q", "origin", "HEAD"]);
}

#[test]
fn a_write_repairs_a_missing_schema_in_its_own_commit() {
    let w = World::new();
    remove_schema_and_commit(&w);
    let mut c = McpClient::for_person(&w, Person::A, Spawn::default());
    // Reads and previews never write.
    let head = git(&w.remote, &["rev-parse", "main"]);
    assert!(!c.call("search", json!({})).is_error);
    assert!(!c
        .call(
            "add_document",
            json!({"folder": "ops", "title": "T", "description": "D.", "body": "B.", "preview": true})
        )
        .is_error);
    assert_eq!(git(&w.remote, &["rev-parse", "main"]), head);
    // A write repairs first, then makes its own commit.
    let r = c.call(
        "add_document",
        json!({"folder": "ops", "title": "After repair", "description": "D.", "body": "B."}),
    );
    assert!(!r.is_error, "{}", r.json);
    assert!(
        r.json.to_string().contains("added the missing schema key"),
        "{}",
        r.json
    );
    let log = git(&w.remote, &["log", "--format=%s", "-3", "main"]);
    let lines: Vec<&str> = log.lines().collect();
    assert_eq!(lines[0], "waymark: add \"After repair\"", "{log}");
    assert_eq!(lines[1], "waymark: repair \"settings\"", "{log}");
    let settings = git(
        &w.remote,
        &["show", "main:agent-memory/.waymark/settings.toml"],
    );
    // A comment on top stays on top, and the key comes before the first setting.
    assert!(
        settings.lines().next().unwrap().starts_with('#'),
        "{settings}"
    );
    let schema_at = settings.find("\nschema = 1\n").expect("the key");
    let name_at = settings.find("\nname = ").expect("a name");
    assert!(schema_at < name_at, "{settings}");
    // A second write has nothing to repair.
    let before = git(&w.remote, &["rev-list", "--count", "main"]);
    let r = c.call(
        "add_document",
        json!({"folder": "ops", "title": "Again", "description": "D.", "body": "B."}),
    );
    assert!(!r.is_error, "{}", r.json);
    assert!(!r.json.to_string().contains("Repaired"));
    let after = git(&w.remote, &["rev-list", "--count", "main"]);
    assert_eq!(
        after.trim().parse::<i32>().unwrap(),
        before.trim().parse::<i32>().unwrap() + 1
    );
}

#[test]
fn a_repair_keeps_the_values_that_the_owner_set() {
    let w = World::new();
    let path = w.a.join("agent-memory/.waymark/settings.toml");
    let text = std::fs::read_to_string(&path).unwrap();
    let edited = text.replace("# max_results = 5", "max_results = 2 # tight");
    assert_ne!(edited, text);
    std::fs::write(&path, edited).unwrap();
    remove_schema_and_commit(&w);
    let mut c = McpClient::for_person(&w, Person::A, Spawn::default());
    let r = c.call(
        "add_document",
        json!({"folder": "ops", "title": "Keep limits", "description": "D.", "body": "B."}),
    );
    assert!(!r.is_error, "{}", r.json);
    let settings = git(
        &w.remote,
        &["show", "main:agent-memory/.waymark/settings.toml"],
    );
    assert!(settings.contains("max_results = 2 # tight"), "{settings}");
    assert!(
        settings.contains("\nschema = 1\n") || settings.starts_with("schema = 1\n"),
        "{settings}"
    );
}

#[test]
fn the_template_explains_every_setting_and_keeps_no_wrong_header() {
    let w = World::new();
    let text = std::fs::read_to_string(w.a.join("agent-memory/.waymark/settings.toml")).unwrap();
    assert!(!text.contains("Every key in the sections below is optional"));
    assert!(text.contains("Identifies this memory repository. Required."));
    // Every line that sets a key has a comment line right above it.
    let lines: Vec<&str> = text.lines().collect();
    for (i, l) in lines.iter().enumerate() {
        let is_key = !l.starts_with('#') && l.contains(" = ") && !l.starts_with('[');
        if is_key {
            assert!(lines[i - 1].starts_with('#'), "no comment above: {l}");
        }
    }
}
