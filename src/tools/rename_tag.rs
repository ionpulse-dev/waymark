//! The `rename_tag` tool: change a tag in every document (see `specs/features/manage-tags`).

use crate::config::Kind;
use crate::error::Error;
use crate::gitsync::Verb;
use crate::mcp::context::Context;
use crate::mcp::result::ToolOutput;
use crate::mcp::tool::{Env, Tool};
use crate::store::Store;
use crate::writes::{self, FileChange, Plan};
use serde_json::{json, Map, Value};

pub struct RenameTag;

const SHOWN_IDS: usize = 20;

impl Tool for RenameTag {
    fn name(&self) -> &'static str {
        "rename_tag"
    }

    fn description(&self) -> &'static str {
        "Renames a tag in every document that has it, in one commit. When a document has the new tag already, the two tags merge. Commits and pushes. Returns from, to, documents, ids, and commit."
    }

    fn kind(&self) -> Kind {
        Kind::Write
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "from": {"type": "string", "description": "The tag to change."},
                "to": {"type": "string", "description": "The new tag. No # and no spaces."},
                "preview": {"type": "boolean", "description": "true: dry run. Writes nothing."}
            },
            "required": ["from", "to"],
            "additionalProperties": false
        })
    }

    fn call(&self, ctx: &Context, env: &Env, args: Value) -> Result<ToolOutput, Error> {
        let from = writes::get_str(&args, "from")
            .unwrap_or("")
            .trim()
            .to_string();
        let to = writes::get_str(&args, "to")
            .unwrap_or("")
            .trim()
            .to_string();
        let preview = writes::get_bool(&args, "preview");
        writes::check_tag(&from)?;
        writes::check_tag(&to)?;
        if from == to {
            return Err(Error::invalid("The new tag is the same as the old tag.")
                .with_next("Send a different tag in to."));
        }
        let author = writes::author_string(ctx)?;
        let today = writes::today();

        writes::execute(
            ctx,
            env,
            "rename_tag",
            Kind::Write,
            preview,
            |tx, memrepo| {
                let store = Store::load(memrepo);
                let mut ids: Vec<String> = store
                    .docs()
                    .filter(|d| env.effective.allows_folder(&d.id) && d.tags().contains(&from))
                    .map(|d| d.id.clone())
                    .collect();
                ids.sort();
                if ids.is_empty() {
                    return Err(
                        Error::not_found(format!("No document has the tag \"{from}\"."))
                            .with_next("Use list_tags to see the tags."),
                    );
                }
                let id_set = memrepo.id_set();
                let mut changes = Vec::new();
                let mut notices = Vec::new();
                for id in &ids {
                    let (_, mut doc) = writes::read_doc(tx, memrepo, id)?;
                    let mut tags: Vec<String> = Vec::new();
                    for t in doc.frontmatter.get_list("tags").unwrap_or(&[]) {
                        let t = if *t == from { to.clone() } else { t.clone() };
                        if !tags.contains(&t) {
                            tags.push(t);
                        }
                    }
                    doc.frontmatter.set_list("tags", &tags);
                    writes::touch(&mut doc, &today, &author);
                    let text = doc.render();
                    let mut problems = writes::Problems::default();
                    writes::check_document(
                        memrepo,
                        id,
                        &text,
                        &id_set,
                        &[("tags", tags.join("\n"))],
                        None,
                        &mut problems,
                        &mut notices,
                    )?;
                    if !problems.is_empty() {
                        let e = problems.into_error();
                        return Err(Error::validation(format!(
                            "The document \"{id}\" would not pass the checks. Nothing was changed.\n{}",
                            e.message
                        ))
                        .with_next(e.next.unwrap_or_default()));
                    }
                    changes.push(FileChange::Write {
                        id: id.clone(),
                        text,
                        action: "update",
                    });
                }
                let n = ids.len();
                let mut data = Map::new();
                data.insert("from".into(), json!(from));
                data.insert("to".into(), json!(to));
                data.insert("documents".into(), json!(n));
                data.insert(
                    "ids".into(),
                    json!(ids.iter().take(SHOWN_IDS).collect::<Vec<_>>()),
                );
                let plural = if n == 1 { "document" } else { "documents" };
                Ok(Plan {
                    first: None,
                    revert: None,
                    changes,
                    verb: Verb::RenameTag,
                    title: format!("{from} to {to}"),
                    commit_body: None,
                    summary: format!("Rename the tag \"{from}\" to \"{to}\" in {n} {plural}"),
                    message: format!("Renamed the tag \"{from}\" to \"{to}\" in {n} {plural}."),
                    data,
                    text: None,
                    notices,
                })
            },
        )
    }
}
