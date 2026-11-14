//! Rewrite links to a document that goes away (used by `delete`).
//!
//! Links in fenced code and inline code are never touched. Only the link text changes, so the
//! rest of the body stays byte for byte the same.

use crate::format::frontmatter::Frontmatter;
use crate::format::links::normalize;
use regex::{Captures, Regex};
use std::sync::OnceLock;

fn md_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(!?)\[([^\]\n]*)\]\(([^)\s]+)(?:\s+[^)]*)?\)").unwrap())
}

fn wiki_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(!?)\[\[([^\[\]\n|#]+)(?:#[^\[\]\n|]*)?(?:\|([^\[\]\n]*))?\]\]").unwrap()
    })
}

/// The document that goes away.
pub struct Target<'a> {
    pub id: &'a str,
    pub title: &'a str,
    /// Does a wikilink name (without alias and heading) point to the target?
    pub wiki_matches: &'a dyn Fn(&str) -> bool,
}

fn fix_segment(doc_dir: &str, seg: &str, t: &Target, count: &mut usize) -> String {
    let step1 = md_re().replace_all(seg, |c: &Captures| {
        let raw = &c[3];
        let target = raw.split('#').next().unwrap_or("");
        let is_doc = !raw.contains("://")
            && target
                .strip_suffix(".md")
                .and_then(|stem| normalize(doc_dir, stem))
                .map(|id| id == t.id)
                .unwrap_or(false);
        if is_doc {
            *count += 1;
            if &c[1] == "!" {
                String::new()
            } else {
                c[2].to_string()
            }
        } else {
            c[0].to_string()
        }
    });
    wiki_re()
        .replace_all(&step1, |c: &Captures| {
            let name = c[2].trim().trim_end_matches(".md");
            if (t.wiki_matches)(name) {
                *count += 1;
                if &c[1] == "!" {
                    String::new()
                } else {
                    c.get(3)
                        .map(|a| a.as_str().to_string())
                        .filter(|a| !a.is_empty())
                        .unwrap_or_else(|| t.title.to_string())
                }
            } else {
                c[0].to_string()
            }
        })
        .to_string()
}

/// Fix the links in a body. `doc_path` is the path of the document, with `.md`.
/// Returns the new body and the number of links changed.
pub fn fix_body(doc_path: &str, body: &str, t: &Target) -> (String, usize) {
    let doc_dir = doc_path.rsplit_once('/').map(|(d, _)| d).unwrap_or("");
    let mut count = 0;
    let mut out = String::with_capacity(body.len());
    let mut in_fence = false;
    for line in body.split_inclusive('\n') {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            out.push_str(line);
            continue;
        }
        if in_fence {
            out.push_str(line);
            continue;
        }
        // Split at backticks: odd parts are inline code.
        for (i, part) in line.split('`').enumerate() {
            if i > 0 {
                out.push('`');
            }
            if i % 2 == 0 {
                out.push_str(&fix_segment(doc_dir, part, t, &mut count));
            } else {
                out.push_str(part);
            }
        }
    }
    (out, count)
}

/// Remove the entries that point to the target from `related` and `supersedes`.
/// An empty list removes the property. Returns the number of entries removed.
pub fn fix_properties(fm: &mut Frontmatter, t: &Target) -> usize {
    let mut removed = 0;
    for field in ["related", "supersedes"] {
        let Some(items) = fm.get_list(field).map(|l| l.to_vec()) else {
            continue;
        };
        let keep: Vec<String> = items
            .iter()
            .filter(|i| {
                let hit = crate::format::fields::property_link_target(i)
                    .map(|name| name == t.id || (t.wiki_matches)(name))
                    .unwrap_or(false);
                if hit {
                    removed += 1;
                }
                !hit
            })
            .cloned()
            .collect();
        if keep.len() != items.len() {
            if keep.is_empty() {
                fm.remove(field);
            } else {
                fm.set_list(field, &keep);
            }
        }
    }
    removed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::parse;

    fn target<'a>(f: &'a dyn Fn(&str) -> bool) -> Target<'a> {
        Target {
            id: "ops/old",
            title: "Old deploy",
            wiki_matches: f,
        }
    }

    fn matches(name: &str) -> bool {
        name == "ops/old" || name == "old"
    }

    #[test]
    fn markdown_links_become_their_text() {
        let (b, n) = fix_body(
            "ops/new.md",
            "See [the old one](old.md) and [x](../ops/old.md#h).\n",
            &target(&matches),
        );
        assert_eq!(b, "See the old one and x.\n");
        assert_eq!(n, 2);
    }

    #[test]
    fn wikilinks_become_the_alias_or_the_title() {
        let (b, n) = fix_body(
            "a.md",
            "A [[ops/old]], B [[old|the alias]], C [[ops/old#Head]].\n",
            &target(&matches),
        );
        assert_eq!(b, "A Old deploy, B the alias, C Old deploy.\n");
        assert_eq!(n, 3);
    }

    #[test]
    fn embeds_are_removed() {
        let (b, n) = fix_body("a.md", "Before ![[ops/old]] after\n", &target(&matches));
        assert_eq!(b, "Before  after\n");
        assert_eq!(n, 1);
    }

    #[test]
    fn other_links_and_code_stay() {
        let body = "[ext](https://x.org/old.md) [[ops/other]] [o](other.md)\n`[[ops/old]]`\n```\n[[ops/old]]\n[a](old.md)\n```\n";
        let (b, n) = fix_body("ops/new.md", body, &target(&matches));
        assert_eq!(b, body);
        assert_eq!(n, 0);
    }

    #[test]
    fn property_entries_are_removed_and_an_empty_list_drops_the_property() {
        let mut d = parse("---\ntitle: A\nrelated:\n  - \"[[ops/old]]\"\n  - \"[[ops/keep]]\"\nsupersedes:\n  - \"[[ops/old]]\"\n---\nBody\n").unwrap();
        let n = fix_properties(&mut d.frontmatter, &target(&matches));
        assert_eq!(n, 2);
        let out = d.render();
        assert!(
            out.contains("ops/keep") && !out.contains("ops/old") && !out.contains("supersedes")
        );
    }
}

// ---- retarget: links to documents that move (used by `move`) ----

use std::collections::{HashMap, HashSet};

/// The documents of a move: old ID to new ID, and the file names before and after.
pub struct Remap {
    pub moved: HashMap<String, String>,
    old_ids: HashSet<String>,
    new_ids: HashSet<String>,
}

fn file_name(id: &str) -> &str {
    id.rsplit('/').next().unwrap_or(id)
}

fn dir_of(id: &str) -> &str {
    id.rsplit_once('/').map(|(d, _)| d).unwrap_or("")
}

impl Remap {
    /// `old_ids` is the set of all IDs before the move.
    pub fn new(moved: HashMap<String, String>, old_ids: &HashSet<String>) -> Remap {
        let mut new_ids: HashSet<String> = old_ids
            .iter()
            .filter(|i| !moved.contains_key(*i))
            .cloned()
            .collect();
        new_ids.extend(moved.values().cloned());
        Remap {
            moved,
            old_ids: old_ids.clone(),
            new_ids,
        }
    }

    pub fn new_id<'a>(&'a self, old: &'a str) -> &'a str {
        self.moved.get(old).map(String::as_str).unwrap_or(old)
    }

    /// The old ID that a short wikilink name found, if it found exactly one document.
    fn old_target_of_name(&self, name: &str) -> Option<&str> {
        let mut found = self.old_ids.iter().filter(|i| file_name(i) == name);
        let first = found.next()?;
        found.next().is_none().then_some(first.as_str())
    }

    fn new_name_is_unique(&self, name: &str, id: &str) -> bool {
        let mut found = self.new_ids.iter().filter(|i| file_name(i) == name);
        found.next().map(String::as_str) == Some(id) && found.next().is_none()
    }

    /// The new full-path form of a wikilink name, or `None` if the link stays as it is.
    fn wiki_target(&self, name: &str) -> Option<String> {
        if name.contains('/') {
            return self.moved.get(name).cloned();
        }
        let old = self.old_target_of_name(name)?;
        let new = self.moved.get(old)?;
        if file_name(new) == name && self.new_name_is_unique(name, new) {
            None
        } else {
            Some(new.clone())
        }
    }
}

/// The relative path from a folder to a document file.
fn relative_path(from_dir: &str, target_id: &str) -> String {
    let from: Vec<&str> = if from_dir.is_empty() {
        Vec::new()
    } else {
        from_dir.split('/').collect()
    };
    let to: Vec<&str> = target_id.split('/').collect();
    let common = from
        .iter()
        .zip(to.iter().take(to.len().saturating_sub(1)))
        .take_while(|(a, b)| a == b)
        .count();
    let mut parts: Vec<String> = vec!["..".to_string(); from.len() - common];
    parts.extend(to[common..].iter().map(|s| s.to_string()));
    format!("{}.md", parts.join("/"))
}

fn retarget_segment(
    old_dir: &str,
    new_dir: &str,
    seg: &str,
    r: &Remap,
    count: &mut usize,
) -> String {
    let step1 = md_re().replace_all(seg, |c: &Captures| {
        let raw = &c[3];
        if raw.contains("://") || raw.starts_with("mailto:") || raw.starts_with('#') {
            return c[0].to_string();
        }
        let (path, heading) = match raw.split_once('#') {
            Some((p, h)) => (p, Some(h)),
            None => (raw, None),
        };
        let Some(stem) = path.strip_suffix(".md") else {
            return c[0].to_string();
        };
        let absolute = path.starts_with('/');
        let Some(old_target) = normalize(old_dir, stem) else {
            return c[0].to_string();
        };
        let new_target = r.new_id(&old_target).to_string();
        // Still valid from the new location? Then leave the text alone.
        if normalize(new_dir, stem).as_deref() == Some(new_target.as_str()) {
            return c[0].to_string();
        }
        let new_path = if absolute {
            format!("/{new_target}.md")
        } else {
            relative_path(new_dir, &new_target)
        };
        *count += 1;
        let new_raw = match heading {
            Some(h) => format!("{new_path}#{h}"),
            None => new_path,
        };
        c[0].replacen(raw, &new_raw, 1)
    });
    wiki_re()
        .replace_all(&step1, |c: &Captures| {
            let name = c[2].trim().trim_end_matches(".md");
            match r.wiki_target(name) {
                Some(new) => {
                    *count += 1;
                    c[0].replacen(&c[2], &new, 1)
                }
                None => c[0].to_string(),
            }
        })
        .to_string()
}

/// Rewrite the links of a body. `old_id` and `new_id` are the IDs of the linking document before
/// and after the move. Returns the new body and the number of links changed.
pub fn retarget_body(old_id: &str, new_id: &str, body: &str, r: &Remap) -> (String, usize) {
    let (old_dir, new_dir) = (dir_of(old_id), dir_of(new_id));
    let mut count = 0;
    let mut out = String::with_capacity(body.len());
    let mut in_fence = false;
    for line in body.split_inclusive('\n') {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            out.push_str(line);
            continue;
        }
        if in_fence {
            out.push_str(line);
            continue;
        }
        for (i, part) in line.split('`').enumerate() {
            if i > 0 {
                out.push('`');
            }
            if i % 2 == 0 {
                out.push_str(&retarget_segment(old_dir, new_dir, part, r, &mut count));
            } else {
                out.push_str(part);
            }
        }
    }
    (out, count)
}

/// Replace moved IDs in `related` and `supersedes`. Returns the number of entries changed.
pub fn retarget_properties(fm: &mut Frontmatter, r: &Remap) -> usize {
    let mut changed = 0;
    for field in ["related", "supersedes"] {
        let Some(items) = fm.get_list(field).map(|l| l.to_vec()) else {
            continue;
        };
        let new: Vec<String> = items
            .iter()
            .map(|i| match crate::format::fields::property_link_target(i) {
                Some(id) if r.moved.contains_key(id) => {
                    changed += 1;
                    format!("[[{}]]", r.moved[id])
                }
                _ => i.clone(),
            })
            .collect();
        if new != items {
            fm.set_list(field, &new);
        }
    }
    changed
}

#[cfg(test)]
mod retarget_tests {
    use super::*;
    use crate::format::parse;

    fn remap(pairs: &[(&str, &str)], all: &[&str]) -> Remap {
        let moved = pairs
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect();
        let old: HashSet<String> = all.iter().map(|s| s.to_string()).collect();
        Remap::new(moved, &old)
    }

    #[test]
    fn relative_paths_are_computed_from_the_new_folder() {
        assert_eq!(relative_path("a/b", "a/c/x"), "../c/x.md");
        assert_eq!(relative_path("", "a/x"), "a/x.md");
        assert_eq!(relative_path("a", "a/x"), "x.md");
        assert_eq!(relative_path("a/b", "x"), "../../x.md");
    }

    #[test]
    fn a_link_to_a_moved_document_is_rewritten() {
        let r = remap(&[("old/t", "new/t")], &["old/t", "ops/d"]);
        let (b, n) = retarget_body("ops/d", "ops/d", "See [t](../old/t.md#h \"title\").\n", &r);
        assert_eq!(b, "See [t](../new/t.md#h \"title\").\n");
        assert_eq!(n, 1);
    }

    #[test]
    fn links_between_documents_that_move_together_stay() {
        let r = remap(&[("a/x", "b/x"), ("a/y", "b/y")], &["a/x", "a/y"]);
        let (b, n) = retarget_body("a/x", "b/x", "See [y](y.md).\n", &r);
        assert_eq!(b, "See [y](y.md).\n");
        assert_eq!(n, 0);
    }

    #[test]
    fn a_link_from_a_moved_document_to_a_document_that_stays_is_recomputed() {
        let r = remap(&[("a/x", "b/c/x")], &["a/x", "a/y"]);
        let (b, n) = retarget_body("a/x", "b/c/x", "See [y](y.md).\n", &r);
        assert_eq!(b, "See [y](../../a/y.md).\n");
        assert_eq!(n, 1);
    }

    #[test]
    fn wikilinks_keep_alias_and_heading() {
        let r = remap(&[("a/x", "b/x")], &["a/x", "c/z"]);
        let (b, n) = retarget_body(
            "c/z",
            "c/z",
            "[[a/x]] [[a/x|alias]] [[a/x#H|alias]] ![[a/x]]\n",
            &r,
        );
        assert_eq!(b, "[[b/x]] [[b/x|alias]] [[b/x#H|alias]] ![[b/x]]\n");
        assert_eq!(n, 4);
    }

    #[test]
    fn a_short_name_changes_only_if_it_no_longer_finds_the_document() {
        // Moved to another folder, same file name: the short name still finds it.
        let r = remap(&[("a/x", "b/x")], &["a/x", "c/z"]);
        assert_eq!(retarget_body("c/z", "c/z", "[[x]]\n", &r).0, "[[x]]\n");
        // Renamed: the short name finds nothing. Use the full path.
        let r = remap(&[("a/x", "a/renamed")], &["a/x", "c/z"]);
        assert_eq!(
            retarget_body("c/z", "c/z", "[[x|alias]]\n", &r).0,
            "[[a/renamed|alias]]\n"
        );
        // Another document with the same file name appears: the short name is ambiguous now.
        let r = remap(&[("a/x", "b/x")], &["a/x", "c/x", "d/z"]);
        assert_eq!(retarget_body("d/z", "d/z", "[[x]]\n", &r).0, "[[x]]\n");
    }

    #[test]
    fn code_and_external_links_stay() {
        let r = remap(&[("a/x", "b/x")], &["a/x", "c/z"]);
        let body = "`[[a/x]]` [e](https://x.org/a/x.md)\n```\n[[a/x]]\n```\n";
        assert_eq!(retarget_body("c/z", "c/z", body, &r), (body.to_string(), 0));
    }

    #[test]
    fn properties_are_retargeted() {
        let r = remap(&[("a/x", "b/x")], &["a/x", "c/z"]);
        let mut d =
            parse("---\ntitle: T\nrelated:\n  - \"[[a/x]]\"\n  - \"[[c/other]]\"\n---\nB\n")
                .unwrap();
        assert_eq!(retarget_properties(&mut d.frontmatter, &r), 1);
        let out = d.render();
        assert!(out.contains("[[b/x]]") && out.contains("[[c/other]]") && !out.contains("[[a/x]]"));
    }
}
