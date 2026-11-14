//! Tests of `agent-tool-setup`: the snippets in the docs match the real server.

mod common;
use common::{McpClient, Spawn, World};
use serde_json::Value;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(root().join(rel)).unwrap()
}

/// The fenced blocks that follow a marker comment: `(marker value, block text)`.
fn blocks(text: &str, marker: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let lines: Vec<&str> = text.lines().collect();
    let prefix = format!("<!-- {marker}:");
    let mut i = 0;
    while i < lines.len() {
        let l = lines[i].trim();
        if l.starts_with(&prefix) || l == format!("<!-- {marker} -->") {
            let value = l
                .strip_prefix(&prefix)
                .map(|v| v.trim_end_matches("-->").trim().to_string())
                .unwrap_or_default();
            // The next fence opens the block.
            let mut j = i + 1;
            while j < lines.len() && !lines[j].starts_with("```") {
                j += 1;
            }
            let mut body = Vec::new();
            j += 1;
            while j < lines.len() && !lines[j].starts_with("```") {
                body.push(lines[j]);
                j += 1;
            }
            out.push((value, body.join("\n")));
            i = j;
        }
        i += 1;
    }
    out
}

fn snippets() -> Vec<(String, String)> {
    blocks(&read("docs/install.md"), "snippet")
}

/// Tools that the permission advice names before their features exist (see `decisions.md`).
const NOT_YET_BUILT: &[&str] = &[];

#[test]
fn a1_every_snippet_parses_and_names_waymark() {
    let all = snippets();
    assert!(all.len() >= 6, "found {} snippets", all.len());
    for (name, text) in all {
        let lower = text.to_lowercase();
        for bad in ["token", "password", "secret"] {
            assert!(!lower.contains(bad), "{name} has {bad}");
        }
        if name == "codex" {
            let t: toml::Value = toml::from_str(&text).unwrap();
            let e = &t["mcp_servers"]["waymark"];
            assert_eq!(e["command"].as_str(), Some("waymark"));
            assert_eq!(e["args"][0].as_str(), Some("--repo"));
        } else if name == "claude-code-permissions" {
            let v: Value = serde_json::from_str(&text).unwrap();
            assert!(v["permissions"]["allow"].is_array());
        } else {
            let v: Value = serde_json::from_str(&text).unwrap();
            let servers = v
                .get("mcpServers")
                .or_else(|| v.get("servers"))
                .unwrap_or_else(|| panic!("{name}: no server table"));
            let e = &servers["waymark"];
            assert_eq!(e["command"].as_str(), Some("waymark"), "{name}");
            assert_eq!(e["args"][0].as_str(), Some("--repo"), "{name}");
            if let Some(env) = e.get("env") {
                for (_, val) in env.as_object().unwrap() {
                    let s = val.as_str().unwrap();
                    assert!(!s.contains('@') || s.ends_with("example.org"), "{name}");
                }
            }
        }
    }
}

#[test]
fn a3_agents_md_examples_are_marked_as_suggestions() {
    let install = blocks(&read("docs/install.md"), "example");
    assert_eq!(install.len(), 3);
    for (_, text) in install {
        let mut lines = text.lines();
        assert_eq!(
            lines.next().unwrap(),
            "Suggestion only. waymark does not insert or check this text."
        );
        assert_eq!(
            text.lines().last().unwrap(),
            "Never write secrets, tokens, or personal data."
        );
    }
}

#[test]
fn a4_the_server_writes_nothing_into_agent_tool_folders() {
    let w = World::new();
    let mut c = McpClient::for_person(&w, common::Person::A, Spawn::default());
    assert!(!c.call("status", serde_json::json!({})).is_error);
    assert!(
        !c.call("search", serde_json::json!({"query": "anything"}))
            .is_error
    );
    let home = c.home.clone();
    let _ = c.close();
    for name in [
        ".claude",
        ".claude.json",
        ".codex",
        ".copilot",
        ".vscode",
        ".mcp.json",
    ] {
        assert!(!home.join(name).exists(), "{name} exists");
    }
    assert!(!w.a.join(".mcp.json").exists() && !w.a.join(".vscode").exists());
}

#[test]
fn a5_tool_names_in_the_advice_are_real_tools() {
    let w = World::new();
    let mut c = McpClient::for_person(&w, common::Person::A, Spawn::default());
    let listed = c.tool_names();
    let text = read("docs/install.md");
    let mut found = 0;
    for part in text.split("mcp__waymark__").skip(1) {
        let name: String = part
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if name.is_empty() {
            continue; // the prefix in prose
        }
        found += 1;
        assert!(
            listed.contains(&name) || NOT_YET_BUILT.contains(&name.as_str()),
            "{name} is not a tool"
        );
        for gated in ["verify_document", "review", "reject_document"] {
            assert_ne!(name, gated, "advice must not name a verification tool");
        }
    }
    assert!(found >= 18);
    for name in NOT_YET_BUILT {
        // When a feature lands, remove its name from NOT_YET_BUILT.
        if listed.contains(&name.to_string()) {
            panic!("{name} exists now: remove it from NOT_YET_BUILT");
        }
    }
}

#[test]
fn a6_unverified_statements_keep_the_word() {
    let research = read("docs/agent-tools-research.md");
    assert!(research.contains("| unverified |"));
    let install = read("docs/install.md");
    for phrase in [
        "explicit `ask` rule still prompts",
        "Whether Claude Code shows that question",
        "Whether Codex uses the read-only hint",
        "project entry overrides a user entry of the same name in Codex",
        "The exact key is unverified",
        "How Copilot asks before a write tool",
        "How Copilot CLI asks before a write tool",
        "project entry overrides a user entry of the same name is unverified",
    ] {
        let line = install
            .lines()
            .find(|l| l.contains(phrase))
            .unwrap_or_else(|| panic!("missing: {phrase}"));
        let _ = line;
        let at = install.find(phrase).unwrap();
        let start = install[..at].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let end = install[at..]
            .find('\n')
            .map(|i| at + i)
            .unwrap_or(install.len());
        assert!(
            install[start..end].contains("unverified"),
            "no 'unverified' near: {phrase}"
        );
    }
}
