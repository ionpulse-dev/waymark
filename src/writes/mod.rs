//! Shared parts of the write tools (see `specs/features/write-documents`).
//!
//! A tool builds a `Plan`: the files to write, remove, or keep, the commit message, and the
//! data for the result. `execute` runs the plan through `Repo::write`, which holds the lock,
//! brings the clone up to date, commits under the author, and pushes.

pub mod linkfix;
pub mod slug;
pub mod titles;

use crate::config::Kind;
use crate::error::{Code, Error, Result};
use crate::format::{self, Document};
use crate::gitsync::{commit_message, Change, Tx, Verb};
use crate::kb::KnowledgeBase;
use crate::mcp::context::Context;
use crate::mcp::result::ToolOutput;
use crate::mcp::tool::Env;
use serde_json::{json, Map, Value};
use std::collections::HashSet;

// ---- reading arguments (the schema has checked the types) ----

pub fn get_str<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key).and_then(Value::as_str)
}

pub fn get_bool(args: &Value, key: &str) -> bool {
    args.get(key).and_then(Value::as_bool).unwrap_or(false)
}

pub fn get_list(args: &Value, key: &str) -> Option<Vec<String>> {
    args.get(key).and_then(Value::as_array).map(|a| {
        a.iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect()
    })
}

// ---- small helpers ----

/// Today's date on this machine, as `YYYY-MM-DD`.
pub fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

/// The author in the form `Name <email>`.
pub fn author_string(ctx: &Context) -> Result<String> {
    let a = ctx.checked_author()?;
    Ok(format!("{} <{}>", a.name, a.email))
}

/// A body as stored: LF line ends, one empty line after the frontmatter, one final newline.
pub fn make_body(text: &str) -> String {
    let s = text.replace("\r\n", "\n").replace('\r', "\n");
    let t = s.trim_matches('\n');
    if t.is_empty() {
        String::new()
    } else {
        format!("\n{t}\n")
    }
}

/// Set `updated` and `updated_by`.
pub fn touch(doc: &mut Document, today: &str, author: &str) {
    doc.frontmatter.set_date("updated", today);
    doc.frontmatter.set_text("updated_by", author);
}

/// A property link in the full-path form.
pub fn wikilink(id: &str) -> String {
    let id = id.trim();
    if id.starts_with("[[") {
        id.to_string()
    } else {
        format!("[[{id}]]")
    }
}

/// A call outside the `folders` limit sees no document: `not_found`.
pub fn require_visible(env: &Env, id: &str) -> Result<()> {
    if env.effective.allows_folder(id) {
        Ok(())
    } else {
        Err(not_found(id))
    }
}

pub fn not_found(id: &str) -> Error {
    Error::not_found(format!("There is no document with the ID \"{id}\"."))
        .with_next("Use search or list to find the ID.")
}

/// Read and parse a document inside a write. `not_found` if the file does not exist.
pub fn read_doc(tx: &Tx, kb: &KnowledgeBase, id: &str) -> Result<(String, Document)> {
    let rel = kb.repo_relative(id)?;
    let text = tx.read(&rel)?.ok_or_else(|| not_found(id))?;
    // A document with a manual edit that is not published cannot be changed (see `manual-edits`).
    if tx.repo().dirty_paths()?.contains(&rel) {
        return Err(Error::new(
            Code::StaleBase,
            format!("The document \"{id}\" has manual edits that are not published."),
        )
        .with_next("This document has manual edits that are not published. Run `sync`."));
    }
    let doc = format::parse(&text).map_err(|e| {
        Error::validation(format!("The document \"{id}\" cannot be read: {e:?}."))
            .with_next("Fix the file by hand, then try again.")
    })?;
    Ok((rel, doc))
}

/// The ID and title of every active document.
pub fn active_titles(kb: &KnowledgeBase) -> Vec<(String, String)> {
    kb.documents()
        .into_iter()
        .filter_map(|d| {
            let text = kb.read(&d.id).ok()?;
            let doc = format::parse(&text).ok()?;
            let fm = &doc.frontmatter;
            if fm.get_text("status") == Some("active") {
                Some((d.id, fm.get_text("title")?.to_string()))
            } else {
                None
            }
        })
        .collect()
}

// ---- checks ----

/// The findings of a write. Every finding has a line and a `next` hint.
#[derive(Default)]
pub struct Problems {
    items: Vec<(String, String)>,
}

impl Problems {
    pub fn add(&mut self, message: impl Into<String>, next: impl Into<String>) {
        self.items.push((message.into(), next.into()));
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn into_error(self) -> Error {
        let message = self
            .items
            .iter()
            .map(|(m, _)| m.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let mut nexts: Vec<&str> = Vec::new();
        for (_, n) in &self.items {
            if !nexts.contains(&n.as_str()) {
                nexts.push(n);
            }
        }
        Error::new(Code::ValidationFailed, message).with_next(nexts.join(" "))
    }
}

/// What to check about the title of a document.
pub struct TitleCheck<'a> {
    /// IDs that may share the title: the document itself, or the one that it replaces.
    pub skip: &'a [&'a str],
}

/// Which broken body links stop a write (see `specs/features/write-documents`).
#[derive(Clone, Copy)]
pub enum NewLinks<'a> {
    /// The write does not change the body: a broken body link stays a notice.
    Ignore,
    /// A new document: every broken body link is a problem.
    All,
    /// A changed document: only a broken body link that the old text did not have is a problem.
    Since(&'a str),
}

/// Check the full text of a document as it will be written.
/// `ids_after` is the set of document IDs after the write. `scan_fields` are the texts to scan for
/// secrets and personal data. Warnings go to `notices`.
#[allow(clippy::too_many_arguments)]
pub fn check_document(
    kb: &KnowledgeBase,
    id: &str,
    text: &str,
    ids_after: &HashSet<String>,
    scan_fields: &[(&str, String)],
    title_check: Option<TitleCheck>,
    problems: &mut Problems,
    notices: &mut Vec<String>,
) -> Result<()> {
    check_document_links(
        kb,
        id,
        text,
        ids_after,
        scan_fields,
        title_check,
        NewLinks::Ignore,
        problems,
        notices,
    )
}

/// `check_document`, with the rule for broken body links of the write.
#[allow(clippy::too_many_arguments)]
pub fn check_document_links(
    kb: &KnowledgeBase,
    id: &str,
    text: &str,
    ids_after: &HashSet<String>,
    scan_fields: &[(&str, String)],
    title_check: Option<TitleCheck>,
    new_links: NewLinks,
    problems: &mut Problems,
    notices: &mut Vec<String>,
) -> Result<()> {
    let path = format!("{id}.md");
    let custom = crate::lint::secrets::compile(&kb.settings.extra_secret_patterns)?;
    let old_broken: Vec<String> = match new_links {
        NewLinks::Since(old) => format::check(&path, old, &kb.settings.documents, Some(ids_after))
            .into_iter()
            .filter(|f| f.rule == "broken-link")
            .map(|f| f.message)
            .collect(),
        _ => Vec::new(),
    };
    for f in format::check(&path, text, &kb.settings.documents, Some(ids_after)) {
        if f.is_error() {
            problems.add(f.message, crate::lint::next_for(f.rule));
        } else if f.rule == "broken-link"
            && match new_links {
                NewLinks::Ignore => false,
                NewLinks::All => true,
                NewLinks::Since(_) => !old_broken.contains(&f.message),
            }
        {
            problems.add(
                f.message,
                "Write the linked document first, or remove the link now and add it back after.",
            );
        } else {
            notices.push(f.message);
        }
    }
    for (field, value) in scan_fields {
        for hit in crate::lint::secrets::scan(value, &custom) {
            problems.add(
                format!(
                    "The {field} looks like it contains a secret ({}, line {}).",
                    hit.rule, hit.line
                ),
                "Remove the secret, or replace it with a placeholder.",
            );
        }
        for line in crate::lint::personal::scan(value) {
            problems.add(
                format!("The {field} looks like it contains an IBAN or a payment card number (line {line})."),
                "Remove the number.",
            );
        }
    }
    if let Ok(doc) = format::parse(text) {
        for field in ["created_by", "updated_by"] {
            if let Some(a) = doc.frontmatter.get_text(field) {
                if let Some(why) = crate::lint::authors::not_allowed(a, &kb.settings.people) {
                    problems.add(
                        format!("The author in {field} is not allowed. {why}"),
                        "Set another author email in your Git settings.",
                    );
                }
            }
        }
        if let (Some(tc), Some(title)) = (title_check, doc.frontmatter.get_text("title")) {
            for clash in titles::clashes(title, &active_titles(kb), tc.skip) {
                match clash {
                    titles::Clash::Exact(other) => problems.add(
                        format!("A document with this title exists: `{other}`."),
                        "Update that document instead, or choose another title.",
                    ),
                    titles::Clash::Near(other) => problems.add(
                        format!("A document with a similar title exists: `{other}`."),
                        "Update that document instead, or choose another title.",
                    ),
                }
            }
        }
    }
    Ok(())
}

// ---- the plan and its execution ----

pub enum FileChange {
    Write {
        id: String,
        text: String,
        /// `create`, `update`, or `deprecate`.
        action: &'static str,
    },
    Remove {
        id: String,
    },
    /// Part of the commit, but not changed. Used when a call finds nothing to change.
    Keep {
        id: String,
    },
    /// A file that is not a document, such as `.gitkeep`. `path` is below the root.
    WriteFile {
        path: String,
        text: String,
        action: &'static str,
    },
    RemoveFile {
        path: String,
    },
}

/// A first commit that holds only moves (see `move-documents`). It is made before `Plan::changes`.
pub struct FirstCommit {
    /// Paths below the repository folder: (from, to), for every file that moves.
    pub moves: Vec<(String, String)>,
    pub title: String,
    /// After the commit, the history of each new path must reach the old commit: (new path, old commit).
    pub history: Vec<(String, String)>,
}

pub struct Plan {
    pub first: Option<FirstCommit>,
    /// Commits to revert (newest first) in the working tree, before the commit (see `undo`).
    pub revert: Option<Vec<String>>,
    pub changes: Vec<FileChange>,
    pub verb: Verb,
    pub title: String,
    /// Lines after the subject of the commit message, for example `Reason: ...`.
    pub commit_body: Option<String>,
    /// What an approval prompt shows (see `write-approval`).
    pub summary: String,
    pub message: String,
    /// The fields of the tool's result.
    pub data: Map<String, Value>,
    /// The full text that `add` and `update_document` would write (for a preview).
    pub text: Option<String>,
    pub notices: Vec<String>,
}

/// The approval step of the write flow (see `write-approval`). Returns `Ok` after a yes, or when no
/// rule applies. It asks through the client, fails closed, and reads the limits again after a yes,
/// so that a pause or a read-only setting during the wait ends the write.
pub(crate) fn approve(
    ctx: &Context,
    env: &Env,
    kb: &KnowledgeBase,
    tool: &str,
    kind: Kind,
    summary: &str,
    paths: &[String],
) -> Result<bool> {
    approve_named(
        ctx,
        env,
        &kb.settings.name,
        Some(kb),
        tool,
        kind,
        summary,
        paths,
    )
}

/// `approve` with a name for the question, for a call before the knowledge base exists (`init`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn approve_named(
    ctx: &Context,
    env: &Env,
    kb_name: &str,
    kb: Option<&KnowledgeBase>,
    tool: &str,
    kind: Kind,
    summary: &str,
    paths: &[String],
) -> Result<bool> {
    if !env.effective.needs_approval(tool, kind) {
        return Ok(false);
    }
    if env.effective.approval_via == crate::config::ApprovalVia::Console {
        return Err(Error::new(
            Code::ApprovalUnavailable,
            "The approval channel `console` is not available in local mode.",
        )
        .with_next("Set `via` to `elicitation` where it was set."));
    }
    let author = author_string(ctx)?;
    let message = crate::mcp::approval_text::render(&crate::mcp::approval_text::Request {
        tool,
        kb_name,
        author: &author,
        summary,
        paths,
    });
    let channel = ctx.channel.lock().unwrap().clone();
    crate::mcp::approval::ask(channel.as_ref(), message)?;
    let now = ctx.limits(kb);
    if let Some(p) = now.problem {
        return Err(Error::new(
            Code::SetupProblem,
            format!(
                "The settings in {} cannot be used: {}",
                p.path.display(),
                p.message
            ),
        )
        .with_next("Fix or remove that file, then call status."));
    }
    match now.effective.check_tool(tool, kind) {
        Ok(()) => Ok(true),
        Err(crate::config::Denied::Paused) => Err(Error::new(
            Code::Paused,
            "Writes were paused while the question was open.",
        )
        .with_next("Nothing was written. Only the person can resume writes.")),
        Err(crate::config::Denied::AccessDenied(m)) => Err(Error::new(Code::AccessDenied, m)
            .with_next("Nothing was written. A limit blocks this tool.")),
    }
}

/// Run a write: build the plan inside the lock, ask for approval, apply the files, commit, push.
pub fn execute(
    ctx: &Context,
    env: &Env,
    tool: &str,
    kind: Kind,
    preview: bool,
    build: impl FnOnce(&Tx, &KnowledgeBase) -> Result<Plan>,
) -> Result<ToolOutput> {
    let repo = ctx.repo()?;
    ctx.kb()?;
    let author = author_string(ctx)?;
    let today = today();
    let (res, plan) = repo.write(preview, |tx| {
        let kb = KnowledgeBase::open(repo.dir())?;
        // Manual edits go first, in their own commit (see `manual-edits`).
        let (pending_notices, blocked) = if preview {
            (Vec::new(), Vec::new())
        } else {
            crate::manual::commit_before_write(tx, &kb, &env.effective, &author, &today)?
        };
        let mut plan = build(tx, &kb)?;
        for change in &plan.changes {
            let id = match change {
                FileChange::Write { id, .. }
                | FileChange::Remove { id }
                | FileChange::Keep { id } => id,
                FileChange::WriteFile { .. } | FileChange::RemoveFile { .. } => continue,
            };
            if blocked.contains(&kb.repo_relative(id)?) {
                return Err(Error::new(
                    Code::StaleBase,
                    format!("The document \"{id}\" has manual edits that are not published."),
                )
                .with_next("This document has manual edits that are not published. Run `sync`."));
            }
        }
        plan.notices.extend(pending_notices);
        let mut approved = false;
        if !preview {
            let paths: Vec<String> = plan
                .changes
                .iter()
                .map(|c| match c {
                    FileChange::Write { id, .. }
                    | FileChange::Remove { id }
                    | FileChange::Keep { id } => id.clone(),
                    FileChange::WriteFile { path, .. } | FileChange::RemoveFile { path } => {
                        path.clone()
                    }
                })
                .collect();
            approved = approve(ctx, env, &kb, tool, kind, &plan.summary, &paths)?;
        }
        if approved {
            plan.data.insert("approval".into(), Value::String("approved".into()));
        } else if preview && env.effective.needs_approval(tool, kind) {
            plan.data.insert("approval".into(), Value::String("required".into()));
        }
        // A repair of the settings file goes in its own commit, before the change of the caller.
        if !preview {
            let rel = kb
                .root
                .join(crate::kb::layout::MARKER)
                .join(crate::kb::layout::SETTINGS_FILE)
                .strip_prefix(&kb.repo_dir)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            if let Some(text) = tx.read(&rel)? {
                if let Some((fixed, done)) = crate::kb::repairs::repair(&text) {
                    tx.write_file(&rel, &fixed)?;
                    tx.repo().commit_change(&Change {
                        paths: vec![rel],
                        message: commit_message(Verb::Repair, "settings"),
                    })?;
                    for r in done {
                        plan.notices.push(r.notice.to_string());
                    }
                }
            }
        }
        let mut paths = Vec::new();
        let mut first_hash: Option<String> = None;
        let mut history_checked = false;
        if !preview {
            if let Some(first) = &plan.first {
                let head = tx.repo().head();
                let mut moved_paths = Vec::new();
                for (from, to) in &first.moves {
                    tx.rename(from, to)?;
                    moved_paths.push(from.clone());
                    moved_paths.push(to.clone());
                }
                first_hash = tx.repo().commit_change(&Change {
                    paths: moved_paths,
                    message: commit_message(Verb::Move, &first.title),
                })?;
                for (new_rel, old_commit) in &first.history {
                    if !tx.repo().history_reaches(new_rel, old_commit)? {
                        // Take the commit back, keep the files. The transaction restores them.
                        if let Some(h) = &head {
                            tx.repo().undo_commit_keep_files(h)?;
                        }
                        return Err(Error::validation(format!(
                            "Git cannot follow the history of \"{new_rel}\" after the move."
                        ))
                        .with_next("Nothing was changed. Move fewer documents at once, or ask someone who knows Git."));
                    }
                }
                history_checked = true;
            }
            if let Some(hashes) = &plan.revert {
                match tx.repo().revert_no_commit(hashes)? {
                    Ok(files) => {
                        for (_, path) in files {
                            if !paths.contains(&path) {
                                paths.push(path);
                            }
                        }
                    }
                    Err(why) => {
                        return Err(Error::new(
                            Code::StaleBase,
                            format!("The revert does not apply any more: {why}"),
                        )
                        .with_next("Change the document with `update_document` instead."))
                    }
                }
            }
            for change in &plan.changes {
                match change {
                    FileChange::Write { id, text, .. } => {
                        let rel = kb.repo_relative(id)?;
                        tx.write_file(&rel, text)?;
                        paths.push(rel);
                    }
                    FileChange::Remove { id } => {
                        let rel = kb.repo_relative(id)?;
                        tx.remove_file(&rel)?;
                        paths.push(rel);
                    }
                    FileChange::Keep { id } => paths.push(kb.repo_relative(id)?),
                    FileChange::WriteFile { path, text, .. } => {
                        let rel = kb.repo_relative_file(path);
                        tx.write_file(&rel, text)?;
                        paths.push(rel);
                    }
                    FileChange::RemoveFile { path } => {
                        let rel = kb.repo_relative_file(path);
                        tx.remove_file(&rel)?;
                        paths.push(rel);
                    }
                }
            }
        }
        let mut message = commit_message(plan.verb, &plan.title);
        if let Some(body) = &plan.commit_body {
            message.push_str("\n\n");
            message.push_str(body);
        }
        if let Some(h) = &first_hash {
            message.push_str(&format!("\n\nMove: {h}"));
        }
        Ok((Change { paths, message }, (plan, first_hash, history_checked)))
    })?;
    let (plan, first_hash, history_checked) = plan;

    let mut data = plan.data.clone();
    if plan.first.is_some() {
        let commits: Vec<&String> = first_hash.iter().chain(res.commit.iter()).collect();
        data.insert(
            "commits".into(),
            json!(if preview { Value::Null } else { json!(commits) }),
        );
        data.insert(
            "history_checked".into(),
            if preview {
                Value::Null
            } else {
                json!(history_checked)
            },
        );
    } else {
        data.insert("commit".into(), json!(res.commit));
        if res.commit.is_some() {
            data.insert("version".into(), json!(res.commit));
        } else if !data.contains_key("version") {
            data.insert("version".into(), Value::Null);
        }
    }
    data.insert(
        "pushed".into(),
        if preview {
            Value::Null
        } else {
            json!(res.pushed)
        },
    );
    data.insert("preview".into(), json!(preview));
    if preview {
        let changes: Vec<Value> = plan
            .changes
            .iter()
            .filter_map(|c| match c {
                FileChange::Write { id, action, .. } => Some(json!({"id": id, "action": action})),
                FileChange::Remove { id } => Some(json!({"id": id, "action": "delete"})),
                FileChange::WriteFile { path, action, .. } => {
                    Some(json!({"id": path, "action": action}))
                }
                FileChange::RemoveFile { path } => Some(json!({"id": path, "action": "delete"})),
                FileChange::Keep { .. } => None,
            })
            .collect();
        if !data.contains_key("changes") {
            data.insert("changes".into(), json!(changes));
        }
        if let Some(t) = &plan.text {
            data.insert("text".into(), json!(t));
        }
    }
    let message = if preview {
        format!("Preview, nothing was written. {}", plan.message)
    } else {
        plan.message.clone()
    };
    let mut out = ToolOutput::new(message, Value::Object(data));
    for n in plan.notices.iter().chain(res.notices.iter()) {
        out = out.notice(n.clone());
    }
    Ok(out)
}

/// Check a `reason` or other one-line text: one line, not empty, at most `max_words` words.
pub fn check_reason(reason: &str, max_words: usize) -> Result<()> {
    if reason.trim().is_empty() || reason.contains('\n') {
        return Err(Error::validation("The reason must be one line of text.")
            .with_next("Send a short reason in one line."));
    }
    if format::count::words(reason) > max_words {
        return Err(
            Error::validation(format!("The reason has more than {max_words} words."))
                .with_next("Shorten the reason."),
        );
    }
    Ok(())
}

/// A folder from the input: no leading `/`, no `..`, no hidden segment. `""` is the root.
pub fn clean_folder(folder: &str) -> Result<String> {
    let bad = || {
        Error::invalid(format!("\"{folder}\" is not a valid folder.")).with_next(
            "Use a path below the root, such as \"engineering/backend\". Use \"\" for the root.",
        )
    };
    if folder.starts_with('/') || folder.contains('\\') || folder.contains('\0') {
        return Err(bad());
    }
    let trimmed = folder.trim_end_matches('/');
    if trimmed.is_empty() {
        return Ok(String::new());
    }
    for seg in trimmed.split('/') {
        if seg.is_empty() || seg == "." || seg == ".." || seg.starts_with('.') {
            return Err(bad());
        }
    }
    Ok(trimmed.to_string())
}

/// A folder path for the folder tools: parts of lowercase letters, digits, and `-`, joined by `/`,
/// at most 8 levels deep. Returns the path without a trailing `/`.
pub fn check_folder_path(path: &str) -> Result<String> {
    let bad = |why: &str| {
        Error::invalid(format!("The field `folder` is wrong: {why}"))
            .with_next("Fix the field `folder` and call again.")
    };
    let trimmed = path.trim_end_matches('/');
    if trimmed.is_empty() {
        return Err(bad("it must name a folder below the root"));
    }
    let parts: Vec<&str> = trimmed.split('/').collect();
    if parts.len() > 8 {
        return Err(bad("at most 8 levels"));
    }
    for seg in &parts {
        let ok = !seg.is_empty()
            && !seg.starts_with('-')
            && seg
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
        if !ok {
            return Err(bad(&format!(
                "\"{path}\" must use lowercase letters, digits, and `-` only, in parts joined by `/`"
            )));
        }
    }
    Ok(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folder_paths_are_checked() {
        assert_eq!(check_folder_path("ops/deploy/").unwrap(), "ops/deploy");
        for bad in [
            "",
            "/a",
            "a/../b",
            "A",
            "a b",
            "-a",
            "a//b",
            ".mkbase",
            "a/b/c/d/e/f/g/h/i",
        ] {
            assert!(check_folder_path(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn body_is_normalized() {
        assert_eq!(make_body("Hi\r\nthere\n\n"), "\nHi\nthere\n");
        assert_eq!(make_body("\n\n"), "");
    }

    #[test]
    fn folders_are_checked() {
        assert_eq!(clean_folder("").unwrap(), "");
        assert_eq!(clean_folder("a/b/").unwrap(), "a/b");
        for bad in ["/a", "a/../b", "..", ".mkbase", "a/.hidden", "a//b"] {
            assert!(clean_folder(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn reasons_are_one_short_line() {
        assert!(check_reason("It is wrong.", 50).is_ok());
        assert!(check_reason("", 50).is_err());
        assert!(check_reason("a\nb", 50).is_err());
        assert!(check_reason(&"word ".repeat(51), 50).is_err());
    }

    #[test]
    fn problems_make_one_error_with_all_lines() {
        let mut p = Problems::default();
        p.add("One.", "Do A.");
        p.add("Two.", "Do A.");
        let e = p.into_error();
        assert_eq!(e.code, Code::ValidationFailed);
        assert_eq!(e.message, "One.\nTwo.");
        assert_eq!(e.next.as_deref(), Some("Do A."));
    }
}
