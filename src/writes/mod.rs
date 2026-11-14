//! Shared parts of the write tools (see `specs/features/write-documents`).
//!
//! A tool builds a `Plan`: the files to write, remove, or keep, the commit message, and the
//! data for the result. `execute` runs the plan through `Repo::write`, which holds the lock,
//! brings the clone up to date, commits under the author, and pushes.

pub mod linkfix;
pub mod secrets;
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
    let repo = ctx.repo()?;
    let a = repo.author();
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

/// Check the full text of a document as it will be written.
/// `ids_after` is the set of document IDs after the write. `secret_fields` are the texts to scan.
/// Warnings go to `notices`.
#[allow(clippy::too_many_arguments)]
pub fn check_document(
    kb: &KnowledgeBase,
    id: &str,
    text: &str,
    ids_after: &HashSet<String>,
    secret_fields: &[(&str, String)],
    title_check: Option<TitleCheck>,
    problems: &mut Problems,
    notices: &mut Vec<String>,
) {
    let path = format!("{id}.md");
    for f in format::check(&path, text, &kb.settings.documents, Some(ids_after)) {
        if f.is_error() {
            problems.add(
                f.message,
                "Fix the problems that are listed, then call again.",
            );
        } else {
            notices.push(f.message);
        }
    }
    let custom = secrets::compile(&kb.settings.extra_secret_patterns);
    for (field, value) in secret_fields {
        for hit in secrets::scan(value, &custom) {
            problems.add(
                format!(
                    "The {field} looks like it contains a secret (rule {}, line {}).",
                    hit.rule, hit.line
                ),
                "Remove the secret, or replace it with a placeholder.",
            );
        }
    }
    if let Some(tc) = title_check {
        if let Ok(doc) = format::parse(text) {
            if let Some(title) = doc.frontmatter.get_text("title") {
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
    }
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
}

pub struct Plan {
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

/// The approval step of the write flow. `write-approval` fills it in. Until then a write that
/// needs approval fails, so that nothing is written without the approval that the limits ask for.
fn approve(env: &Env, tool: &str, kind: Kind, _summary: &str) -> Result<()> {
    if env.effective.needs_approval(tool, kind) {
        Err(Error::new(
            Code::ApprovalUnavailable,
            "This write needs approval, and no approval channel is available.",
        )
        .with_next(
            "Remove the tool from approval.required_for, or use a client that supports approval.",
        ))
    } else {
        Ok(())
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
    let (res, plan) = repo.write(preview, |tx| {
        let kb = KnowledgeBase::open(repo.dir())?;
        let plan = build(tx, &kb)?;
        if !preview {
            approve(env, tool, kind, &plan.summary)?;
        }
        let mut paths = Vec::new();
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
            }
        }
        let mut message = commit_message(plan.verb, &plan.title);
        if let Some(body) = &plan.commit_body {
            message.push_str("\n\n");
            message.push_str(body);
        }
        Ok((Change { paths, message }, plan))
    })?;

    let mut data = plan.data.clone();
    data.insert("commit".into(), json!(res.commit));
    if res.commit.is_some() {
        data.insert("version".into(), json!(res.commit));
    } else if !data.contains_key("version") {
        data.insert("version".into(), Value::Null);
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
                FileChange::Keep { .. } => None,
            })
            .collect();
        data.insert("changes".into(), json!(changes));
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

#[cfg(test)]
mod tests {
    use super::*;

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
