//! The conformance fixtures of `standard/conformance/`, run against the real binary through the
//! `lint` tool (task G2 of the plan).

mod common;
use common::{McpClient, Person, Spawn, World};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn conformance_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("standard/conformance")
}

fn copy_tree(from: &Path, to: &Path) {
    for e in fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let name = e.file_name().to_string_lossy().to_string();
        if name == ".DS_Store" {
            continue;
        }
        let dest = to.join(&name);
        if e.path().is_dir() {
            fs::create_dir_all(&dest).unwrap();
            copy_tree(&e.path(), &dest);
        } else if name != "exclude" {
            fs::copy(e.path(), dest).unwrap();
        }
    }
}

fn lint(w: &World) -> Value {
    let mut c = McpClient::for_person(w, Person::A, Spawn::default());
    let r = c.call("lint", json!({}));
    assert!(!r.is_error, "{}", r.json);
    r.json
}

#[test]
fn the_sample_vault_has_no_error_in_the_binary() {
    let w = World::new();
    let root = w.a.join("knowledge");
    copy_tree(&conformance_dir().join("vault"), &root);
    fs::copy(
        conformance_dir().join("vault/.mkbase/exclude"),
        root.join(".mkbase/exclude"),
    )
    .unwrap();
    let r = lint(&w);
    let errors: Vec<&Value> = r["findings"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| f["severity"] == "error")
        .collect();
    assert!(errors.is_empty(), "{errors:?}");
    assert!(r["checked"].as_u64().unwrap_or(0) >= 15, "{r}");
}

#[test]
fn every_invalid_document_has_the_rule_of_its_file_name_in_the_binary() {
    let mut count = 0;
    for e in fs::read_dir(conformance_dir().join("invalid")).unwrap() {
        let p = e.unwrap().path();
        if p.extension().and_then(|x| x.to_str()) != Some("md") {
            continue;
        }
        let rule = p.file_stem().unwrap().to_string_lossy().to_string();
        let w = World::new();
        fs::copy(&p, w.a.join("knowledge").join(p.file_name().unwrap())).unwrap();
        let r = lint(&w);
        let rules: Vec<String> = r["findings"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| f["rule"].as_str().unwrap().to_string())
            .collect();
        assert!(rules.contains(&rule), "{rule}: got {rules:?}");
        count += 1;
    }
    assert!(count >= 13, "found {count} invalid documents");
}
