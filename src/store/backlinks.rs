//! Backlinks: the documents that link to one document.

use super::{Doc, Store};
use crate::format::links::{body_links, BodyLink};

/// The IDs of documents that link to `id` by a body link, a wikilink, or a property.
/// `visible` says which documents the caller may see. Sorted by ID.
pub fn backlinks(store: &Store, id: &str, visible: &dyn Fn(&str) -> bool) -> Vec<String> {
    let name = id.rsplit('/').next().unwrap_or(id);
    let same_name = store
        .docs()
        .filter(|d| d.id.rsplit('/').next() == Some(name))
        .count();
    let resolves =
        |wiki: &str| wiki == id || (!wiki.contains('/') && wiki == name && same_name == 1);
    let mut out: Vec<String> = store
        .docs()
        .filter(|d| d.id != id && visible(&d.id))
        .filter(|d| links_to(d, id, &resolves))
        .map(|d| d.id.clone())
        .collect();
    out.sort();
    out
}

fn links_to(d: &Doc, id: &str, resolves: &dyn Fn(&str) -> bool) -> bool {
    let in_body = body_links(&format!("{}.md", d.id), &d.body)
        .into_iter()
        .any(|l| match l {
            BodyLink::Id(target) => target == id,
            BodyLink::Wiki(name) => resolves(&name),
        });
    in_body
        || ["related", "supersedes"]
            .iter()
            .any(|k| d.property_ids(k).iter().any(|t| resolves(t)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::parse;

    fn doc(id: &str, props: &str, body: &str) -> Doc {
        let text = format!("---\ntitle: T\ndescription: D\nstatus: active\n{props}---\n\n{body}\n");
        let p = parse(&text).unwrap();
        Doc {
            id: id.into(),
            raw: text,
            front: Some(p.frontmatter),
            body: p.body,
        }
    }

    fn store(docs: Vec<Doc>) -> Store {
        Store { docs }
    }

    #[test]
    fn body_links_wikilinks_and_properties_count() {
        let s = store(vec![
            doc("ops/target", "", "x"),
            doc("a", "", "See [t](ops/target.md)."),
            doc("b", "", "See [[ops/target]]."),
            doc("c", "related:\n  - \"[[ops/target]]\"\n", "x"),
            doc("d", "", "Nothing."),
        ]);
        assert_eq!(backlinks(&s, "ops/target", &|_| true), vec!["a", "b", "c"]);
    }

    #[test]
    fn index_files_and_hidden_documents_are_ignored() {
        let s = store(vec![
            doc("ops/target", "", "x"),
            doc("index", "tags: [index]\n", "[[ops/target]]"),
            doc("secret/x", "", "[[ops/target]]"),
        ]);
        assert!(backlinks(&s, "ops/target", &|id| !id.starts_with("secret")).is_empty());
    }
}
