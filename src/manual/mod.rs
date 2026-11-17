//! Manual edits: documents that people changed outside waymark (see `specs/features/manual-edits`
//! and `specs/features/document-names`).
//!
//! Each pending edit is sorted into the write tool that would make the same change, and the rules
//! of that tool apply on top of `lint`: `add_document` for a new file, `update_document` for a changed
//! file, `delete_document` for a deleted file, and `move` for a renamed file with the same text.

pub mod pending;

use crate::config::Kind as ToolKind;
use crate::error::Result;
use crate::format::{self, Finding};
use crate::gitsync::{commit_message, Change, Tx, Verb};
use crate::lint;
use crate::mcp::tool::Env;
use crate::memrepo::MemRepo;
use crate::store::Store;
use crate::writes::linkfix::{retarget_body, retarget_properties, Remap};
use crate::writes::{self, titles, FileChange};
pub use pending::{Edit, Kind, Skip};
use std::collections::{HashMap, HashSet};

pub struct Blocked {
    pub id: String,
    pub path: String,
    pub findings: Vec<Finding>,
    /// For a blocked rename: the old path below the repository folder. It stays uncommitted too.
    pub from_rel: Option<String>,
}

/// An edit that passed, with what its commit holds.
pub struct Planned {
    pub edit: Edit,
    /// The write tool whose rules applied.
    pub tool: &'static str,
    pub title: String,
    /// Documents to write in the commit of the edit (for a move: the link commit), by ID.
    pub writes: Vec<FileChange>,
}

impl Planned {
    /// The word of the `Action:` line in the commit message.
    pub fn action(&self) -> &'static str {
        match self.edit.kind {
            Kind::New => "add",
            Kind::Changed => "update",
            Kind::Deleted => "delete",
            Kind::Moved => "move",
        }
    }
}

pub struct Outcome {
    /// The edits that passed the checks, in ID order.
    pub passing: Vec<Planned>,
    pub blocked: Vec<Blocked>,
    pub skipped: Vec<Skip>,
}

/// The write tool that makes the same change as an edit.
pub fn tool_of(kind: Kind) -> &'static str {
    match kind {
        Kind::New => "add_document",
        Kind::Changed => "update_document",
        Kind::Deleted => "delete_document",
        Kind::Moved => "move",
    }
}

/// The text of a document: the planned text if an earlier edit writes it, or the file.
fn current_text(planned: &[Planned], memrepo: &MemRepo, id: &str) -> Option<String> {
    for p in planned.iter().rev() {
        for c in &p.writes {
            if let FileChange::Write { id: cid, text, .. } = c {
                if cid == id {
                    return Some(text.clone());
                }
            }
        }
    }
    memrepo.read(id).ok()
}

/// Find the pending edits and check each one with `lint` and the rules of its write tool. For the
/// edits that pass, compute the files that their commits hold. Nothing is written or committed
/// here.
pub fn prepare(
    tx: &Tx,
    memrepo: &MemRepo,
    env: &Env,
    author: &str,
    today: &str,
) -> Result<Outcome> {
    let found = pending::find(tx.repo(), memrepo)?;
    if found.edits.is_empty() {
        return Ok(Outcome {
            passing: Vec::new(),
            blocked: Vec::new(),
            skipped: found.skipped,
        });
    }
    let checked: HashSet<String> = found
        .edits
        .iter()
        .filter(|e| e.kind != Kind::Deleted)
        .map(|e| e.id.clone())
        .collect();
    let report = if checked.is_empty() {
        None
    } else {
        let store = Store::load(memrepo);
        Some(lint::run(
            memrepo,
            &store,
            &|id| checked.contains(id),
            None,
        )?)
    };

    let disk_ids = memrepo.id_set();
    // The IDs as they are committed: the files on disk, without new files, with deleted files.
    let mut ids_now: HashSet<String> = disk_ids.clone();
    for e in &found.edits {
        match e.kind {
            Kind::New => {
                ids_now.remove(&e.id);
            }
            Kind::Deleted => {
                ids_now.insert(e.id.clone());
            }
            Kind::Moved => {
                ids_now.remove(&e.id);
                if let Some(from) = &e.from {
                    ids_now.insert(from.id.clone());
                }
            }
            Kind::Changed => {}
        }
    }
    // Documents with an edit that did not pass (yet) are not rewritten and do not count for titles.
    let mut waiting: HashSet<String> = found
        .edits
        .iter()
        .filter(|e| e.kind != Kind::Deleted)
        .map(|e| e.id.clone())
        .collect();
    let active = writes::active_titles(memrepo);

    let mut passing: Vec<Planned> = Vec::new();
    let mut blocked = Vec::new();
    // Deletes go last: then every other edit is decided, and a delete can check the committed text
    // of the documents whose edits are blocked.
    let mut edits = found.edits;
    edits.sort_by_key(|e| (e.kind == Kind::Deleted, e.id.clone()));
    for edit in edits {
        let tool = tool_of(edit.kind);
        let path = format!("{}.md", edit.id);
        let mut findings: Vec<Finding> = Vec::new();

        // The rules of the write tool: the tool is not switched off, and the folders limit.
        // `Read`: only `tools_denied` counts here. Read-only access and a pause are checked by the
        // callers, so a preview with read-only access does not report every tool as switched off.
        if env.effective.check_tool(tool, ToolKind::Read).is_err() {
            findings.push(Finding::error(
                &path,
                "tool-switched-off",
                format!("The tool `{tool}` is switched off, so this edit cannot be published."),
            ));
        }
        let mut places = vec![edit.id.clone()];
        if let Some(from) = &edit.from {
            places.push(from.id.clone());
        }
        for place in &places {
            if !env.effective.allows_folder(place) {
                findings.push(Finding::error(
                    &path,
                    "outside-folders",
                    format!("\"{place}\" is outside the allowed folders."),
                ));
            }
        }

        if edit.kind == Kind::Deleted {
            // A deleted document that another document still links to would leave a broken
            // link, so the deletion waits (see `write-documents`).
            let linked = crate::tools::delete::linked_from(memrepo, &edit.id, "");
            // A document whose edit is blocked keeps its committed text, which may still link.
            let mut waiting_ids: Vec<&String> = waiting.iter().collect();
            waiting_ids.sort();
            let blocked_linker = waiting_ids.into_iter().find(|w| {
                memrepo
                    .repo_relative(w)
                    .ok()
                    .and_then(|rel| tx.repo().show_at("HEAD", &rel).ok().flatten())
                    .is_some_and(|text| text_links_to(w, &text, &edit.id))
            });
            if let Some(w) = blocked_linker {
                findings.push(Finding::error(
                    &path,
                    "broken-link",
                    format!(
                        "The deleted document is still linked from {w}, whose edit is not published. Publish or fix that edit first."
                    ),
                ));
            }
            if !linked.is_empty() {
                let from: Vec<String> = linked
                    .iter()
                    .filter_map(|l| l["id"].as_str().map(str::to_string))
                    .collect::<std::collections::BTreeSet<_>>()
                    .into_iter()
                    .collect();
                findings.push(Finding::error(
                    &path,
                    "broken-link",
                    format!(
                        "The deleted document is still linked from {}. Remove or replace those links first.",
                        from.join(", ")
                    ),
                ));
            }
            if findings.iter().any(Finding::is_error) {
                blocked.push(Blocked {
                    id: edit.id,
                    path,
                    findings,
                    from_rel: None,
                });
                continue;
            }
            ids_now.remove(&edit.id);
            let title = head_title(tx, &edit).unwrap_or_else(|| edit.id.clone());
            passing.push(Planned {
                edit,
                tool,
                title,
                writes: Vec::new(),
            });
            continue;
        }

        findings.extend(
            report
                .as_ref()
                .map(|r| {
                    r.findings
                        .iter()
                        .filter(|f| f.path == path)
                        .cloned()
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
        );
        match edit.kind {
            Kind::Moved => {
                // The links of a renamed document are rewritten below, so a link that its new
                // place breaks is no problem.
                findings.retain(|f| f.rule != "broken-link" || f.is_error());
            }
            _ => {
                // A broken body link that the edit adds stops the publish, like in a write tool. A
                // link that was broken in the last commit stays a warning.
                let old_broken: Vec<String> = match edit.kind {
                    Kind::Changed => tx
                        .repo()
                        .show_at("HEAD", &edit.rel)
                        .ok()
                        .flatten()
                        .map(|old| {
                            format::check(&path, &old, &memrepo.settings.documents, Some(&disk_ids))
                                .into_iter()
                                .filter(|f| f.rule == "broken-link")
                                .map(|f| f.message)
                                .collect()
                        })
                        .unwrap_or_default(),
                    _ => Vec::new(),
                };
                for f in findings.iter_mut() {
                    if f.rule == "broken-link" && !f.is_error() && !old_broken.contains(&f.message)
                    {
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
            }
        }

        let text = tx.read(&edit.rel)?.unwrap_or_default();
        let doc = format::parse(&text).ok();

        // Titles: exact and near duplicates, like `add_document`, and like `update_document` when
        // the title changed.
        if let (Some(doc), Kind::New | Kind::Changed) = (&doc, edit.kind) {
            let title_changed = edit.kind == Kind::New
                || tx
                    .repo()
                    .show_at("HEAD", &edit.rel)
                    .ok()
                    .flatten()
                    .and_then(|old| format::parse(&old).ok())
                    .map(|old| {
                        old.frontmatter.get_text("title") != doc.frontmatter.get_text("title")
                    })
                    .unwrap_or(true);
            if let Some(title) = doc.frontmatter.get_text("title").filter(|_| title_changed) {
                let mut skip: Vec<&str> = waiting.iter().map(String::as_str).collect();
                skip.push(edit.id.as_str());
                let has_duplicate = findings.iter().any(|f| f.rule == "duplicate-title");
                for clash in titles::clashes(title, &active, &skip) {
                    match clash {
                        titles::Clash::Exact(other) if !has_duplicate => {
                            findings.push(Finding::error(
                                &path,
                                "duplicate-title",
                                format!("A document with this title exists: `{other}`."),
                            ))
                        }
                        titles::Clash::Near(other) => findings.push(Finding::error(
                            &path,
                            "near-duplicate-title",
                            format!("A document with a similar title exists: `{other}`."),
                        )),
                        _ => {}
                    }
                }
            }
        }

        if findings.iter().any(Finding::is_error) {
            blocked.push(Blocked {
                from_rel: edit.from.as_ref().map(|f| f.rel.clone()),
                id: edit.id,
                path,
                findings,
            });
            continue;
        }

        let mut writes_of_edit: Vec<FileChange> = Vec::new();
        let title = doc
            .as_ref()
            .and_then(|d| d.frontmatter.get_text("title").map(str::to_string))
            .unwrap_or_else(|| edit.id.clone());
        let skip_waiting = |i: &str| waiting.contains(i);

        match edit.kind {
            Kind::New | Kind::Changed => {
                if let Some(mut doc) = doc.clone() {
                    let mut changed = false;
                    if edit.kind == Kind::Changed {
                        // The author fields, as `update_document` sets them.
                        if doc.frontmatter.get_text("updated") != Some(today) {
                            doc.frontmatter.set_date("updated", today);
                            changed = true;
                        }
                        if doc.frontmatter.get_text("updated_by") != Some(author) {
                            doc.frontmatter.set_text("updated_by", author);
                            changed = true;
                        }
                    }
                    if memrepo.settings.verification && trust_resets(tx, &edit, &doc) {
                        doc.frontmatter.set_text("trust", "unverified");
                        doc.frontmatter.remove("verified_by");
                        doc.frontmatter.remove("verified");
                        changed = true;
                    }
                    if changed {
                        writes_of_edit.push(FileChange::Write {
                            id: edit.id.clone(),
                            text: doc.render(),
                            action: "update",
                        });
                    }
                }
                if edit.kind == Kind::New {
                    // Short wikilinks to a document with the same file name keep their target.
                    let old = overlay(&passing);
                    let mut planned_now: Vec<FileChange> = overlay(&passing);
                    let fixes = crate::writes::shortlinks::add_fixes_for_new_ids(
                        memrepo,
                        Some(env),
                        &ids_now,
                        std::slice::from_ref(&edit.id),
                        &skip_waiting,
                        today,
                        author,
                        &mut planned_now,
                    );
                    match fixes {
                        Ok(_) => {
                            take_fixes(&mut writes_of_edit, &old, planned_now);
                        }
                        Err(e) => {
                            blocked.push(Blocked {
                                id: edit.id.clone(),
                                path: path.clone(),
                                findings: vec![Finding::error(&path, "outside-folders", e.message)],
                                from_rel: None,
                            });
                            continue;
                        }
                    }
                }
            }
            Kind::Moved => {
                let from = edit.from.clone().expect("a move has a source");
                let moved: HashMap<String, String> =
                    [(from.id.clone(), edit.id.clone())].into_iter().collect();
                let remap = Remap::new(moved, &ids_now);
                let mut outside = None;
                let mut fixes = Vec::new();
                let mut ids: Vec<&String> = ids_now.iter().collect();
                ids.sort();
                for old_id in ids {
                    let new_id = remap.new_id(old_id).to_string();
                    if old_id != &from.id && waiting.contains(old_id) {
                        continue;
                    }
                    let source = if old_id == &from.id { &edit.id } else { old_id };
                    let Some(text) = current_text(&passing, memrepo, source) else {
                        continue;
                    };
                    let Ok(mut d) = format::parse(&text) else {
                        continue;
                    };
                    let in_props = retarget_properties(&mut d.frontmatter, &remap);
                    let (body, in_body) = retarget_body(old_id, &new_id, &d.body, &remap);
                    if in_props + in_body == 0 {
                        continue;
                    }
                    if !env.effective.allows_folder(old_id) || !env.effective.allows_folder(&new_id)
                    {
                        outside = Some(old_id.clone());
                        break;
                    }
                    d.body = body;
                    writes::touch(&mut d, today, author);
                    fixes.push(FileChange::Write {
                        id: new_id,
                        text: d.render(),
                        action: "update",
                    });
                }
                if let Some(o) = outside {
                    blocked.push(Blocked {
                        from_rel: Some(from.rel.clone()),
                        id: edit.id.clone(),
                        path: path.clone(),
                        findings: vec![Finding::error(
                            &path,
                            "outside-folders",
                            format!("The document \"{o}\" links to the renamed document and is outside the allowed folders."),
                        )],
                    });
                    continue;
                }
                writes_of_edit = fixes;
            }
            Kind::Deleted => unreachable!("handled above"),
        }

        // The edit passed: it counts for the next edits.
        waiting.remove(&edit.id);
        if let Some(from) = &edit.from {
            ids_now.remove(&from.id);
        }
        ids_now.insert(edit.id.clone());
        passing.push(Planned {
            edit,
            tool,
            title,
            writes: writes_of_edit,
        });
    }

    Ok(Outcome {
        passing,
        blocked,
        skipped: found.skipped,
    })
}

/// All planned writes of the edits so far, in order.
fn overlay(passing: &[Planned]) -> Vec<FileChange> {
    let mut out = Vec::new();
    for p in passing {
        for c in &p.writes {
            if let FileChange::Write { id, text, action } = c {
                out.push(FileChange::Write {
                    id: id.clone(),
                    text: text.clone(),
                    action,
                });
            }
        }
    }
    out
}

/// Take the result of the short-link fixes for the current edit. `old` is the overlay before the
/// fixes. A text that an earlier edit plans and that the fixes changed goes into the current edit
/// as a new write, so that each fix is in the commit of the edit that needs it. New entries (from
/// index `old.len()` on) go to the current edit too.
fn take_fixes(current: &mut Vec<FileChange>, old: &[FileChange], planned_now: Vec<FileChange>) {
    for (i, c) in planned_now.into_iter().enumerate() {
        let changed = match (old.get(i), &c) {
            (Some(FileChange::Write { text: a, .. }), FileChange::Write { text: b, .. }) => a != b,
            (None, _) => true,
            _ => false,
        };
        if changed {
            current.push(c);
        }
    }
}

/// With verification on: does the edit set the document to `unverified`? A new file always does.
/// A changed file does when it was verified and its title, description, or body changed.
fn trust_resets(tx: &Tx, edit: &Edit, doc: &format::Document) -> bool {
    let fm = &doc.frontmatter;
    match edit.kind {
        Kind::New => {
            fm.get_text("trust") != Some("unverified")
                || fm.get_text("verified_by").is_some()
                || fm.get_text("verified").is_some()
        }
        Kind::Changed => {
            if fm.get_text("trust") != Some("verified") {
                // Not verified, but with fields of a verification: remove them.
                return fm.get_text("verified_by").is_some() || fm.get_text("verified").is_some();
            }
            let Some(old) = tx
                .repo()
                .show_at("HEAD", &edit.rel)
                .ok()
                .flatten()
                .and_then(|t| format::parse(&t).ok())
            else {
                return true;
            };
            // Only `verify_document` sets a document to verified. A hand edit that verifies it, or
            // that changes who or when, is reset.
            let fields = ["trust", "verified_by", "verified"];
            let verified_by_hand = fields
                .iter()
                .any(|f| old.frontmatter.get_text(f) != fm.get_text(f));
            verified_by_hand
                || old.frontmatter.get_text("title") != fm.get_text("title")
                || old.frontmatter.get_text("description") != fm.get_text("description")
                || old.body != doc.body
        }
        _ => false,
    }
}

/// Does the text of document `from` link to the document `id`, in the body or in a property?
fn text_links_to(from: &str, text: &str, id: &str) -> bool {
    let Ok(mut doc) = format::parse(text) else {
        return false;
    };
    let short = id.rsplit('/').next().unwrap_or(id);
    let matches = |name: &str| name == id || name == short;
    let target = crate::writes::linkfix::Target {
        id,
        title: "",
        wiki_matches: &matches,
    };
    let (_, in_body) = crate::writes::linkfix::fix_body(&format!("{from}.md"), &doc.body, &target);
    in_body > 0 || crate::writes::linkfix::fix_properties(&mut doc.frontmatter, &target) > 0
}

/// The title of a deleted document, from the last commit.
fn head_title(tx: &Tx, edit: &Edit) -> Option<String> {
    tx.repo()
        .show_at("HEAD", &edit.rel)
        .ok()
        .flatten()
        .and_then(|t| format::parse(&t).ok())
        .and_then(|d| d.frontmatter.get_text("title").map(str::to_string))
}

/// Write and commit each planned edit: one commit for each edit, two for a move (the rename alone,
/// then the link fixes). Returns the commit of each edit (for a move: the last one), in order.
pub fn commit_planned(
    tx: &Tx,
    memrepo: &MemRepo,
    planned: &[Planned],
) -> Result<Vec<Option<String>>> {
    let mut out = Vec::new();
    for p in planned {
        let message = format!(
            "{}\n\nAction: {}",
            commit_message(Verb::Sync, &p.title),
            p.action()
        );
        let mut last = None;
        if let (Kind::Moved, Some(from)) = (p.edit.kind, &p.edit.from) {
            last = tx.repo().commit_change(&Change {
                paths: vec![from.rel.clone(), p.edit.rel.clone()],
                message: message.clone(),
            })?;
            tx.forget();
        }
        let mut paths = Vec::new();
        if p.edit.kind != Kind::Moved {
            paths.push(p.edit.rel.clone());
        }
        for c in &p.writes {
            if let FileChange::Write { id, text, .. } = c {
                let rel = memrepo.repo_relative(id)?;
                tx.write_file(&rel, text)?;
                if !paths.contains(&rel) {
                    paths.push(rel);
                }
            }
        }
        if !paths.is_empty() {
            let mut m = message;
            if let Some(h) = &last {
                m.push_str(&format!("\n\nMove: {h}"));
            }
            if let Some(h) = tx.repo().commit_change(&Change { paths, message: m })? {
                last = Some(h);
            }
            // Committed: a later failure must not put these files back.
            tx.forget();
        }
        out.push(last);
    }
    Ok(out)
}

/// Before another write: publish the passing edits, each in its own commit. Blocked edits stay. An
/// edit whose write tool needs approval stays too: it waits for `sync`, which asks.
/// Returns notices, and the paths of the edits that stay uncommitted.
pub fn commit_before_write(
    tx: &Tx,
    memrepo: &MemRepo,
    env: &Env,
    author: &str,
    today: &str,
) -> Result<(Vec<String>, Vec<String>)> {
    let outcome = prepare(tx, memrepo, env, author, today)?;
    // An edit whose write tool needs approval waits for `sync`. Every edit after it waits too: a
    // later edit may build on its link fixes, and a delete (deletes come last) may need its text.
    let mut publish = Vec::new();
    let mut held = Vec::new();
    for p in outcome.passing {
        if !held.is_empty() || env.effective.needs_approval(p.tool, ToolKind::Write) {
            held.push(p);
        } else {
            publish.push(p);
        }
    }
    commit_planned(tx, memrepo, &publish)?;
    // The published edits are committed. A later failure of the agent's write must not put their
    // files back.
    tx.forget();
    let mut notices = Vec::new();
    let n = outcome.blocked.len() + held.len();
    if n > 0 {
        notices.push(if n == 1 {
            "1 manual edit is not published. Run `sync`.".to_string()
        } else {
            format!("{n} manual edits are not published. Run `sync`.")
        });
    }
    // The paths of the edits that stay, below the repository folder.
    let mut rels: Vec<String> = outcome
        .blocked
        .iter()
        .filter_map(|b| memrepo.repo_relative(&b.id).ok())
        .collect();
    rels.extend(outcome.blocked.iter().filter_map(|b| b.from_rel.clone()));
    for p in &held {
        rels.push(p.edit.rel.clone());
        if let Some(from) = &p.edit.from {
            rels.push(from.rel.clone());
        }
    }
    Ok((notices, rels))
}
