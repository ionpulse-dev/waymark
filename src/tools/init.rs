//! The `init` tool: set up an empty repository as a memory repository, or add the settings file to a
//! folder that has the marker `.waymark` but no `settings.toml`
//! (see `specs/features/create-memory-repository`).

use crate::config::Kind;
use crate::error::Error;
use crate::gitsync::{commit_message, Change, Verb};
use crate::mcp::context::Context;
use crate::mcp::result::ToolOutput;
use crate::mcp::tool::{Env, Tool};
use crate::memrepo::agents_file::Audience;
use crate::memrepo::init::{derive_slug, files, settings_text, InitInput};
use crate::memrepo::layout::{MARKER, SETTINGS_FILE};
use crate::writes;
use serde_json::{json, Value};

pub struct Init;

const LIMIT_KEYS: [&str; 7] = [
    "access",
    "paused",
    "tools_denied",
    "folders",
    "max_results",
    "max_tokens",
    "max_documents_opened",
];

fn field(name: &str, why: &str) -> Error {
    Error::invalid(format!("The field `{name}` is wrong: {why}"))
        .with_next(format!("Fix the field `{name}` and call init again."))
}

fn is_slug(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 40
        && !s.starts_with('-')
        && !s.ends_with('-')
        && !s.contains("--")
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

fn is_domain(s: &str) -> bool {
    s.contains('.')
        && !s.starts_with('.')
        && !s.ends_with('.')
        && !s.contains("..")
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '.')
}

/// A folder below `agent-memory/`: parts of lowercase letters, digits, and `-`, joined by `/`.
fn check_folder(f: &str) -> Result<(), Error> {
    let ok = !f.is_empty()
        && f.split('/').all(|seg| {
            !seg.is_empty()
                && !seg.starts_with('-')
                && seg
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        });
    if ok {
        Ok(())
    } else {
        Err(field(
            "folders",
            &format!(
                "\"{f}\" must use lowercase letters, digits, and `-` only, in parts joined by `/`"
            ),
        ))
    }
}

fn to_toml(v: &Value) -> toml::Value {
    match v {
        Value::Bool(b) => (*b).into(),
        Value::Number(n) => n
            .as_i64()
            .map(toml::Value::Integer)
            .unwrap_or_else(|| toml::Value::Float(n.as_f64().unwrap_or(0.0))),
        Value::String(s) => s.clone().into(),
        Value::Array(a) => toml::Value::Array(a.iter().map(to_toml).collect()),
        Value::Object(o) => {
            toml::Value::Table(o.iter().map(|(k, v)| (k.clone(), to_toml(v))).collect())
        }
        Value::Null => toml::Value::String(String::new()),
    }
}

fn parse_input(args: &Value) -> Result<InitInput, Error> {
    let name = writes::get_str(args, "name")
        .unwrap_or("")
        .trim()
        .to_string();
    if name.is_empty() || name.chars().count() > 80 {
        return Err(field(
            "name",
            "it must have 1 to 80 characters and not be blank",
        ));
    }
    let slug = match writes::get_str(args, "slug") {
        Some(s) if !is_slug(s) => {
            return Err(field(
                "slug",
                "use lowercase letters, digits, and single `-`, at most 40 characters",
            ))
        }
        Some(s) => s.to_string(),
        None => derive_slug(&name),
    };
    let mut email_domains = Vec::new();
    for d in writes::get_list(args, "email_domains").unwrap_or_default() {
        let d = d.trim().to_lowercase();
        if !is_domain(&d) {
            return Err(field(
                "email_domains",
                &format!("\"{d}\" is not a domain name"),
            ));
        }
        email_domains.push(d);
    }
    let audience = match writes::get_str(args, "audience") {
        None | Some("team") => Audience::Team,
        Some("personal") => Audience::Personal,
        Some(_) => return Err(field("audience", "use `personal` or `team`")),
    };
    let mut limits = toml::Table::new();
    if let Some(o) = args.get("limits").and_then(Value::as_object) {
        for (k, v) in o {
            if !LIMIT_KEYS.contains(&k.as_str()) {
                return Err(field("limits", &format!("the key \"{k}\" is unknown")));
            }
            limits.insert(k.clone(), to_toml(v));
        }
    }
    let mut approval = toml::Table::new();
    if let Some(o) = args.get("approval").and_then(Value::as_object) {
        for (k, v) in o {
            if k != "required_for" && k != "via" {
                return Err(field("approval", &format!("the key \"{k}\" is unknown")));
            }
            approval.insert(k.clone(), to_toml(v));
        }
    }
    let folders = writes::get_list(args, "folders").unwrap_or_default();
    if folders.len() > 50 {
        return Err(field("folders", "at most 50 folders"));
    }
    for f in &folders {
        check_folder(f)?;
    }
    Ok(InitInput {
        name,
        slug,
        email_domains,
        audience,
        limits,
        approval,
        folders,
    })
}

impl Tool for Init {
    fn name(&self) -> &'static str {
        "init"
    }

    fn description(&self) -> &'static str {
        "Sets up an empty repository as a memory repository. Also adds the settings file to a folder that has the older .waymark marker but no settings file. Commits and pushes. Returns id, name, slug, audience, files, commit, and adopted."
    }

    fn kind(&self) -> Kind {
        Kind::Write
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Name of the memory repository. 1 to 80 characters."
                },
                "slug": {
                    "type": "string",
                    "description": "Short name for the memory repository: lowercase letters, digits, and -. Default: from the name."
                },
                "email_domains": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    },
                    "description": "Domains whose email addresses may write, for example example.org."
                },
                "limits": {
                    "type": "object",
                    "description": "Shared limits for the settings file. All keys are optional.",
                    "properties": {
                        "access": {
                            "type": "string",
                            "enum": [
                                "read-only",
                                "read-write"
                            ],
                            "description": "read-only lets agents only read."
                        },
                        "paused": {
                            "type": "boolean",
                            "description": "true pauses all writes."
                        },
                        "tools_denied": {
                            "type": "array",
                            "items": {
                                "type": "string"
                            },
                            "description": "Tool names to switch off."
                        },
                        "folders": {
                            "type": "array",
                            "items": {
                                "type": "string"
                            },
                            "description": "Folders that agents may use. Default: all."
                        },
                        "max_results": {
                            "type": "integer",
                            "description": "Most documents that search returns."
                        },
                        "max_tokens": {
                            "type": "integer",
                            "description": "Most tokens that search returns."
                        },
                        "max_documents_opened": {
                            "type": "integer",
                            "description": "Most documents that show opens in one session."
                        }
                    }
                },
                "approval": {
                    "type": "object",
                    "description": "Approval rules for the settings file. All keys are optional.",
                    "properties": {
                        "required_for": {
                            "type": "array",
                            "items": {
                                "type": "string"
                            },
                            "description": "Tool names, or write, that need approval before they run."
                        },
                        "via": {
                            "type": "string",
                            "enum": [
                                "elicitation",
                                "console"
                            ],
                            "description": "How approval is asked. Default elicitation."
                        }
                    }
                },
                "folders": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    },
                    "description": "Folders to create, such as ops/runbooks. At most 50."
                },
                "audience": {
                    "type": "string",
                    "enum": [
                        "personal",
                        "team"
                    ],
                    "description": "personal or team. Default team."
                },
                "preview": {
                    "type": "boolean",
                    "description": "true: dry run. Writes nothing."
                }
            },
            "required": [
                "name"
            ],
            "additionalProperties": false
        })
    }

    fn call(&self, ctx: &Context, env: &Env, args: Value) -> Result<ToolOutput, Error> {
        let input = parse_input(&args)?;
        let preview = writes::get_bool(&args, "preview");
        let repo = ctx.repo()?;
        ctx.checked_author()?;
        let id = uuid::Uuid::new_v4().to_string();
        // Build the files once, so that a bad value fails before anything is written.
        let planned_new = files(&input, &id)?;

        let (res, (planned, adopted)) = repo.write(preview, |tx| {
            // The marker is at the repository root or in `agent-memory/`.
            let marker_base = ["", "agent-memory/"]
                .into_iter()
                .find(|b| repo.dir().join(format!("{b}{MARKER}")).is_dir());
            let (planned, adopted) = match marker_base {
                Some(base) => {
                    if repo.dir().join(format!("{base}{MARKER}/{SETTINGS_FILE}")).exists() {
                        return Err(Error::validation(
                            "This repository is already a memory repository. Nothing was changed.",
                        )
                        .with_next("Use the other tools, or choose another repository."));
                    }
                    // A folder with the marker and no settings file: add only that file.
                    if args.get("folders").is_some() || args.get("audience").is_some() {
                        return Err(Error::invalid(
                            "The fields `folders` and `audience` do not apply to a folder that has the `.waymark` marker. init only adds the settings file there.",
                        )
                        .with_next("Call init again without `folders` and `audience`."));
                    }
                    let path = format!("{base}{MARKER}/{SETTINGS_FILE}");
                    (vec![(path, settings_text(&input, &id)?)], true)
                }
                None => {
                    if !crate::memrepo::layout::repo_is_empty(repo.dir()) {
                        return Err(Error::validation(
                            "This repository is not empty. init works only on an empty repository, or on a folder that has the `.waymark` marker and no settings file.",
                        )
                        .with_next("Create a new empty repository, or remove the files and commits."));
                    }
                    (planned_new.clone(), false)
                }
            };
            let paths: Vec<String> = planned.iter().map(|(p, _)| p.clone()).collect();
            // A second guard: never overwrite a file.
            for p in &paths {
                if tx.read(p)?.is_some() {
                    return Err(Error::validation(format!(
                        "The file {p} exists already. init does not overwrite files."
                    ))
                    .with_next("Use an empty repository."));
                }
            }
            if !preview {
                let summary = format!(
                    "Set up the memory repository \"{}\"\n\nFiles: {}",
                    input.name,
                    paths.join(", ")
                );
                writes::approve_named(
                    ctx,
                    env,
                    &input.name,
                    None,
                    "init",
                    Kind::Write,
                    &summary,
                    &paths,
                )?;
                for (p, text) in &planned {
                    tx.write_file(p, text)?;
                }
            }
            Ok((
                Change {
                    paths: paths.clone(),
                    message: commit_message(Verb::Init, &input.name),
                },
                (planned, adopted),
            ))
        })?;
        let paths: Vec<String> = planned.iter().map(|(p, _)| p.clone()).collect();

        let message = if preview {
            format!(
                "Preview, nothing was written. This would set up the memory repository \"{}\".",
                input.name
            )
        } else {
            format!(
                "The memory repository \"{}\" is set up and pushed.",
                input.name
            )
        };
        let audience = match input.audience {
            Audience::Personal => "personal",
            Audience::Team => "team",
        };
        let mut out = ToolOutput::new(
            message,
            json!({
                "id": if preview { Value::Null } else { json!(id) },
                "name": input.name,
                "slug": input.slug,
                "audience": audience,
                "root": "agent-memory",
                "files": paths,
                "commit": res.commit,
                "pushed": !preview && res.pushed,
                "adopted": adopted,
            }),
        );
        if input.approval.get("via").and_then(|v| v.as_str()) == Some("console") {
            out = out.notice(
                "The approval channel `console` is not available in local mode, so a write that needs approval fails with `approval_unavailable`.",
            );
        }
        for n in res.notices {
            out = out.notice(n);
        }
        Ok(out)
    }
}
