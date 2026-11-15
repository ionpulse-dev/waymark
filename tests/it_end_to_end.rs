//! End-to-end scenarios across features (task G1 of the plan). One memory repository is created, two
//! people use it, and every limit layer is tried. Each step names the spec it covers.

mod common;
use common::git::git;
use common::{McpClient, Person, Spawn, World};
use serde_json::{json, Value};

fn client(w: &World, p: Person, extra: &[&str]) -> McpClient {
    let mut s = Spawn::args(extra);
    s.env = vec![("WAYMARK_FETCH_MAX_AGE".into(), "0".into())];
    McpClient::for_person(w, p, s)
}

fn ok(r: common::Reply) -> Value {
    assert!(!r.is_error, "{}", r.json);
    r.json
}

fn ids(v: &Value, key: &str) -> Vec<String> {
    v[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["id"].as_str().unwrap().to_string())
        .collect()
}

fn personal(w: &World, text: &str) {
    let p = w.root.join("home-a/.config/waymark/config.toml");
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, text).unwrap();
}

#[test]
fn a_team_creates_joins_writes_moves_undoes_disputes_and_syncs() {
    let w = World::empty();

    // create-memory-repository: the owner sets up the empty repository.
    let mut a = client(&w, Person::A, &[]);
    ok(a.call(
        "init",
        json!({"name": "Team KB", "email_domains": ["example.org"], "folders": ["ops"], "limits": {"max_results": 3}}),
    ));

    // join-memory-repository: a second person gets the shared limits with no setup.
    let mut b = client(&w, Person::B, &[]);
    let s = ok(b.call("status", json!({})));
    assert_eq!(s["healthy"], true, "{s}");
    assert_eq!(s["settings"]["max_results"], 3);

    // write-documents, search-and-read: A writes, B finds and reads.
    // write-documents: a body link to a document that does not exist is refused.
    let refused = a.call(
        "add_document",
        json!({"folder": "ops", "title": "Deploy pipeline", "description": "How code reaches production.", "body": "Use the pipeline. See [rollback](rollback.md)."}),
    );
    assert_eq!(
        refused.code(),
        Some("validation_failed"),
        "{}",
        refused.json
    );
    let added = ok(a.call(
        "add_document",
        json!({"folder": "ops", "title": "Deploy pipeline", "description": "How code reaches production.", "body": "Use the pipeline."}),
    ));
    let id = added["id"].as_str().unwrap().to_string();
    assert_eq!(id, "ops/deploy-pipeline");
    let found = ok(b.call("search", json!({"query": "pipeline production"})));
    assert_eq!(ids(&found, "results"), vec![id.clone()]);
    let shown = ok(b.call("show", json!({"id": id})));
    assert!(shown["body"].as_str().unwrap().contains("Use the pipeline"));
    assert!(
        shown["updated_by"]
            .as_str()
            .unwrap()
            .contains("anna@example.org")
            || shown.to_string().contains("anna@example.org")
    );

    // quality-checks: the written documents pass the checks.
    let lint = ok(a.call("lint", json!({})));
    assert!(!lint.to_string().contains("broken-link"), "{lint}");

    // write-documents: B changes it, with the version of the read.
    let base = shown["version"].as_str().unwrap().to_string();
    ok(b.call(
        "update_document",
        json!({"id": id, "base": base, "body": "Use the pipeline. Check the dashboard."}),
    ));
    // A used an older version: stale_base.
    let stale = a.call(
        "update_document",
        json!({"id": id, "base": added["version"], "body": "A text."}),
    );
    assert_eq!(stale.code(), Some("stale_base"), "{}", stale.json);
    let now = ok(a.call("show", json!({"id": id})));
    assert!(now["body"].as_str().unwrap().contains("dashboard"));

    // move-documents: A renames it. The old ID is gone, the new one works.
    ok(a.call("move", json!({"from": id, "to": "ops/release-pipeline"})));
    // `show` follows the move (see `move-documents`) and names the new ID.
    let old = b.call("show", json!({"id": id}));
    assert!(
        old.json.to_string().contains("ops/release-pipeline"),
        "{}",
        old.json
    );
    ok(b.call("show", json!({"id": "ops/release-pipeline"})));

    // activity-and-undo: A takes the move back.
    let act = ok(a.call("activity", json!({})));
    let mv = act["entries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["verb"] == "move")
        .expect("a move in the activity");
    ok(a.call("undo", json!({"commit": mv["commit"]})));
    ok(b.call("show", json!({"id": id})));

    // dispute, updates, notices: B disputes. A hears about it in a new session.
    a.call("updates", json!({}));
    drop(a);
    std::thread::sleep(std::time::Duration::from_millis(1100));
    let v = ok(b.call("show", json!({"id": id})));
    ok(b.call(
        "dispute_document",
        json!({"id": id, "base": v["version"], "reason": "The dashboard moved."}),
    ));
    let mut a = client(&w, Person::A, &[]);
    let r = ok(a.call("search", json!({"query": "pipeline"})));
    assert!(r.to_string().contains("disputed"), "{r}");
    assert!(
        r["results"][0]["disputed"] == true
            || r.to_string().contains("dispute_reason")
            || r.to_string().contains("The dashboard moved"),
        "{r}"
    );
    let up = ok(a.call("updates", json!({})));
    assert!(up.to_string().contains("The dashboard moved."));

    // manual-edits and sync with a conflict: both people change the same text.
    let v = ok(b.call("show", json!({"id": id})));
    ok(b.call(
        "dispute_document",
        json!({"id": id, "base": v["version"], "clear": true}),
    ));
    let v = ok(b.call("show", json!({"id": id})));
    ok(b.call(
        "update_document",
        json!({"id": id, "base": v["version"], "body": "B version of the text."}),
    ));
    let file = w.a.join("agent-memory/ops/deploy-pipeline.md");
    let text = std::fs::read_to_string(&file).unwrap();
    std::fs::write(&file, text.replace("Use the pipeline.", "A hand edit.")).unwrap();
    let sync = ok(a.call("sync", json!({})));
    assert_eq!(sync["conflicts"].as_array().unwrap().len(), 1, "{sync}");
    let after = ok(a.call("search", json!({})));
    assert!(
        ids(&after, "results")
            .iter()
            .any(|i| i.contains("conflict")),
        "{after}"
    );

    // pause-and-resume: a pause ends writes at the next call. Reads stay.
    personal(&w, "[limits]\npaused = true\n");
    let paused = a.call(
        "add_document",
        json!({"folder": "ops", "title": "While paused", "description": "D.", "body": "B."}),
    );
    assert_eq!(paused.code(), Some("paused"));
    ok(a.call("search", json!({"query": "pipeline"})));
    personal(&w, "[limits]\npaused = false\n");
    ok(a.call(
        "add_document",
        json!({"folder": "ops", "title": "After resume", "description": "D.", "body": "B."}),
    ));

    // trust-model: a switched-off tool is not listed and a direct call is refused.
    personal(&w, "[limits]\ntools_denied = [\"delete\"]\n");
    assert!(!a.tool_names().contains(&"delete_document".to_string()));
    let denied = a.call(
        "delete_document",
        json!({"id": "ops/after-resume", "base": "x"}),
    );
    assert_eq!(denied.code(), Some("access_denied"));
    personal(&w, "[limits]\naccess = \"read-only\"\n");
    assert!(!a.tool_names().contains(&"add_document".to_string()));
    assert_eq!(
        a.call(
            "add_document",
            json!({"folder": "ops", "title": "T", "description": "D.", "body": "B."})
        )
        .code(),
        Some("access_denied")
    );

    // The remote holds the whole history under the right names.
    let log = git(&w.remote, &["log", "--format=%an", "main"]);
    assert!(log.contains("Anna Example") && log.contains("Ben Example"));
}

#[test]
fn project_binding_refuses_another_memory_repository() {
    let w = World::empty();
    let mut a = client(&w, Person::A, &[]);
    ok(a.call("init", json!({"name": "Team KB"})));
    drop(a);
    let other = "123e4567-e89b-42d3-a456-426614174000";
    let mut c = McpClient::for_person(
        &w,
        Person::A,
        Spawn::args(&["--memory-repository-id", other]),
    );
    assert_eq!(c.call("search", json!({})).code(), Some("setup_problem"));
    let s = ok(c.call("status", json!({})));
    assert_eq!(s["healthy"], false);
}
