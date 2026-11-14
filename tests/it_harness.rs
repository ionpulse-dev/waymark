//! Tests of the harness itself (see `specs/foundation/integration-testing`).

mod common;
use common::git::git;
use common::memrepo::commit_doc;
use common::{McpClient, Person, Spawn, World};

#[test]
fn world_has_remote_and_two_clones() {
    let w = World::new();
    assert!(w.remote.join("HEAD").exists());
    assert!(w.a.join(".git").exists() && w.b.join(".git").exists());
    assert_ne!(
        World::author(Person::A).email,
        World::author(Person::B).email
    );
}

#[test]
fn a_commit_of_a_shows_author_a_in_b() {
    let w = World::new();
    let repo = w.repo(Person::A);
    commit_doc(
        &w.a,
        "ops/deploy",
        "Deploys go through the pipeline",
        "Body.",
    );
    git(&w.b, &["pull", "-q"]);
    assert!(w.b.join("agent-memory/ops/deploy.md").exists());
    let _ = repo;
}

#[test]
fn world_isolates_environment() {
    let _w = World::new();
    assert_eq!(std::env::var("GIT_CONFIG_GLOBAL").unwrap(), "/dev/null");
    let home = std::env::var("HOME").unwrap();
    assert!(home.contains("waymark-test-home"), "HOME is {home}");
}

#[test]
fn client_lists_tools() {
    let w = World::new();
    let mut c = McpClient::for_person(&w, Person::A, Spawn::default());
    assert!(c.tool_names().contains(&"status".to_string()));
    assert_eq!(
        c.initialize_result["serverInfo"]["name"].as_str(),
        Some("waymark")
    );
}

#[test]
fn worlds_run_in_parallel() {
    let handles: Vec<_> = (0..8)
        .map(|i| {
            std::thread::spawn(move || {
                let w = World::new();
                commit_doc(
                    &w.a,
                    &format!("ops/doc-{i}"),
                    &format!("Document {i}"),
                    "Body.",
                );
                let mut c = McpClient::for_person(&w, Person::B, Spawn::default());
                let r = c.call("status", serde_json::json!({}));
                assert!(!r.is_error);
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
}

#[test]
fn failed_test_keeps_folder() {
    let root = std::panic::catch_unwind(|| {
        let w = World::new();
        let root = w.root.clone();
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _inner = w;
            panic!("planned failure");
        }));
        root
    })
    .unwrap();
    assert!(root.exists(), "the folder of a failed test must stay");
    let _ = std::fs::remove_dir_all(root);
}
