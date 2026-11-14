//! The error of a tool call and its MCP form (see `specs/foundation/mcp-server`).

use crate::error::Error;
use serde_json::{json, Value};

/// A tool error is the shared `Error`: a code, a message, and a `next` sentence.
pub type ToolError = Error;

/// The structured content of a failed call: `{ code, message, next? }`, plus `settings` and `notices`.
pub fn error_json(e: &Error, settings: Option<Value>, notices: &[String]) -> Value {
    let mut v = json!({ "code": e.code.as_str(), "message": e.message });
    if let Some(next) = &e.next {
        v["next"] = json!(next);
    }
    if let Some(s) = settings {
        v["settings"] = s;
    }
    if !notices.is_empty() {
        v["notices"] = json!(notices);
    }
    v
}
