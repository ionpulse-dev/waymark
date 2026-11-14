//! The net change of documents in a time window (see `specs/features/updates`).

use crate::error::Result;
use crate::format;
use crate::gitsync::diff::PathChange;
use crate::gitsync::Repo;
use crate::kb::KnowledgeBase;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    New,
    Changed,
    Moved,
    Deprecated,
    Disputed,
    Deleted,
}

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Kind::New => "new",
            Kind::Changed => "changed",
            Kind::Moved => "moved",
            Kind::Deprecated => "deprecated",
            Kind::Disputed => "disputed",
            Kind::Deleted => "deleted",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Change {
    pub id: String,
    pub title: String,
    pub kind: Kind,
    pub mine: bool,
    pub by: String,
    pub date: String,
    pub reason: Option<String>,
    pub from: Option<String>,
    pub trust: Option<String>,
}

/// The email inside `Name <email>`.
pub fn email_of(person: &str) -> String {
    person
        .rsplit_once('<')
        .and_then(|(_, r)| r.split_once('>'))
        .map(|(e, _)| e.trim().to_lowercase())
        .unwrap_or_default()
}

fn root_prefix(kb: &KnowledgeBase) -> String {
    let rel = kb.root.strip_prefix(&kb.repo_dir).unwrap_or(&kb.root);
    rel.to_string_lossy().trim_matches('/').to_string()
}

/// The document ID of a repository path, or `None` if the path is not a document.
fn id_of(kb: &KnowledgeBase, path: &str) -> Option<String> {
    let prefix = root_prefix(kb);
    let below = if prefix.is_empty() {
        path
    } else {
        path.strip_prefix(&format!("{prefix}/"))?
    };
    let id = below.strip_suffix(".md")?;
    if id.split('/').any(|s| s.starts_with('.')) || kb.is_excluded(id) {
        return None;
    }
    Some(id.to_string())
}

fn is_index(id: &str, doc: &format::Document) -> bool {
    (id == "index" || id.ends_with("/index"))
        && doc
            .frontmatter
            .get_list("tags")
            .is_some_and(|t| t.iter().any(|x| x == "index"))
}

/// Classify every changed document between `base` and `HEAD`. Changes that only the person made
/// are dropped.
pub fn changes_since(
    repo: &Repo,
    kb: &KnowledgeBase,
    base: &str,
    own_email: &str,
) -> Result<Vec<Change>> {
    let prefix = root_prefix(kb);
    let own = own_email.to_lowercase();
    let mut out = Vec::new();
    for pc in repo.changed_paths(base, &prefix)? {
        let PathChange { status, path, from } = pc;
        let Some(id) = id_of(kb, &path) else { continue };
        let at_head = repo
            .show_at("HEAD", &path)?
            .and_then(|t| format::parse(&t).ok());
        let at_base = |p: &str| {
            repo.show_at(base, p)
                .ok()
                .flatten()
                .and_then(|t| format::parse(&t).ok())
        };
        let old_path = from.clone().unwrap_or_else(|| path.clone());
        let before = if status == 'A' {
            None
        } else {
            at_base(&old_path)
        };
        let shown = at_head.as_ref().or(before.as_ref());
        let Some(shown) = shown else { continue };
        if is_index(&id, shown) {
            continue;
        }
        let commits = repo.window_commits(base, &path)?;
        let Some(latest) = commits.iter().find(|c| c.email != own) else {
            continue;
        };
        let fm = &shown.frontmatter;
        let (kind, reason) = match status {
            'A' => (Kind::New, None),
            'D' => (Kind::Deleted, None),
            'R' => (Kind::Moved, None),
            _ => {
                let was_disputed = before
                    .as_ref()
                    .and_then(|b| b.frontmatter.get_bool("disputed"))
                    == Some(true);
                let now_disputed = fm.get_bool("disputed") == Some(true);
                let was_active = before
                    .as_ref()
                    .map(|b| b.frontmatter.get_text("status") != Some("deprecated"))
                    .unwrap_or(true);
                if now_disputed && !was_disputed {
                    (
                        Kind::Disputed,
                        fm.get_text("dispute_reason").map(String::from),
                    )
                } else if fm.get_text("status") == Some("deprecated") && was_active {
                    (Kind::Deprecated, None)
                } else {
                    (Kind::Changed, None)
                }
            }
        };
        out.push(Change {
            title: fm.get_text("title").unwrap_or("").to_string(),
            mine: email_of(fm.get_text("created_by").unwrap_or("")) == own,
            by: format!("{} <{}>", latest.name, latest.email),
            date: latest.date.clone(),
            reason,
            from: from.as_deref().and_then(|f| id_of(kb, f)),
            trust: fm.get_text("trust").map(String::from),
            kind,
            id,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_of_reads_the_angle_brackets() {
        assert_eq!(email_of("Alex Roe <Alex@Example.org>"), "alex@example.org");
        assert_eq!(email_of("no email"), "");
    }
}
