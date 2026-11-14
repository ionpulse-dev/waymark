//! The result of a tool call: `{ message, notices?, settings, ...data }`.

use serde_json::{json, Map, Value};

#[derive(Debug, Clone)]
pub struct ToolOutput {
    pub message: String,
    pub notices: Vec<String>,
    /// The tool's own fields. Must be a JSON object or `null`.
    pub data: Value,
}

impl ToolOutput {
    pub fn new(message: impl Into<String>, data: Value) -> ToolOutput {
        ToolOutput {
            message: message.into(),
            notices: Vec::new(),
            data,
        }
    }

    pub fn notice(mut self, text: impl Into<String>) -> ToolOutput {
        self.notices.push(text.into());
        self
    }

    /// The JSON form. `settings` is added by the registry. Data fields never replace the three common fields.
    pub fn to_json(&self, settings: Value, extra_notices: &[String]) -> Value {
        let mut map = Map::new();
        if let Value::Object(d) = &self.data {
            for (k, v) in d {
                map.insert(k.clone(), v.clone());
            }
        }
        map.insert("message".into(), json!(self.message));
        let notices: Vec<&String> = self.notices.iter().chain(extra_notices.iter()).collect();
        if !notices.is_empty() {
            map.insert("notices".into(), json!(notices));
        }
        map.insert("settings".into(), settings);
        Value::Object(map)
    }
}
