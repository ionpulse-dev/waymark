//! The writes of a person, read from Git history (see `specs/features/activity-and-undo`).

use crate::gitsync::{parse_subject, CommitInfo, Verb};
use crate::kb::KnowledgeBase;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct Entry {
    /// The commit to show and to undo (for a move: the move commit).
    pub commit: String,
    pub hashes: Vec<String>,
    pub time: String,
    pub verb: Verb,
    pub title: String,
    pub folder: String,
    pub ids: Vec<String>,
    /// All changed paths below the repository folder, of all commits of the entry.
    pub paths: Vec<String>,
    pub files: usize,
    pub undone: bool,
    pub undone_by: Option<String>,
    pub later_changes: bool,
    /// Position in the newest-first list of all commits.
    pub index: usize,
}

/// The ID of a document path, if it is a document below the root.
pub fn id_of(kb: &KnowledgeBase, rel: &str) -> Option<String> {
    let prefix = kb
        .root
        .strip_prefix(&kb.repo_dir)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let below = if prefix.is_empty() {
        rel
    } else {
        rel.strip_prefix(&format!("{prefix}/"))?
    };
    if below.split('/').any(|s| s.starts_with('.')) {
        return None;
    }
    below.strip_suffix(".md").map(str::to_string)
}

fn reverts_of(c: &CommitInfo) -> Vec<String> {
    c.body
        .lines()
        .find_map(|l| l.strip_prefix("Reverts:"))
        .map(|l| l.split_whitespace().map(str::to_string).collect())
        .unwrap_or_default()
}

fn move_ref(c: &CommitInfo) -> Option<String> {
    c.body
        .lines()
        .find_map(|l| l.strip_prefix("Move:"))
        .map(|l| l.trim().to_string())
}

fn same(a: &str, b: &str) -> bool {
    !a.is_empty() && !b.is_empty() && (a.starts_with(b) || b.starts_with(a))
}

/// The entries of one person, newest first. `commits` is newest first.
pub fn entries(kb: &KnowledgeBase, commits: &[CommitInfo], email: &str) -> Vec<Entry> {
    let email = email.to_lowercase();
    let mine =
        |c: &CommitInfo| c.email == email && c.subject.starts_with(crate::gitsync::message::PREFIX);

    // `undone`, in time order: an undo marks its targets, and a redo clears the targets of the
    // undo that it reverts.
    let mut undone: HashMap<String, Option<String>> = HashMap::new();
    for c in commits.iter().rev() {
        let Some((Verb::Undo, _)) = parse_subject(&c.subject) else {
            continue;
        };
        let targets = reverts_of(c);
        for t in &targets {
            // A target that is an undo itself: its own targets are live again.
            if let Some(u) = commits.iter().find(|x| same(&x.hash, t)) {
                if matches!(parse_subject(&u.subject), Some((Verb::Undo, _))) {
                    for t2 in reverts_of(u) {
                        if let Some(full) = commits.iter().find(|x| same(&x.hash, &t2)) {
                            undone.insert(full.hash.clone(), None);
                        }
                    }
                }
                undone.insert(u.hash.clone(), Some(c.short.clone()));
            }
        }
    }

    // The link-fix commit of a move belongs to the move entry.
    let mut grouped: HashMap<String, &CommitInfo> = HashMap::new();
    for c in commits.iter().filter(|c| mine(c)) {
        if let (Some((Verb::Update, _)), Some(mv)) = (parse_subject(&c.subject), move_ref(c)) {
            grouped.insert(mv, c);
        }
    }
    let mut seen_fix: HashSet<String> = grouped.values().map(|c| c.hash.clone()).collect();
    seen_fix.retain(|h| commits.iter().any(|c| c.hash == *h));

    let mut out = Vec::new();
    for (index, c) in commits.iter().enumerate() {
        if !mine(c) || seen_fix.contains(&c.hash) {
            continue;
        }
        let Some((verb, title)) = parse_subject(&c.subject) else {
            continue;
        };
        let mut hashes = vec![c.hash.clone()];
        let mut files = c.files.clone();
        if verb == Verb::Move {
            if let Some(fix) = grouped
                .iter()
                .find(|(mv, _)| same(&c.hash, mv))
                .map(|(_, f)| *f)
            {
                hashes.insert(0, fix.hash.clone());
                files.extend(fix.files.clone());
            }
        }
        let mut paths: Vec<String> = Vec::new();
        let mut ids: Vec<String> = Vec::new();
        for (_, p) in &files {
            if !paths.contains(p) {
                paths.push(p.clone());
            }
        }
        for (_, p) in &files {
            if let Some(id) = id_of(kb, p) {
                if !ids.contains(&id) {
                    ids.push(id);
                }
            }
        }
        if ids.is_empty() {
            continue;
        }
        let folder = ids[0]
            .rsplit_once('/')
            .map(|(f, _)| f.to_string())
            .unwrap_or_default();
        let (is_undone, by) = match undone.get(&c.hash) {
            Some(Some(u)) => (true, Some(u.clone())),
            _ => (false, None),
        };
        out.push(Entry {
            commit: c.short.clone(),
            hashes: hashes
                .iter()
                .map(|h| h[..7.min(h.len())].to_string())
                .collect(),
            time: c.time.clone(),
            verb,
            title,
            folder,
            files: ids.len(),
            ids,
            paths,
            undone: is_undone,
            undone_by: by,
            later_changes: false,
            index,
        });
    }

    // `later_changes`: a newer commit by anyone changed one of the same files.
    for e in &mut out {
        let reverters: Vec<&CommitInfo> = commits
            .iter()
            .filter(|x| {
                reverts_of(x)
                    .iter()
                    .any(|r| e.hashes.iter().any(|h| same(h, r)))
            })
            .collect();
        let own: HashSet<&String> = e.hashes.iter().collect();
        e.later_changes = commits[..e.index].iter().any(|x| {
            !own.iter().any(|h| same(&x.hash, h))
                && !reverters.iter().any(|r| r.hash == x.hash)
                && !(move_ref(x).is_some_and(|m| same(&c_hash(commits, e.index), &m)))
                && x.files.iter().any(|(_, p)| e.paths.contains(p))
        });
    }
    out
}

fn c_hash(commits: &[CommitInfo], index: usize) -> String {
    commits[index].hash.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn commit(
        hash: &str,
        email: &str,
        subject: &str,
        body: &str,
        files: &[(char, &str)],
    ) -> CommitInfo {
        CommitInfo {
            hash: format!("{hash}0000000"),
            short: hash.to_string(),
            email: email.to_string(),
            name: "N".into(),
            time: "2026-11-14T10:00:00+01:00".into(),
            subject: subject.to_string(),
            body: body.to_string(),
            files: files.iter().map(|(s, p)| (*s, p.to_string())).collect(),
        }
    }

    fn kb() -> (tempfile::TempDir, KnowledgeBase) {
        let d = tempfile::tempdir().unwrap();
        crate::kb::init(d.path(), "T", "t").unwrap();
        let kb = KnowledgeBase::open(d.path()).unwrap();
        (d, kb)
    }

    #[test]
    fn only_my_waymark_commits_with_documents_show() {
        let (_d, kb) = kb();
        let commits = vec![
            commit(
                "a1",
                "me@x.org",
                "waymark: add \"A\"",
                "",
                &[('A', "knowledge/ops/a.md")],
            ),
            commit(
                "b1",
                "other@x.org",
                "waymark: add \"B\"",
                "",
                &[('A', "knowledge/ops/b.md")],
            ),
            commit(
                "c1",
                "me@x.org",
                "plain commit",
                "",
                &[('A', "knowledge/ops/c.md")],
            ),
            commit(
                "d1",
                "me@x.org",
                "waymark: update \"S\"",
                "",
                &[('M', "knowledge/.mkbase/waymark.toml")],
            ),
        ];
        let e = entries(&kb, &commits, "ME@x.org");
        assert_eq!(e.len(), 1);
        assert_eq!(e[0].title, "A");
        assert_eq!(e[0].ids, vec!["ops/a"]);
        assert_eq!(e[0].folder, "ops");
    }

    #[test]
    fn an_undo_marks_the_target_and_a_redo_clears_it() {
        let (_d, kb) = kb();
        let add = commit(
            "a1",
            "me@x.org",
            "waymark: add \"A\"",
            "",
            &[('A', "knowledge/a.md")],
        );
        let undo = commit(
            "u1",
            "me@x.org",
            "waymark: undo \"A\"",
            "Reverts: a1",
            &[('D', "knowledge/a.md")],
        );
        let log = vec![undo.clone(), add.clone()];
        let e = entries(&kb, &log, "me@x.org");
        let a = e.iter().find(|x| x.commit == "a1").unwrap();
        assert!(a.undone && a.undone_by.as_deref() == Some("u1"));
        // A redo reverts the undo: the add is live again, and the undo is undone.
        let redo = commit(
            "r1",
            "me@x.org",
            "waymark: undo \"A\"",
            "Reverts: u1",
            &[('A', "knowledge/a.md")],
        );
        let log = vec![redo, undo, add];
        let e = entries(&kb, &log, "me@x.org");
        assert!(!e.iter().find(|x| x.commit == "a1").unwrap().undone);
        assert!(e.iter().find(|x| x.commit == "u1").unwrap().undone);
    }

    #[test]
    fn a_move_and_its_link_fix_are_one_entry() {
        let (_d, kb) = kb();
        let mv = commit(
            "m1",
            "me@x.org",
            "waymark: move \"New\"",
            "",
            &[('D', "knowledge/a/x.md"), ('A', "knowledge/b/x.md")],
        );
        let fix = commit(
            "f1",
            "me@x.org",
            "waymark: update \"New\"",
            "Move: m1",
            &[('M', "knowledge/c/y.md")],
        );
        let e = entries(&kb, &[fix, mv], "me@x.org");
        assert_eq!(e.len(), 1);
        assert_eq!(e[0].verb, Verb::Move);
        assert_eq!(e[0].hashes.len(), 2);
        assert_eq!(e[0].files, 3);
        assert!(!e[0].later_changes);
    }

    #[test]
    fn later_changes_are_found() {
        let (_d, kb) = kb();
        let add = commit(
            "a1",
            "me@x.org",
            "waymark: add \"A\"",
            "",
            &[('A', "knowledge/a.md")],
        );
        let other = commit(
            "o1",
            "other@x.org",
            "waymark: update \"A\"",
            "",
            &[('M', "knowledge/a.md")],
        );
        let e = entries(&kb, &[other, add], "me@x.org");
        assert!(e[0].later_changes);
    }
}
