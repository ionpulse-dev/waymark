//! Shared parts of the read tools: levels, caps, and the folder and limit arguments.

use crate::error::{Error, Result};
use crate::format::count;
use crate::store::Doc;
use serde_json::{json, Map, Value};

pub fn folders_arg(args: &Value) -> Vec<String> {
    crate::writes::get_list(args, "folders").unwrap_or_default()
}

/// The `limit` argument, never above the cap. A limit below 1 is `invalid_arguments`.
pub fn limit_arg(args: &Value, cap: usize) -> Result<usize> {
    match args.get("limit").and_then(Value::as_i64) {
        None => Ok(cap),
        Some(n) if n >= 1 => Ok((n as usize).min(cap)),
        Some(_) => Err(Error::invalid("\"limit\" must be 1 or more.")
            .with_next("Send a limit of 1 or more, or leave it out.")),
    }
}

pub fn level_arg(args: &Value, default: u8) -> u8 {
    args.get("level")
        .and_then(Value::as_u64)
        .map(|n| n as u8)
        .unwrap_or(default)
}

/// One entry of a result at a level (see the requirements of `search-and-read`).
pub fn entry(d: &Doc, level: u8) -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("id".into(), json!(d.id));
    if level >= 1 {
        m.insert("title".into(), json!(d.title()));
        m.insert("status".into(), json!(d.status()));
        m.insert("updated".into(), json!(d.updated()));
        if d.disputed() {
            m.insert("disputed".into(), json!(true));
            m.insert(
                "dispute_reason".into(),
                json!(d.text("dispute_reason").unwrap_or("")),
            );
        }
    }
    if level >= 2 {
        m.insert("description".into(), json!(d.description()));
    }
    m
}

/// The estimated tokens of an entry: all its text.
pub fn tokens(entry: &Map<String, Value>) -> usize {
    let words: usize = entry
        .values()
        .filter_map(Value::as_str)
        .map(count::words)
        .sum();
    count::estimated_tokens(words.max(1))
}

/// Keep entries while the count and the token budget allow. The first entry always stays.
pub fn cap(entries: Vec<Map<String, Value>>, max_count: usize, max_tokens: usize) -> Vec<Value> {
    let mut out = Vec::new();
    let mut used = 0;
    for e in entries {
        let t = tokens(&e);
        if !out.is_empty() && (out.len() >= max_count || used + t > max_tokens) {
            break;
        }
        used += t;
        out.push(Value::Object(e));
    }
    out
}

pub fn status_ok(filter: &str, status: &str) -> bool {
    filter == "any" || filter == status
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(text: &str) -> Map<String, Value> {
        let mut m = Map::new();
        m.insert("id".into(), json!("a"));
        m.insert("description".into(), json!(text));
        m
    }

    #[test]
    fn the_count_cap_applies() {
        let v = cap(vec![e("x"), e("x"), e("x")], 2, 1000);
        assert_eq!(v.len(), 2);
    }

    #[test]
    fn the_token_cap_applies_and_the_first_entry_stays() {
        let long = "word ".repeat(100);
        assert_eq!(cap(vec![e(&long), e(&long)], 5, 50).len(), 1);
        assert_eq!(cap(vec![e("a"), e(&long)], 5, 50).len(), 1);
        assert_eq!(cap(vec![e("a"), e("b")], 5, 50).len(), 2);
    }

    #[test]
    fn a_limit_is_never_above_the_cap() {
        assert_eq!(limit_arg(&json!({"limit": 50}), 5).unwrap(), 5);
        assert_eq!(limit_arg(&json!({"limit": 2}), 5).unwrap(), 2);
        assert_eq!(limit_arg(&json!({}), 5).unwrap(), 5);
        assert!(limit_arg(&json!({"limit": 0}), 5).is_err());
    }
}
