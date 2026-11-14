//! End-to-end tests of the MCP server (see `specs/foundation/mcp-server/test.md`).

mod common;
use common::git::git;
use common::{McpClient, Person, Spawn, World};
use serde_json::json;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

fn client(w: &World) -> McpClient {
    McpClient::for_person(w, Person::A, Spawn::default())
}

#[test]
fn lists_allowed_tools_only() {
    let w = World::new();
    let mut c = client(&w);
    let names = c.tool_names();
    assert!(names.contains(&"status".to_string()));
    for control in ["set_limits", "pause", "resume", "login", "set_access"] {
        assert!(
            !names.contains(&control.to_string()),
            "{control} must not exist"
        );
    }
}

#[test]
fn results_have_message_and_settings() {
    let w = World::new();
    let mut c = client(&w);
    let r = c.call("status", json!({}));
    assert!(!r.is_error);
    assert!(r.json["message"].is_string());
    assert_eq!(r.json["settings"]["access"], "read-write");
    // An error has them too.
    let e = c.call("status", json!({"x": 1}));
    assert!(e.is_error);
    assert!(e.json["settings"].is_object());
}

#[test]
fn control_tool_is_unknown() {
    let w = World::new();
    let mut c = client(&w);
    let r = c.call_raw("set_limits", json!({"access": "read-write"}));
    assert_eq!(r["error"]["code"], -32602);
    assert!(r["error"]["message"]
        .as_str()
        .unwrap()
        .contains("Unknown tool"));
}

#[test]
fn calls_are_idempotent() {
    let w = World::new();
    let mut c = client(&w);
    let head = git(&w.a, &["rev-parse", "HEAD"]);
    let first = c.call("status", json!({}));
    let second = c.call("status", json!({}));
    assert_eq!(first.json, second.json);
    assert_eq!(git(&w.a, &["rev-parse", "HEAD"]), head);
    assert_eq!(git(&w.a, &["status", "--porcelain"]), "");
}

#[test]
fn errors_have_code_and_next() {
    let w = World::new();
    let mut c = client(&w);
    let e = c.call("status", json!({"unknown": true}));
    assert_eq!(e.code(), Some("invalid_arguments"));
    assert!(e.json["next"].is_string());
}

#[test]
fn status_reports_the_memory_repository() {
    let w = World::new();
    let mut c = client(&w);
    let r = c.call("status", json!({}));
    assert_eq!(r.json["healthy"], true);
    assert_eq!(r.json["memory_repository"]["name"], "Test");
    let checks = r.json["checks"].as_array().unwrap();
    assert!(checks.iter().all(|c| c["ok"] == true));
}

#[test]
fn status_reports_a_path_that_is_not_a_memory_repository() {
    let dir = tempfile::tempdir().unwrap();
    let mut c = McpClient::start(
        &dir.path().join("home"),
        Spawn::args(&["--path", dir.path().to_str().unwrap()]),
    );
    let r = c.call("status", json!({}));
    assert!(!r.is_error, "status does not fail: it reports");
    assert_eq!(r.json["healthy"], false);
    let clone = r.json["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "clone")
        .unwrap();
    assert_eq!(clone["ok"], false);
    assert!(clone["fix"].is_string());
}

#[test]
fn read_only_access_shows_in_settings_and_status_still_works() {
    let w = World::new();
    let mut c = McpClient::for_person(&w, Person::A, Spawn::args(&["--access", "read-only"]));
    let r = c.call("status", json!({}));
    assert_eq!(r.json["settings"]["access"], "read-only");
    assert!(c.tool_names().contains(&"status".to_string()));
}

#[test]
fn tools_denied_does_not_hide_status() {
    let w = World::new();
    let mut c = McpClient::for_person(&w, Person::A, Spawn::args(&["--tools-denied", "status"]));
    assert!(c.tool_names().contains(&"status".to_string()));
    assert!(!c.call("status", json!({})).is_error);
}

#[test]
fn a_broken_config_file_is_reported_by_status() {
    let w = World::new();
    let home = w.root.join("home-a");
    std::fs::create_dir_all(home.join(".config/waymark")).unwrap();
    std::fs::write(home.join(".config/waymark/config.toml"), "access = [broken").unwrap();
    let mut c = client(&w);
    let r = c.call("status", json!({}));
    assert_eq!(r.json["healthy"], false);
    let settings = r.json["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "settings")
        .unwrap();
    assert!(settings["detail"].as_str().unwrap().contains("config.toml"));
    // The limits fall back to read-only.
    assert_eq!(r.json["settings"]["access"], "read-only");
}

#[test]
fn personal_layer_is_strictest() {
    let w = World::new();
    let home = w.root.join("home-a");
    std::fs::create_dir_all(home.join(".config/waymark")).unwrap();
    std::fs::write(
        home.join(".config/waymark/config.toml"),
        "[limits]\naccess = \"read-only\"\n",
    )
    .unwrap();
    let mut c = McpClient::for_person(&w, Person::A, Spawn::args(&["--access", "read-write"]));
    let r = c.call("status", json!({}));
    assert_eq!(r.json["settings"]["access"], "read-only");
}

/// The first clone runs in the background: ask until the setup is healthy.
fn healthy_soon(c: &mut McpClient) -> bool {
    for _ in 0..100 {
        if c.call("status", json!({})).json["healthy"] == true {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    false
}

#[test]
fn clones_once_and_reuses() {
    let w = World::new();
    let target = w.root.join("clone-c");
    let url = w.remote.display().to_string();
    let args = ["--repo", url.as_str(), "--path", target.to_str().unwrap()];
    let author = [
        "--author-name",
        "Cy Example",
        "--author-email",
        "cy@example.org",
    ];
    let all: Vec<&str> = args.iter().chain(author.iter()).copied().collect();
    {
        let mut c = McpClient::start(&w.root.join("home-c"), Spawn::args(&all));
        assert!(healthy_soon(&mut c));
    }
    assert!(target.join(".git").exists());
    git(&target, &["config", "--local", "marker.test", "kept"]);
    {
        let mut c = McpClient::start(&w.root.join("home-c"), Spawn::args(&all));
        assert!(healthy_soon(&mut c));
    }
    assert_eq!(
        git(&target, &["config", "--local", "marker.test"]).trim(),
        "kept"
    );
}

#[test]
fn a_wrong_memory_repository_id_is_a_setup_problem() {
    let w = World::new();
    let mut c = McpClient::for_person(
        &w,
        Person::A,
        Spawn::args(&[
            "--memory-repository-id",
            "00000000-0000-4000-8000-000000000000",
        ]),
    );
    let r = c.call("status", json!({}));
    assert_eq!(r.json["healthy"], false);
}

#[test]
fn nothing_but_protocol_messages_goes_to_stdout() {
    let w = World::new();
    let mut child = Command::new(env!("CARGO_BIN_EXE_waymark"))
        .args(["--path", w.a.to_str().unwrap()])
        .env("HOME", w.root.join("home-x"))
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    {
        let stdin = child.stdin.as_mut().unwrap();
        writeln!(stdin, "{}", json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"t","version":"0"}}})).unwrap();
        writeln!(
            stdin,
            "{}",
            json!({"jsonrpc":"2.0","method":"notifications/initialized"})
        )
        .unwrap();
        writeln!(stdin, "{}", json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"status","arguments":{}}})).unwrap();
    }
    drop(child.stdin.take());
    let lines: Vec<String> = BufReader::new(child.stdout.take().unwrap())
        .lines()
        .map(|l| l.unwrap())
        .collect();
    let _ = child.wait();
    assert!(lines.len() >= 2);
    for l in lines {
        let v: serde_json::Value = serde_json::from_str(&l).expect("stdout line is JSON");
        assert_eq!(v["jsonrpc"], "2.0");
    }
}

#[test]
fn version_and_help_work_and_no_other_command_exists() {
    let bin = env!("CARGO_BIN_EXE_waymark");
    let v = Command::new(bin).arg("--version").output().unwrap();
    assert!(v.status.success());
    assert!(String::from_utf8_lossy(&v.stdout).starts_with("waymark "));
    let h = Command::new(bin).arg("--help").output().unwrap();
    assert!(h.status.success());
    assert!(!String::from_utf8_lossy(&h.stdout).contains("Commands:"));
    let bad = Command::new(bin).arg("search").output().unwrap();
    assert!(!bad.status.success());
}

#[test]
fn tools_list_stays_under_the_budget() {
    let w = World::new();
    let mut c = client(&w);
    let tools = c.list_tools();
    let description_chars: usize = tools
        .iter()
        .map(|t| t["description"].as_str().unwrap_or("").len())
        .sum();
    let total = serde_json::to_string(&tools).unwrap().len();
    assert!(
        description_chars < 9000,
        "descriptions use {description_chars} characters"
    );
    assert!(total < 28000, "tools/list uses {total} characters");
}

#[test]
fn the_server_ends_when_the_client_closes() {
    let w = World::new();
    let c = client(&w);
    assert!(c.close());
}
