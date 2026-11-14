//! Conformance tests: the fixtures in `standard/conformance/`.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use waymark::format::{check, has_errors, parse, Limits, Severity};

fn conformance_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("standard/conformance")
}

/// All documents of a vault: (path below the root, text). Skips folders that start with `.`
/// and the files named in `.mkbase/exclude`.
fn vault_documents(root: &Path) -> Vec<(String, String)> {
    let exclude: Vec<String> = fs::read_to_string(root.join(".mkbase/exclude"))
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(String::from)
        .collect();
    let mut out = Vec::new();
    fn walk(root: &Path, dir: &Path, exclude: &[String], out: &mut Vec<(String, String)>) {
        let mut entries: Vec<_> = fs::read_dir(dir).unwrap().map(|e| e.unwrap()).collect();
        entries.sort_by_key(|e| e.file_name());
        for e in entries {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }
            let p = e.path();
            let rel = p.strip_prefix(root).unwrap().to_string_lossy().to_string();
            if exclude
                .iter()
                .any(|x| rel == *x || rel.starts_with(&format!("{x}/")))
            {
                continue;
            }
            if p.is_dir() {
                walk(root, &p, exclude, out);
            } else if rel.ends_with(".md") {
                out.push((rel, fs::read_to_string(&p).unwrap()));
            }
        }
    }
    walk(root, root, &exclude, &mut out);
    out
}

#[test]
fn every_vault_document_passes_with_no_error() {
    let root = conformance_dir().join("vault");
    let docs = vault_documents(&root);
    assert!(docs.len() >= 15, "the sample vault has documents");
    let ids: HashSet<String> = docs
        .iter()
        .map(|(p, _)| p.trim_end_matches(".md").to_string())
        .collect();
    for (path, text) in &docs {
        let findings = check(path, text, &Limits::default(), Some(&ids));
        let errors: Vec<_> = findings.iter().filter(|f| f.is_error()).collect();
        assert!(errors.is_empty(), "{path}: {errors:?}");
    }
}

#[test]
fn every_invalid_document_fails_with_the_rule_of_its_file_name() {
    let dir = conformance_dir().join("invalid");
    let mut count = 0;
    for e in fs::read_dir(&dir).unwrap() {
        let p = e.unwrap().path();
        if p.extension().and_then(|x| x.to_str()) != Some("md") {
            continue;
        }
        let rule = p.file_stem().unwrap().to_string_lossy().to_string();
        let text = fs::read_to_string(&p).unwrap();
        let path = p.file_name().unwrap().to_string_lossy().to_string();
        // No other document exists, so every link target is missing.
        let ids: HashSet<String> = HashSet::new();
        let findings = check(&path, &text, &Limits::default(), Some(&ids));
        assert!(
            findings.iter().any(|f| f.rule == rule),
            "{path}: expected a finding for rule {rule}, got {:?}",
            findings.iter().map(|f| f.rule).collect::<Vec<_>>()
        );
        count += 1;
    }
    assert!(count >= 13, "found {count} invalid documents");
}

#[test]
fn a_vault_document_renders_back_to_itself() {
    let root = conformance_dir().join("vault");
    for (path, text) in vault_documents(&root) {
        let doc = parse(&text).unwrap_or_else(|e| panic!("{path}: {e:?}"));
        assert_eq!(doc.render(), text, "{path}");
    }
}

const BASE: &str = "---\ntitle: A title\ndescription: A description\nstatus: active\ncreated: 2026-11-08\nupdated: 2026-11-08\ncreated_by: \"Jane Doe <jane@example.org>\"\nupdated_by: \"Jane Doe <jane@example.org>\"\n---\n";

#[test]
fn the_hard_cap_is_exact() {
    let limits = Limits::default();
    let fill = |total: usize| {
        let mut t = BASE.to_string();
        let n = total - t.len();
        t.push_str(&"a".repeat(n));
        t
    };
    let ok = fill(limits.max_bytes);
    assert_eq!(ok.len(), 102_400);
    assert!(!has_errors(&check("a.md", &ok, &limits, None)));
    let big = fill(limits.max_bytes + 1);
    let f = check("a.md", &big, &limits, None);
    let cap = f
        .iter()
        .find(|f| f.rule == "document-too-large")
        .expect("cap finding");
    assert!(cap.message.contains("Split"), "{}", cap.message);
    assert_eq!(cap.severity, Severity::Error);
}

#[test]
fn all_findings_are_returned_not_only_the_first() {
    let t = "---\ntitle: 123\nstatus: draft\ncreated: 08.11.2026\n---\nBody\n";
    let rules: Vec<&str> = check("Bad_Name.md", t, &Limits::default(), None)
        .iter()
        .map(|f| f.rule)
        .collect();
    for expected in [
        "bad-file-name",
        "bad-type",
        "bad-status",
        "bad-date",
        "missing-description",
        "missing-updated",
        "missing-created-by",
        "missing-updated-by",
    ] {
        assert!(rules.contains(&expected), "missing {expected} in {rules:?}");
    }
}

#[test]
fn link_severities() {
    let doc = BASE.replacen(
        "updated_by: \"Jane Doe <jane@example.org>\"\n",
        "updated_by: \"Jane Doe <jane@example.org>\"\nrelated:\n  - \"[[gone/doc]]\"\n",
        1,
    ) + "See [x](missing.md).\n";
    let ids: HashSet<String> = HashSet::new();
    let f = check("a.md", &doc, &Limits::default(), Some(&ids));
    let prop = f.iter().find(|f| f.rule == "broken-property-link").unwrap();
    let body = f.iter().find(|f| f.rule == "broken-link").unwrap();
    assert_eq!(prop.severity, Severity::Error);
    assert_eq!(body.severity, Severity::Warning);
}

#[test]
fn a_nested_property_fails() {
    let doc = BASE.replacen("---\n", "---\nmeta:\n  owner: x\n", 1);
    let f = check("a.md", &doc, &Limits::default(), None);
    assert!(f.iter().any(|f| f.rule == "nested-property"));
}

#[test]
fn crlf_and_invalid_utf8_fail() {
    let crlf = BASE.replace('\n', "\r\n");
    assert!(check("a.md", &crlf, &Limits::default(), None)
        .iter()
        .any(|f| f.rule == "bad-line-ends"));
    let f = waymark::format::check_bytes("a.md", &[0xff, 0xfe, 0x41], &Limits::default(), None);
    assert!(f.iter().any(|f| f.rule == "not-utf8"));
}
