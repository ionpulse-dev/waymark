//! The `Tool` trait and the registry (see `specs/foundation/mcp-server`).

use super::context::Context;
use super::error::error_json;
use super::result::ToolOutput;
use super::schema::check_arguments;
use crate::config::{ConfigError, Denied, Effective, Kind};
use crate::error::{Code, Error};
use serde_json::{json, Value};

/// What a tool sees of the limits for its call.
pub struct Env {
    pub effective: Effective,
    pub problem: Option<ConfigError>,
}

pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    /// Short and plain. It says what the tool does and what it returns, and gives no advice.
    fn description(&self) -> &'static str;
    fn kind(&self) -> Kind;
    /// A JSON Schema for the arguments. Use `additionalProperties: false`.
    fn input_schema(&self) -> Value;
    /// A tool that runs and is listed in every limit state (only `status`).
    fn always_listed(&self) -> bool {
        false
    }
    /// A tool that exists only when the knowledge base uses verification.
    fn needs_verification(&self) -> bool {
        false
    }
    fn call(&self, ctx: &Context, env: &Env, args: Value) -> Result<ToolOutput, Error>;
}

/// The outcome of a call: the structured content and the error flag.
pub struct Outcome {
    pub json: Value,
    pub is_error: bool,
}

/// The call names no tool that this server lists.
#[derive(Debug)]
pub struct UnknownTool(pub String);

pub struct Registry {
    tools: Vec<Box<dyn Tool>>,
}

impl Registry {
    pub fn new(tools: Vec<Box<dyn Tool>>) -> Registry {
        Registry { tools }
    }

    pub fn all_names(&self) -> Vec<&'static str> {
        self.tools.iter().map(|t| t.name()).collect()
    }

    fn visible(&self, t: &dyn Tool, env: &Env, verification: bool) -> bool {
        if t.needs_verification() && !verification {
            return false;
        }
        t.always_listed() || env.effective.lists_tool(t.name(), t.kind())
    }

    fn env(&self, ctx: &Context) -> (Env, bool, Vec<String>) {
        let kb = ctx.kb_if_ready();
        let limit = ctx.limits(kb.as_ref());
        let verification = kb
            .as_ref()
            .map(|k| k.settings.verification)
            .unwrap_or(false);
        let notices = kb.map(|k| k.settings.notices).unwrap_or_default();
        (
            Env {
                effective: limit.effective,
                problem: limit.problem,
            },
            verification,
            notices,
        )
    }

    /// The tools that the limits allow. A broken config file lists the read tools and `status`.
    pub fn list(&self, ctx: &Context) -> Vec<&dyn Tool> {
        let (env, verification, _) = self.env(ctx);
        self.tools
            .iter()
            .map(|t| t.as_ref())
            .filter(|t| self.visible(*t, &env, verification))
            .collect()
    }

    pub fn call(&self, ctx: &Context, name: &str, args: Value) -> Result<Outcome, UnknownTool> {
        let tool = self
            .tools
            .iter()
            .find(|t| t.name() == name)
            .ok_or_else(|| UnknownTool(name.to_string()))?;
        let (env, verification, mut notices) = self.env(ctx);
        if env.effective.paused {
            notices.insert(0, "Writes are paused.".to_string());
        }
        let settings = env.effective.to_json();
        let fail = |e: Error, notices: &[String]| {
            // Every result, also an error, passes the filter of notices.
            if tool.name() != "status" {
                ctx.set_session_error(e.message.clone());
            }
            let all = crate::notices::finalize(ctx, tool.name(), notices.to_vec());
            Outcome {
                json: error_json(&e, Some(settings.clone()), &all),
                is_error: true,
            }
        };
        // A tool of verification is not listed when verification is off. A direct call is denied.
        if tool.needs_verification() && !verification {
            return Ok(fail(
                Error::new(
                    Code::AccessDenied,
                    "Verification is off in this knowledge base.",
                )
                .with_next(
                    "The owner turns it on with verification = true in .mkbase/waymark.toml.",
                ),
                &notices,
            ));
        }

        if !tool.always_listed() {
            if let Some(p) = &env.problem {
                return Ok(fail(
                    Error::new(
                        Code::SetupProblem,
                        format!(
                            "The settings in {} cannot be used: {}",
                            p.path.display(),
                            p.message
                        ),
                    )
                    .with_next("Fix or remove that file, then call status."),
                    &notices,
                ));
            }
            if let Err(d) = env.effective.check_tool(tool.name(), tool.kind()) {
                let e = match d {
                    Denied::AccessDenied(m) => Error::new(Code::AccessDenied, m).with_next(
                        "A limit blocks this tool. Ask the owner of the setup to change it.",
                    ),
                    Denied::Paused => Error::new(Code::Paused, "Writes are paused. Reads still work.")
                        .with_next("Only the person can resume writes: set paused = false in the file that set it, for example ~/.config/waymark/config.toml."),
                };
                return Ok(fail(e, &notices));
            }
        }
        // A missing `arguments` field means no arguments.
        let args = if args.is_null() { json!({}) } else { args };
        if let Err(e) = check_arguments(&tool.input_schema(), &args) {
            return Ok(fail(e, &notices));
        }
        match tool.call(ctx, &env, args) {
            Ok(mut out) => {
                let mut all = std::mem::take(&mut out.notices);
                all.extend(std::mem::take(&mut notices));
                let all = crate::notices::finalize(ctx, tool.name(), all);
                let json = out.to_json(settings, &all);
                Ok(Outcome {
                    json,
                    is_error: false,
                })
            }
            Err(e) => Ok(fail(e, &notices)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Paths;
    use crate::mcp::args::Args;
    use std::path::PathBuf;

    struct Fake(&'static str, Kind, bool);

    impl Tool for Fake {
        fn name(&self) -> &'static str {
            self.0
        }
        fn description(&self) -> &'static str {
            "fake"
        }
        fn kind(&self) -> Kind {
            self.1
        }
        fn input_schema(&self) -> Value {
            json!({"type": "object", "properties": {}, "additionalProperties": false})
        }
        fn needs_verification(&self) -> bool {
            self.2
        }
        fn call(&self, _: &Context, _: &Env, _: Value) -> Result<ToolOutput, Error> {
            Ok(ToolOutput::new("done", json!({"x": 1})))
        }
    }

    fn registry() -> Registry {
        Registry::new(vec![
            Box::new(Fake("search", Kind::Read, false)),
            Box::new(Fake("add", Kind::Write, false)),
            Box::new(Fake("delete", Kind::Destructive, false)),
            Box::new(Fake("verify", Kind::Write, true)),
        ])
    }

    fn ctx(args: &[&str]) -> Context {
        use clap::Parser;
        let mut v = vec!["waymark"];
        v.extend(args);
        let none = PathBuf::from("/nonexistent/waymark-test");
        Context::with_paths(
            Args::parse_from(v),
            Paths {
                system: vec![none.join("system.toml")],
                personal: none.join("personal.toml"),
            },
        )
    }

    fn names(r: &Registry, c: &Context) -> Vec<&'static str> {
        r.list(c).iter().map(|t| t.name()).collect()
    }

    #[test]
    fn read_write_lists_write_tools_but_not_verification_tools() {
        let c = ctx(&[]);
        assert_eq!(names(&registry(), &c), ["search", "add", "delete"]);
    }

    #[test]
    fn read_only_hides_write_tools_and_a_call_is_denied() {
        let c = ctx(&["--access", "read-only"]);
        let r = registry();
        assert_eq!(names(&r, &c), ["search"]);
        let out = r.call(&c, "add", json!({})).unwrap();
        assert!(out.is_error);
        assert_eq!(out.json["code"], "access_denied");
        assert!(out.json["next"].is_string());
    }

    #[test]
    fn no_delete_hides_only_delete() {
        let c = ctx(&["--no-delete"]);
        assert_eq!(names(&registry(), &c), ["search", "add"]);
    }

    #[test]
    fn a_verification_tool_is_denied_when_verification_is_off() {
        let c = ctx(&[]);
        let out = registry().call(&c, "verify", json!({})).unwrap();
        assert!(out.is_error);
        assert_eq!(out.json["code"], "access_denied");
    }

    #[test]
    fn an_unknown_name_is_unknown() {
        let c = ctx(&[]);
        assert!(registry().call(&c, "set_limits", json!({})).is_err());
    }

    #[test]
    fn a_success_has_message_and_settings_and_the_tool_data() {
        let c = ctx(&[]);
        let out = registry().call(&c, "search", Value::Null).unwrap();
        assert!(!out.is_error);
        assert_eq!(out.json["message"], "done");
        assert_eq!(out.json["x"], 1);
        assert!(out.json["settings"].is_object());
    }

    #[test]
    fn a_wrong_argument_is_invalid_arguments() {
        let c = ctx(&[]);
        let out = registry().call(&c, "search", json!({"q": 1})).unwrap();
        assert_eq!(out.json["code"], "invalid_arguments");
    }
}
