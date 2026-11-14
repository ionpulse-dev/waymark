//! Quality checks: `lint` over stored documents, and the checks that write tools share.

pub mod authors;
pub mod personal;
pub mod secrets;

use crate::config::Effective;
use crate::error::Result;
use crate::format::{self, Finding, Severity};
use crate::gitsync::Repo;
use crate::indexgen;
use crate::memrepo::MemRepo;
use crate::store::{Scope, Store};
use crate::writes::titles;
use regex::Regex;
use std::collections::{BTreeMap, HashSet};

pub struct Report {
    pub checked: usize,
    /// All findings, errors first, then by path and rule.
    pub findings: Vec<Finding>,
}

/// The rule names of the quality spec. The format module has some names of its own.
fn rule_name(rule: &'static str) -> &'static str {
    match rule {
        "not-utf8" => "bad-encoding",
        "bad-line-ends" => "bad-line-ending",
        "broken-property-link" => "broken-link",
        other => other,
    }
}

/// The `next` sentence for a rule.
pub fn next_for(rule: &str) -> &'static str {
    match rule {
        "missing-title" => "Add a title of at most 15 words.",
        "missing-description" => "Add a description of at most 50 words.",
        "title-too-long" => "Shorten the title.",
        "description-too-long" => "Shorten the description to at most 50 words.",
        "document-too-large" => "Split the document, or move details into a linked source.",
        "bad-file-name" => "Use lowercase letters, digits, and - in the name.",
        "broken-link" | "broken-property-link" => "Fix the link, or remove it.",
        "bad-author" => "Write the author as Name <email>.",
        "bad-date" | "updated-before-created" => "Use real dates as YYYY-MM-DD.",
        _ => "Fix the problems that are listed, then call again.",
    }
}

/// Findings that need only the text: secrets and personal data. `path` is the file path.
pub fn text_findings(path: &str, text: &str, custom: &[Regex]) -> Vec<Finding> {
    let mut out = Vec::new();
    for hit in secrets::scan(text, custom) {
        out.push(Finding::error(
            path,
            "secret",
            format!("Line {} looks like a secret ({}).", hit.line, hit.rule),
        ));
    }
    for line in personal::scan(text) {
        out.push(Finding::error(
            path,
            "personal-data",
            format!("Line {line} looks like an IBAN or a payment card number."),
        ));
    }
    out
}

fn sort(findings: &mut [Finding]) {
    findings.sort_by(|a, b| {
        let rank = |f: &Finding| if f.severity == Severity::Error { 0 } else { 1 };
        rank(a)
            .cmp(&rank(b))
            .then_with(|| a.path.cmp(&b.path))
            .then_with(|| a.rule.cmp(b.rule))
    });
}

/// Check the documents that `include` selects. `limit_scope` has only the `folders` limit: it decides what the
/// index files must contain. `repo` is used for the author checks.
pub fn run(
    memrepo: &MemRepo,
    store: &Store,
    include: &dyn Fn(&str) -> bool,
    limit_scope: &Scope,
    repo: Option<&Repo>,
) -> Result<Report> {
    let custom = secrets::compile(&memrepo.settings.extra_secret_patterns)?;
    let ids: HashSet<String> = memrepo.id_set();
    let limits = &memrepo.settings.documents;
    let mut findings: Vec<Finding> = Vec::new();
    let mut checked = 0;

    let mut all_ids: Vec<String> = ids.iter().cloned().collect();
    all_ids.sort();
    for id in all_ids.iter().filter(|i| include(i)) {
        checked += 1;
        let path = format!("{id}.md");
        match store.get(id) {
            Some(doc) => {
                findings.extend(format::check(&path, &doc.raw, limits, Some(&ids)));
                findings.extend(text_findings(&path, &doc.raw, &custom));
                for field in ["created_by", "updated_by"] {
                    if let Some(a) = doc.text(field) {
                        if let Some(why) = authors::not_allowed(a, &memrepo.settings.people) {
                            findings.push(Finding::error(
                                &path,
                                "author-not-allowed",
                                format!("The author in {field} is not allowed. {why}"),
                            ));
                        }
                    }
                }
            }
            None => {
                // A file that is not UTF-8 text.
                if let Ok(bytes) = memrepo.path_of(id).and_then(|p| {
                    std::fs::read(&p).map_err(|e| crate::error::Error::setup(e.to_string()))
                }) {
                    findings.extend(format::check_bytes(&path, &bytes, limits, Some(&ids)));
                }
            }
        }
    }

    // Duplicate titles: the older document keeps the title.
    let mut groups: BTreeMap<Vec<String>, Vec<(&str, &str)>> = BTreeMap::new();
    for d in store.docs().filter(|d| d.status() == "active") {
        let words = titles::words(d.title());
        if !words.is_empty() {
            groups
                .entry(words)
                .or_default()
                .push((d.text("created").unwrap_or(""), d.id.as_str()));
        }
    }
    for group in groups.values_mut().filter(|g| g.len() > 1) {
        group.sort();
        let first = group[0].1;
        for (_, id) in group.iter().skip(1) {
            if include(id) {
                findings.push(Finding::error(
                    &format!("{id}.md"),
                    "duplicate-title",
                    format!("Another document has the same title: `{first}`."),
                ));
            }
        }
    }

    // Authors against Git history. A file with pending edits or no history is skipped.
    if let Some(repo) = repo {
        let history = repo.path_authors()?;
        let dirty: HashSet<String> = repo.dirty_paths()?.into_iter().collect();
        for d in store.docs().filter(|d| include(&d.id)) {
            let Ok(rel) = memrepo.repo_relative(&d.id) else {
                continue;
            };
            if dirty.contains(&rel) {
                continue;
            }
            let path = format!("{}.md", d.id);
            for (field, map) in [
                ("created_by", &history.first),
                ("updated_by", &history.last),
            ] {
                let (Some(git_email), Some(doc_email)) =
                    (map.get(&rel), d.text(field).and_then(authors::email_of))
                else {
                    continue;
                };
                if *git_email != doc_email {
                    findings.push(Finding::warning(
                        &path,
                        "author-mismatch",
                        format!(
                            "{field} names {doc_email}, but Git history names {git_email} for this file."
                        ),
                    ));
                }
            }
        }
    }

    // Generated indexes that are out of date.
    for d in store
        .docs_with_index()
        .filter(|d| d.is_index() && include(&d.id))
    {
        let folder = indexgen::folder_of_index(&d.id);
        let expected = indexgen::expected_body(store, folder, limit_scope);
        if d.body.trim() != expected.trim() {
            findings.push(Finding::warning(
                &format!("{}.md", d.id),
                "index-stale",
                "This index differs from what the index tool would write.",
            ));
        }
    }

    for f in &mut findings {
        f.rule = rule_name(f.rule);
    }
    sort(&mut findings);
    Ok(Report { checked, findings })
}

/// The effective limits are not needed here, but a tool passes them to build scopes.
pub fn limit_scope(effective: &Effective) -> Scope<'_> {
    Scope::limit(effective)
}
