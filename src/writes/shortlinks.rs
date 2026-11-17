//! Short wikilinks keep their target when new IDs appear (see `specs/features/document-names`).
//!
//! A wikilink with a short path, such as `[[readme]]`, finds a document only when exactly one
//! document has that file name. Before a write gives a second document the same file name, every
//! such link to the first document gets its full path (section 8 of the standard).

use super::linkfix::{retarget_body, Remap};
use super::{touch, FileChange};
use crate::error::{Code, Error, Result};
use crate::format;
use crate::mcp::tool::Env;
use crate::memrepo::MemRepo;
use serde_json::{json, Value};
use std::collections::HashSet;

/// Add the link fixes for the documents `added` to `changes`.
/// A document that `changes` writes already is fixed in that text. Other documents are read from
/// the folder. `skip` says which IDs are not rewritten: documents with pending manual edits. The
/// new IDs are never rewritten. Returns the `links_updated` entries.
#[allow(clippy::too_many_arguments)]
pub fn add_fixes_for_new_ids(
    memrepo: &MemRepo,
    env: Option<&Env>,
    old_ids: &HashSet<String>,
    added: &[String],
    skip: &dyn Fn(&str) -> bool,
    today: &str,
    author: &str,
    changes: &mut Vec<FileChange>,
) -> Result<Vec<Value>> {
    let names: HashSet<&str> = added
        .iter()
        .map(|id| id.rsplit('/').next().unwrap_or(id))
        .collect();
    // Only when a new file name is taken already can a short link lose its target.
    let taken = old_ids
        .iter()
        .any(|id| !added.contains(id) && names.contains(id.rsplit('/').next().unwrap_or(id)));
    if !taken {
        return Ok(Vec::new());
    }
    let remap = Remap::added(old_ids, added);
    let mut ids: Vec<&String> = old_ids.iter().collect();
    ids.sort();
    let mut updated = Vec::new();
    for id in ids {
        if skip(id) || added.contains(id) {
            continue;
        }
        // The newest planned text of the document, if any.
        let planned = changes
            .iter()
            .rposition(|c| matches!(c, FileChange::Write { id: cid, .. } if cid == id));
        let text = match planned {
            Some(i) => match &changes[i] {
                FileChange::Write { text, .. } => text.clone(),
                _ => continue,
            },
            None => match memrepo.read(id) {
                Ok(t) => t,
                Err(_) => continue,
            },
        };
        let Ok(mut doc) = format::parse(&text) else {
            continue;
        };
        let (body, count) = retarget_body(id, id, &doc.body, &remap);
        if count == 0 {
            continue;
        }
        if let Some(env) = env {
            if !env.effective.allows_folder(id) {
                return Err(Error::new(
                    Code::AccessDenied,
                    format!("The document \"{id}\" has a short link that must get its full path, and it is outside the allowed folders."),
                )
                .with_next("Choose another name, or ask someone with wider access."));
            }
        }
        doc.body = body;
        touch(&mut doc, today, author);
        updated.push(json!({"id": id, "count": count}));
        match planned {
            Some(i) => {
                if let FileChange::Write { text, .. } = &mut changes[i] {
                    *text = doc.render();
                }
            }
            None => changes.push(FileChange::Write {
                id: id.clone(),
                text: doc.render(),
                action: "update",
            }),
        }
    }
    Ok(updated)
}
