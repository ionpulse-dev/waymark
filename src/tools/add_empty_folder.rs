//! The `add_empty_folder` tool: create a folder that has no document yet
//! (see `specs/features/folders`).

use crate::config::Kind;
use crate::error::{Code, Error};
use crate::gitsync::Verb;
use crate::mcp::context::Context;
use crate::mcp::result::ToolOutput;
use crate::mcp::tool::{Env, Tool};
use crate::memrepo::folders::GITKEEP;
use crate::writes::{self, FileChange, Plan};
use serde_json::{json, Map, Value};

pub struct AddEmptyFolder;

impl Tool for AddEmptyFolder {
    fn name(&self) -> &'static str {
        "create_folder"
    }

    fn description(&self) -> &'static str {
        "Creates an empty folder. Use it only for a folder that has no document yet: add_document creates the folder of a new document by itself. Commits and pushes. Returns folder and commit."
    }

    fn kind(&self) -> Kind {
        Kind::Write
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "folder": {
                    "type": "string",
                    "description": "The folder below the root, for example ops/runbooks. Lowercase letters, digits, and -."
                },
                "preview": {
                    "type": "boolean",
                    "description": "true shows what the call would do and writes nothing."
                }
            },
            "required": [
                "folder"
            ],
            "additionalProperties": false
        })
    }

    fn call(&self, ctx: &Context, env: &Env, args: Value) -> Result<ToolOutput, Error> {
        let path = writes::check_folder_path(writes::get_str(&args, "folder").unwrap_or(""))?;
        let preview = writes::get_bool(&args, "preview");
        if !env.effective.allows_folder(&format!("{path}/.")) {
            return Err(Error::new(
                Code::AccessDenied,
                format!("The folder \"{path}\" is outside the allowed folders."),
            )
            .with_next("Use a folder that the limits allow."));
        }
        writes::author_string(ctx)?;

        writes::execute(
            ctx,
            env,
            "create_folder",
            Kind::Write,
            preview,
            |_tx, memrepo| {
                if memrepo.id_set().contains(&path) {
                    return Err(Error::validation(format!(
                    "A document with the ID \"{path}\" exists. A folder cannot have the same path."
                ))
                    .with_next("Choose another folder name."));
                }
                if memrepo.folders().iter().any(|f| f.path == path) {
                    return Err(Error::validation(format!(
                        "The folder \"{path}\" exists already. Nothing was changed."
                    ))
                    .with_next("Use the folder, or choose another name."));
                }
                let mut data = Map::new();
                data.insert("folder".into(), json!(path));
                data.insert("folders".into(), json!([path]));
                Ok(Plan {
                    first: None,
                    revert: None,
                    changes: vec![FileChange::WriteFile {
                        path: format!("{path}/{GITKEEP}"),
                        text: String::new(),
                        action: "create",
                    }],
                    verb: Verb::AddFolder,
                    title: path.clone(),
                    commit_body: None,
                    summary: format!("Create the empty folder \"{path}\"."),
                    message: format!("Created the empty folder `{path}`."),
                    data,
                    text: None,
                    notices: Vec::new(),
                })
            },
        )
    }
}
