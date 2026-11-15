//! IDs that differ only in case (see `specs/foundation/linux-support/test.md`).
//!
//! macOS file systems are usually case-insensitive and Linux ones are case-sensitive. The Waymark
//! Memory Standard (section 6) allows only lowercase file names, so an ID with capitals names no
//! document on any system. These tests show that the behavior is the same on both.

mod common;
use common::{McpClient, Person, Spawn, World};
use serde_json::json;
use std::path::Path;

/// Tell whether `dir` holds a case-sensitive file system, by making two names that differ in case.
fn is_case_sensitive(dir: &Path) -> bool {
    let probe = dir.join("CaseProbe");
    std::fs::write(&probe, "x").unwrap();
    let sensitive = !dir.join("caseprobe").exists();
    std::fs::remove_file(&probe).unwrap();
    sensitive
}

fn client(w: &World) -> McpClient {
    McpClient::for_person(w, Person::A, Spawn::default())
}

#[test]
fn a_capital_id_names_no_document_on_any_system() {
    let w = World::new();
    let mut c = client(&w);
    let r = c.call(
        "add_document",
        json!({"folder": "notes", "title": "Plan", "description": "About the plan.", "body": "Text."}),
    );
    assert!(!r.is_error, "{}", r.json);
    assert_eq!(r.json["id"], "notes/plan");

    let ok = c.call("show", json!({"id": "notes/plan"}));
    assert!(!ok.is_error, "{}", ok.json);
    for id in ["Notes/Plan", "notes/Plan", "NOTES/PLAN"] {
        let r = c.call("show", json!({"id": id}));
        assert!(r.is_error, "{id} must not open notes/plan: {}", r.json);
    }
}

#[test]
fn the_test_folder_has_the_file_system_that_the_test_assumes() {
    // The temp folder is where all tests make their repositories. In the Linux container it is on the
    // container file system, which is case-sensitive. On a Mac it is usually case-insensitive. The
    // tests above must pass on both. This test only reports which one ran, and checks that the
    // result of the probe is stable.
    let dir = tempfile::tempdir().unwrap();
    let first = is_case_sensitive(dir.path());
    assert_eq!(first, is_case_sensitive(dir.path()));
    eprintln!(
        "file system of {}: case-{}",
        dir.path().display(),
        if first { "sensitive" } else { "insensitive" }
    );
    if std::env::var_os("WAYMARK_EXPECT_CASE_SENSITIVE").is_some() {
        assert!(
            first,
            "the tests ran on a case-insensitive folder, but a case-sensitive one was expected (the shared Mac folder?)"
        );
    }
}
