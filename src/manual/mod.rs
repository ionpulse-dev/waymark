//! Manual edits: documents that people changed outside waymark (see `specs/features/manual-edits`).

pub mod pending;

use crate::config::Effective;
use crate::error::Result;
use crate::format::{self, Finding};
use crate::gitsync::{commit_message, Change, Tx, Verb};
use crate::kb::KnowledgeBase;
use crate::lint;
use crate::store::{Scope, Store};
pub use pending::{Edit, Kind, Skip};
use std::collections::HashSet;

pub struct Blocked {
    pub id: String,
    pub path: String,
    pub findings: Vec<Finding>,
}

pub struct Outcome {
    /// The edits that passed the checks.
    pub passing: Vec<Edit>,
    pub blocked: Vec<Blocked>,
    pub skipped: Vec<Skip>,
    /// The commit to make for the passing edits, if there are any.
    pub change: Option<Change>,
}

/// Find the pending edits, check them with `lint`, and set `updated` and `updated_by` on the
/// changed documents that passed. Writes only those fields. Nothing is committed here.
pub fn prepare(
    tx: &Tx,
    kb: &KnowledgeBase,
    effective: &Effective,
    author: &str,
    today: &str,
    write_files: bool,
) -> Result<Outcome> {
    let found = pending::find(tx.repo(), kb)?;
    let ids: HashSet<String> = found
        .edits
        .iter()
        .filter(|e| e.kind != Kind::Deleted)
        .map(|e| e.id.clone())
        .collect();
    let report = if ids.is_empty() {
        None
    } else {
        let store = Store::load(kb);
        let limit = Scope::limit(effective);
        Some(lint::run(kb, &store, &|id| ids.contains(id), &limit, None)?)
    };

    let all_ids = kb.id_set();
    let mut passing = Vec::new();
    let mut blocked = Vec::new();
    for edit in found.edits {
        if edit.kind == Kind::Deleted {
            // A deleted document that another document still links to would leave a broken
            // link, so the deletion waits (see `write-documents`). Generated index files do not count.
            let linked = crate::tools::delete::linked_from(kb, &edit.id, "");
            if linked.is_empty() {
                passing.push(edit);
            } else {
                let path = format!("{}.md", edit.id);
                let from: Vec<String> = linked
                    .iter()
                    .filter_map(|l| l["id"].as_str().map(str::to_string))
                    .collect::<std::collections::BTreeSet<_>>()
                    .into_iter()
                    .collect();
                let message = format!(
                    "The deleted document is still linked from {}. Remove or replace those links first.",
                    from.join(", ")
                );
                blocked.push(Blocked {
                    id: edit.id,
                    findings: vec![Finding::error(&path, "broken-link", message)],
                    path,
                });
            }
            continue;
        }
        let path = format!("{}.md", edit.id);
        let mut findings: Vec<Finding> = report
            .as_ref()
            .map(|r| {
                r.findings
                    .iter()
                    .filter(|f| f.path == path)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();
        // A broken body link that the edit adds stops the publish, like in a write tool. A link that
        // was broken in the last commit stays a warning.
        let old_broken: Vec<String> = match edit.kind {
            Kind::Changed => tx
                .repo()
                .show_at("HEAD", &edit.rel)
                .ok()
                .flatten()
                .map(|old| {
                    format::check(&path, &old, &kb.settings.documents, Some(&all_ids))
                        .into_iter()
                        .filter(|f| f.rule == "broken-link")
                        .map(|f| f.message)
                        .collect()
                })
                .unwrap_or_default(),
            _ => Vec::new(),
        };
        for f in findings.iter_mut() {
            if f.rule == "broken-link" && !f.is_error() && !old_broken.contains(&f.message) {
                *f = Finding::error(
                    &path,
                    "broken-link",
                    format!(
                        "{} Write the linked document first, or remove the link.",
                        f.message
                    ),
                );
            }
        }
        if findings.iter().any(Finding::is_error) {
            blocked.push(Blocked {
                id: edit.id,
                path,
                findings,
            });
        } else {
            passing.push(edit);
        }
    }

    if write_files {
        for edit in passing.iter().filter(|e| e.kind == Kind::Changed) {
            let Some(text) = tx.read(&edit.rel)? else {
                continue;
            };
            let Ok(mut doc) = format::parse(&text) else {
                continue;
            };
            let fm = &mut doc.frontmatter;
            let mut changed = false;
            if fm.get_text("updated") != Some(today) {
                fm.set_date("updated", today);
                changed = true;
            }
            if fm.get_text("updated_by") != Some(author) {
                fm.set_text("updated_by", author);
                changed = true;
            }
            if changed {
                tx.write_file(&edit.rel, &doc.render())?;
            }
        }
    }

    let change = if passing.is_empty() {
        None
    } else {
        let title = match passing.as_slice() {
            [one] => title_of(tx, one),
            many => format!("{} manual edits", many.len()),
        };
        Some(Change {
            paths: passing.iter().map(|e| e.rel.clone()).collect(),
            message: commit_message(Verb::Sync, &title),
        })
    };
    Ok(Outcome {
        passing,
        blocked,
        skipped: found.skipped,
        change,
    })
}

/// The title of the document of an edit. A deleted document has its title in the last commit.
fn title_of(tx: &Tx, edit: &Edit) -> String {
    let text = match edit.kind {
        Kind::Deleted => tx.repo().show_at("HEAD", &edit.rel).ok().flatten(),
        _ => tx.read(&edit.rel).ok().flatten(),
    };
    text.and_then(|t| format::parse(&t).ok())
        .and_then(|d| d.frontmatter.get_text("title").map(str::to_string))
        .unwrap_or_else(|| edit.id.clone())
}

/// Before another write: commit the passing edits in their own commit. Blocked edits stay.
/// Returns a notice if some edits are not published, and the paths that stay uncommitted.
pub fn commit_before_write(
    tx: &Tx,
    kb: &KnowledgeBase,
    effective: &Effective,
    author: &str,
    today: &str,
) -> Result<(Vec<String>, Vec<String>)> {
    let outcome = prepare(tx, kb, effective, author, today, true)?;
    if let Some(change) = &outcome.change {
        tx.repo().commit_change(change)?;
    }
    let mut notices = Vec::new();
    if !outcome.blocked.is_empty() {
        let n = outcome.blocked.len();
        notices.push(if n == 1 {
            "1 manual edit is not published. Run `sync`.".to_string()
        } else {
            format!("{n} manual edits are not published. Run `sync`.")
        });
    }
    // The paths of blocked edits below the repository folder.
    let rels: Vec<String> = outcome
        .blocked
        .iter()
        .filter_map(|b| kb.repo_relative(&b.id).ok())
        .collect();
    Ok((notices, rels))
}
