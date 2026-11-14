//! Links (section 8 of the standard).

use super::finding::Finding;
use super::frontmatter::Frontmatter;
use regex::Regex;
use std::collections::HashSet;
use std::sync::OnceLock;

/// The set of documents that a link can point to.
pub trait DocSet {
    /// A document with this ID exists.
    fn contains(&self, id: &str) -> bool;
    /// A document with this file name (without folders and `.md`) exists.
    fn contains_name(&self, name: &str) -> bool;
}

impl DocSet for HashSet<String> {
    fn contains(&self, id: &str) -> bool {
        HashSet::contains(self, id)
    }
    fn contains_name(&self, name: &str) -> bool {
        let suffix = format!("/{name}");
        self.iter().any(|id| id == name || id.ends_with(&suffix))
    }
}

fn md_link_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"!?\[[^\]\n]*\]\(([^)\s]+)(?:\s+[^)]*)?\)").unwrap())
}

fn wikilink_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"!?\[\[([^\[\]\n|#]+)(?:#[^\[\]\n|]*)?(?:\|[^\[\]\n]*)?\]\]").unwrap()
    })
}

/// The body without fenced code blocks and inline code, so that code is not read as links.
pub fn strip_code(body: &str) -> String {
    let mut out = String::with_capacity(body.len());
    let mut in_fence = false;
    for line in body.split_inclusive('\n') {
        let t = line.trim_start();
        if t.starts_with("```") || t.starts_with("~~~") {
            in_fence = !in_fence;
            out.push('\n');
            continue;
        }
        if in_fence {
            out.push('\n');
            continue;
        }
        // inline code spans
        let mut res = String::new();
        let mut in_code = false;
        for c in line.chars() {
            if c == '`' {
                in_code = !in_code;
                continue;
            }
            if !in_code {
                res.push(c);
            }
        }
        out.push_str(&res);
    }
    out
}

fn normalize(base_dir: &str, target: &str) -> Option<String> {
    let mut parts: Vec<&str> = if target.starts_with('/') || base_dir.is_empty() {
        Vec::new()
    } else {
        base_dir.split('/').collect()
    };
    for seg in target.trim_start_matches('/').split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            s => parts.push(s),
        }
    }
    Some(parts.join("/"))
}

/// A link target found in a body.
#[derive(Debug, PartialEq, Eq)]
pub enum BodyLink {
    /// A document ID resolved from a relative Markdown link.
    Id(String),
    /// A wikilink: the text between the brackets, without alias or heading.
    Wiki(String),
}

/// All links in a body that point to documents. `path` is the path of the document.
pub fn body_links(path: &str, body: &str) -> Vec<BodyLink> {
    let text = strip_code(body);
    let base_dir = path.rsplit_once('/').map(|(d, _)| d).unwrap_or("");
    let mut out = Vec::new();
    for c in md_link_re().captures_iter(&text) {
        let raw = &c[1];
        if raw.contains("://") || raw.starts_with("mailto:") || raw.starts_with('#') {
            continue;
        }
        let target = raw.split('#').next().unwrap_or("");
        let Some(stem) = target.strip_suffix(".md") else {
            continue;
        };
        if let Some(id) = normalize(base_dir, stem) {
            out.push(BodyLink::Id(id));
        }
    }
    for c in wikilink_re().captures_iter(&text) {
        let name = c[1].trim();
        // A wikilink to a file with another extension (an image, for example) is not a document.
        if let Some((_, ext)) = name.rsplit_once('.') {
            if ext != "md" && ext.chars().all(|ch| ch.is_ascii_alphanumeric()) && ext.len() <= 5 {
                continue;
            }
        }
        out.push(BodyLink::Wiki(name.trim_end_matches(".md").to_string()));
    }
    out
}

/// Check links against the set of documents.
/// A missing target in a property is an error. In the body it is a warning.
pub fn check_links(path: &str, fm: &Frontmatter, body: &str, docs: &dyn DocSet) -> Vec<Finding> {
    let mut out = Vec::new();
    for field in ["related", "supersedes"] {
        if let Some(items) = fm.get_list(field) {
            for item in items {
                if let Some(id) = super::fields::property_link_target(item) {
                    if !docs.contains(id) {
                        out.push(Finding::error(
                            path,
                            "broken-property-link",
                            format!("The link to \"{id}\" in \"{field}\" has no target."),
                        ));
                    }
                }
            }
        }
    }
    for link in body_links(path, body) {
        match link {
            BodyLink::Id(id) => {
                if !docs.contains(&id) {
                    out.push(Finding::warning(
                        path,
                        "broken-link",
                        format!("The link to \"{id}.md\" has no target."),
                    ));
                }
            }
            BodyLink::Wiki(name) => {
                let found = if name.contains('/') {
                    docs.contains(&name)
                } else {
                    docs.contains_name(&name)
                };
                if !found {
                    out.push(Finding::warning(
                        path,
                        "broken-link",
                        format!("The link to \"[[{name}]]\" has no target."),
                    ));
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(ids: &[&str]) -> HashSet<String> {
        ids.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn relative_links_resolve_from_the_document_folder() {
        let l = body_links(
            "backend/payments/a.md",
            "See [x](retry-policy.md) and [y](../auth/login.md#top).",
        );
        assert_eq!(
            l,
            vec![
                BodyLink::Id("backend/payments/retry-policy".into()),
                BodyLink::Id("backend/auth/login".into())
            ]
        );
    }

    #[test]
    fn external_and_non_document_links_are_ignored() {
        let l = body_links(
            "a.md",
            "[w](https://example.org/x.md) [i](img.png) [m](mailto:a@b.c) [h](#head)",
        );
        assert!(l.is_empty());
    }

    #[test]
    fn code_is_not_read_as_links() {
        let l = body_links("a.md", "`[x](a.md)`\n```\n[y](b.md)\n```\n[z](c.md)");
        assert_eq!(l, vec![BodyLink::Id("c".into())]);
    }

    #[test]
    fn wikilinks_with_alias_and_heading() {
        let l = body_links("a.md", "[[b/c|alias]] [[d#h]] ![[pic.png]] ![[e]]");
        assert_eq!(
            l,
            vec![
                BodyLink::Wiki("b/c".into()),
                BodyLink::Wiki("d".into()),
                BodyLink::Wiki("e".into())
            ]
        );
    }

    #[test]
    fn broken_body_link_is_a_warning_and_a_broken_property_link_is_an_error() {
        let d =
            crate::format::parse("---\nrelated:\n  - \"[[x/missing]]\"\n---\nSee [a](gone.md).\n")
                .unwrap();
        let f = check_links("x/doc.md", &d.frontmatter, &d.body, &set(&["x/doc"]));
        assert_eq!(f.len(), 2);
        assert_eq!(f[0].rule, "broken-property-link");
        assert!(f[0].is_error());
        assert_eq!(f[1].rule, "broken-link");
        assert!(!f[1].is_error());
    }

    #[test]
    fn existing_targets_pass() {
        let d =
            crate::format::parse("---\nrelated:\n  - \"[[x/b]]\"\n---\nSee [a](b.md) and [[b]].\n")
                .unwrap();
        let f = check_links("x/doc.md", &d.frontmatter, &d.body, &set(&["x/doc", "x/b"]));
        assert!(f.is_empty(), "{f:?}");
    }
}
