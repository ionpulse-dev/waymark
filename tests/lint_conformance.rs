//! `lint` over the conformance fixtures of `standard/conformance/`.

use std::fs;
use std::path::{Path, PathBuf};
use waymark::config::{Effective, Limits};
use waymark::kb::{self, KnowledgeBase};
use waymark::lint;
use waymark::store::{Scope, Store};

fn conformance_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("standard/conformance")
}

fn copy_tree(from: &Path, to: &Path) {
    for e in fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let name = e.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        let dest = to.join(&name);
        if e.path().is_dir() {
            fs::create_dir_all(&dest).unwrap();
            copy_tree(&e.path(), &dest);
        } else {
            fs::copy(e.path(), dest).unwrap();
        }
    }
}

fn run_lint(dir: &Path) -> lint::Report {
    let kb = KnowledgeBase::open(dir).unwrap();
    let store = Store::load(&kb);
    let eff = Effective::from_layers(&[Limits::default()]);
    let scope = Scope::new(&eff, &[]);
    let limit = Scope::limit(&eff);
    lint::run(&kb, &store, &scope, &limit, None).unwrap()
}

#[test]
fn the_sample_vault_has_no_error() {
    let dir = tempfile::tempdir().unwrap();
    kb::init(dir.path(), "Vault", "vault").unwrap();
    let root = dir.path().join("knowledge");
    copy_tree(&conformance_dir().join("vault"), &root);
    // The vault excludes some files (for example its README).
    let exclude = fs::read_to_string(conformance_dir().join("vault/.mkbase/exclude")).unwrap();
    fs::write(root.join(".mkbase/exclude"), exclude).unwrap();
    let report = run_lint(dir.path());
    let errors: Vec<_> = report
        .findings
        .iter()
        .filter(|f| f.is_error())
        .map(|f| (f.path.clone(), f.rule, f.message.clone()))
        .collect();
    assert!(report.checked >= 15);
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn every_invalid_document_has_the_rule_of_its_file_name() {
    let mut count = 0;
    for e in fs::read_dir(conformance_dir().join("invalid")).unwrap() {
        let p = e.unwrap().path();
        if p.extension().and_then(|x| x.to_str()) != Some("md") {
            continue;
        }
        let rule = p.file_stem().unwrap().to_string_lossy().to_string();
        let dir = tempfile::tempdir().unwrap();
        kb::init(dir.path(), "Invalid", "invalid").unwrap();
        fs::copy(
            &p,
            dir.path().join("knowledge").join(p.file_name().unwrap()),
        )
        .unwrap();
        let report = run_lint(dir.path());
        assert!(
            report.findings.iter().any(|f| f.rule == rule),
            "{rule}: got {:?}",
            report.findings.iter().map(|f| f.rule).collect::<Vec<_>>()
        );
        count += 1;
    }
    assert!(count >= 13, "found {count} invalid documents");
}
