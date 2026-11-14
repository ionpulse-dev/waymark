//! A small check of tool arguments against the tool's input schema.
//!
//! It supports what the tool schemas use: `type` (string, integer, number, boolean, array, object),
//! `required`, `properties`, `enum`, and `additionalProperties: false`.

use crate::error::Error;
use serde_json::Value;

fn type_ok(t: &str, v: &Value) -> bool {
    match t {
        "string" => v.is_string(),
        "integer" => v.is_i64() || v.is_u64(),
        "number" => v.is_number(),
        "boolean" => v.is_boolean(),
        "array" => v.is_array(),
        "object" => v.is_object(),
        "null" => v.is_null(),
        _ => true,
    }
}

fn check_value(path: &str, schema: &Value, v: &Value) -> Result<(), Error> {
    if let Some(t) = schema.get("type").and_then(|t| t.as_str()) {
        if !type_ok(t, v) {
            return Err(Error::invalid(format!("\"{path}\" must be of type {t}."))
                .with_next(format!("Send \"{path}\" as {t}.")));
        }
    }
    if let Some(options) = schema.get("enum").and_then(|e| e.as_array()) {
        if !options.contains(v) {
            let names: Vec<String> = options
                .iter()
                .map(|o| {
                    o.as_str()
                        .map(String::from)
                        .unwrap_or_else(|| o.to_string())
                })
                .collect();
            return Err(
                Error::invalid(format!("\"{path}\" has a value that is not allowed."))
                    .with_next(format!("Use one of: {}.", names.join(", "))),
            );
        }
    }
    if let (Some(items), Some(arr)) = (schema.get("items"), v.as_array()) {
        for (i, x) in arr.iter().enumerate() {
            check_value(&format!("{path}[{i}]"), items, x)?;
        }
    }
    if v.is_object() {
        check_object(path, schema, v)?;
    }
    Ok(())
}

fn check_object(path: &str, schema: &Value, v: &Value) -> Result<(), Error> {
    let obj = v.as_object().expect("checked by the caller");
    let props = schema.get("properties").and_then(|p| p.as_object());
    if let Some(req) = schema.get("required").and_then(|r| r.as_array()) {
        for r in req.iter().filter_map(|r| r.as_str()) {
            if !obj.contains_key(r) {
                let at = if path.is_empty() {
                    r.to_string()
                } else {
                    format!("{path}.{r}")
                };
                return Err(Error::invalid(format!("The argument \"{at}\" is missing."))
                    .with_next(format!("Add \"{at}\" to the call.")));
            }
        }
    }
    for (k, val) in obj {
        let at = if path.is_empty() {
            k.clone()
        } else {
            format!("{path}.{k}")
        };
        match props.and_then(|p| p.get(k)) {
            Some(s) => check_value(&at, s, val)?,
            None => {
                if schema.get("additionalProperties") == Some(&Value::Bool(false)) {
                    return Err(
                        Error::invalid(format!("The argument \"{at}\" is not known."))
                            .with_next("Remove it, or check the tool's input schema."),
                    );
                }
            }
        }
    }
    Ok(())
}

/// Check the arguments of a call. Missing arguments count as an empty object.
pub fn check_arguments(schema: &Value, args: &Value) -> Result<(), Error> {
    if !args.is_object() {
        return Err(Error::invalid("The arguments must be a JSON object.")
            .with_next("Send the arguments as an object."));
    }
    check_object("", schema, args)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Code;
    use serde_json::json;

    fn schema() -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {"type": "string"},
                "limit": {"type": "integer"},
                "mode": {"type": "string", "enum": ["a", "b"]},
                "tags": {"type": "array", "items": {"type": "string"}}
            },
            "required": ["query"],
            "additionalProperties": false
        })
    }

    #[test]
    fn accepts_a_valid_call() {
        assert!(
            check_arguments(&schema(), &json!({"query": "x", "limit": 3, "tags": ["a"]})).is_ok()
        );
    }

    #[test]
    fn rejects_missing_wrong_unknown_and_bad_enum() {
        for bad in [
            json!({}),
            json!({"query": 1}),
            json!({"query": "x", "extra": 1}),
            json!({"query": "x", "mode": "c"}),
            json!({"query": "x", "tags": [1]}),
        ] {
            let e = check_arguments(&schema(), &bad).unwrap_err();
            assert_eq!(e.code, Code::InvalidArguments);
            assert!(e.next.is_some());
        }
    }

    #[test]
    fn arguments_must_be_an_object() {
        assert!(check_arguments(&schema(), &json!([1])).is_err());
    }
}
