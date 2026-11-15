//! End-to-end tests of the tool conventions (see `specs/foundation/tool-conventions/test.md`).

mod common;
use common::git::git;
use common::{McpClient, Person, Spawn, World};
use serde_json::json;

fn client(w: &World) -> McpClient {
    McpClient::for_person(w, Person::A, Spawn::default())
}

fn personal(w: &World, text: &str) {
    let p = w.root.join("home-a/.config/waymark/config.toml");
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, text).unwrap();
}

const OLD_NAMES: [&str; 10] = [
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

fn with_verification() -> World {
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
    w
}

#[test]
fn tool_names_follow_the_naming_scheme() {
    let w = with_verification();
    let mut c = client(&w);
    let tools = c.list_tools();
    let names: Vec<String> = tools
        .iter()
        .map(|t| t["name"].as_str().unwrap().to_string())
        .collect();
    for old in OLD_NAMES {
        assert!(!names.contains(&old.to_string()), "{old} is still listed");
    }
    for n in &names {
        assert!(
            n.chars().all(|c| c.is_ascii_lowercase() || c == '_'),
            "{n} is not snake_case"
        );
    }
    for must in [
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
        "history",
        "move",
    ] {
        assert!(
            names.contains(&must.to_string()),
            "{must} is not listed: {names:?}"
        );
    }
    // A write tool of one document ends in `_document`. A folder tool has `folder` in its name.
    for t in &tools {
        let n = t["name"].as_str().unwrap();
        let writes = t["annotations"]["readOnlyHint"] == json!(false);
        if writes && n.ends_with("_document") {
            continue;
        }
        if writes {
            assert!(
                [
                    "move",
                    "sync",
                    "undo",
                    "init",
                    "create_folder",
                    "delete_folder",
                    "rename_tag"
                ]
                .contains(&n),
                "{n} breaks the naming scheme"
            );
        }
    }
}

#[test]
fn every_parameter_has_a_description() {
    let w = with_verification();
    let mut c = client(&w);
    for t in c.list_tools() {
        let props = t["inputSchema"]["properties"].as_object().unwrap();
        for (name, p) in props {
            let d = p["description"].as_str().unwrap_or("");
            assert!(
                !d.trim().is_empty(),
                "{}.{name} has no description",
                t["name"]
            );
            // The typed objects of `init` describe their keys too.
            if let Some(inner) = p["properties"].as_object() {
                for (k, q) in inner {
                    assert!(
                        !q["description"].as_str().unwrap_or("").trim().is_empty(),
                        "{}.{name}.{k} has no description",
                        t["name"]
                    );
                }
            }
        }
    }
}

#[test]
fn every_description_says_what_it_returns_and_write_tools_say_they_commit() {
    let w = with_verification();
    let mut c = client(&w);
    for t in c.list_tools() {
        let n = t["name"].as_str().unwrap();
        let d = t["description"].as_str().unwrap();
        assert!(
            d.contains("Returns "),
            "{n} does not say what it returns: {d}"
        );
        // `sync` is a read tool for the limits (with read-only access it only pulls), and it commits.
        if t["annotations"]["readOnlyHint"] == json!(false) || n == "sync" {
            assert!(
                d.contains("commit"),
                "{n} is a write tool and does not say that it commits: {d}"
            );
        } else {
            assert!(d.contains("Changes nothing"), "{n} is a read tool: {d}");
        }
    }
}

#[test]
fn the_old_name_list_still_runs_with_a_notice() {
    let w = World::new();
    let mut c = client(&w);
    let r = c.call("list", json!({}));
    assert!(!r.is_error, "{}", r.json);
    let notice = r.json["notices"][0].as_str().unwrap_or("");
    assert!(
        notice.contains("\"list\"") && notice.contains("\"search\"") && notice.contains("0.6.0"),
        "{notice}"
    );
    // The names of 0.4.0 stopped working in 0.5.0, and a tool that never existed is unknown.
    for old in OLD_NAMES.iter().filter(|n| **n != "list") {
        assert!(
            c.call_raw(old, json!({}))["error"].is_object(),
            "{old} is still a tool"
        );
    }
    assert!(c.call_raw("set_limits", json!({}))["error"].is_object());
}

#[test]
fn an_old_name_in_a_limit_still_blocks_the_new_tool() {
    let w = World::new();
    personal(&w, "[limits]\ntools_denied = [\"delete\", \"folders\"]\n");
    let mut c = client(&w);
    let names = c.tool_names();
    assert!(!names.contains(&"delete_document".to_string()), "{names:?}");
    assert!(!names.contains(&"folder_tree".to_string()), "{names:?}");
    assert!(names.contains(&"add_document".to_string()));
    let r = c.call("delete_document", json!({"id": "x", "base": "y"}));
    assert_eq!(r.code(), Some("access_denied"));
    // The old name in a call is an unknown tool, so it cannot get around the limit.
    assert!(c.call_raw("delete", json!({"id": "x", "base": "y"}))["error"].is_object());
    // The entry flag `--no-delete` works too.
    let w = World::new();
    let mut c = McpClient::for_person(&w, Person::A, Spawn::args(&["--no-delete"]));
    assert!(!c.tool_names().contains(&"delete_document".to_string()));
}

#[test]
fn activity_since_accepts_a_date_and_an_rfc_3339_time() {
    let w = World::new();
    let mut c = client(&w);
    let r = c.call(
        "add_document",
        json!({"folder": "ops", "title": "Recent", "description": "About recent.", "body": "x"}),
    );
    assert!(!r.is_error);
    let a = c.call("activity", json!({"since": "2020-01-01"}));
    assert!(!a.is_error, "{}", a.json);
    assert_eq!(a.json["entries"].as_array().unwrap().len(), 1);
    let a = c.call("activity", json!({"since": "2020-01-01T09:00:00Z"}));
    assert!(!a.is_error, "{}", a.json);
    assert_eq!(a.json["entries"].as_array().unwrap().len(), 1);
    // A time in the future leaves everything out.
    let a = c.call("activity", json!({"since": "2999-01-01T00:00:00Z"}));
    assert!(!a.is_error, "{}", a.json);
    assert!(a.json["entries"].as_array().unwrap().is_empty());
    let a = c.call("activity", json!({"since": "yesterday"}));
    assert_eq!(a.code(), Some("invalid_arguments"));
    // `updates` takes the same forms.
    let u = c.call("updates", json!({"since": "2020-01-01T09:00:00Z"}));
    assert!(!u.is_error, "{}", u.json);
    let u = c.call("updates", json!({"since": "2999-01-01"}));
    assert_eq!(u.code(), Some("invalid_arguments"));
}
