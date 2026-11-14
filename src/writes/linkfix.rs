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
