//! The error of a tool call and its MCP form (see `specs/foundation/mcp-server`).

use crate::error::Error;
use serde_json::{json, Value};

/// A tool error is the shared `Error`: a code, a message, and a `next` sentence.
pub type ToolError = Error;

/// The structured content of a failed call: `{ code, message, next? }`, plus `settings` and `notices`.
pub fn error_json(e: &Error, settings: Option<Value>, notices: &[String]) -> Value {
    use crate::identity::{is_login_failure, redact_text, LOGIN_NEXT};
    let (message, next) =
        if e.code == crate::error::Code::SyncFailed && is_login_failure(&e.message) {
            (
                "Git cannot sign in to this repository.".to_string(),
                Some(LOGIN_NEXT.to_string()),
            )
        } else {
            (e.message.clone(), e.next.clone())
        };
    let mut v = json!({ "code": e.code.as_str(), "message": redact_text(&message) });
    if let Some(next) = &next {
        v["next"] = json!(redact_text(next));
    }
    if let Some(d) = &e.data {
        v["data"] = d.clone();
    }
    if let Some(s) = settings {
        v["settings"] = s;
    }
    if !notices.is_empty() {
        v["notices"] = json!(notices);
    }
    v
}
