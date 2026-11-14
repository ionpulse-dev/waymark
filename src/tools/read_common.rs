//! Shared parts of the read tools: levels, caps, and the folder and limit arguments.

use crate::error::{Error, Result};
use crate::format::count;
use crate::store::Doc;
use chrono::{DateTime, NaiveDate, Utc};
use serde_json::{json, Map, Value};

/// A time from an argument such as `since`: `YYYY-MM-DD` (the start of that day, UTC) or an
/// RFC 3339 time. A time in the future is `invalid_arguments`, unless `allow_future` is set.
pub fn parse_when(name: &str, text: &str, allow_future: bool) -> Result<DateTime<Utc>> {
    let bad = || {
        Error::invalid(format!("\"{name}\" is not a date or a time."))
            .with_next("Use YYYY-MM-DD or a time like 2026-11-10T09:00:00Z.")
    };
    let when = if let Ok(d) = NaiveDate::parse_from_str(text, "%Y-%m-%d") {
        d.and_hms_opt(0, 0, 0).ok_or_else(bad)?.and_utc()
    } else {
        DateTime::parse_from_rfc3339(text)
            .map_err(|_| bad())?
            .with_timezone(&Utc)
    };
    if !allow_future && when > Utc::now() {
        return Err(Error::invalid(format!("\"{name}\" is in the future."))
            .with_next("Use a time in the past."));
    }
    Ok(when)
}

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

/// The `offset` argument. A value below 0 is `invalid_arguments`.
pub fn offset_arg(args: &Value) -> Result<usize> {
    match args.get("offset").and_then(Value::as_i64) {
        None => Ok(0),
        Some(n) if n >= 0 => Ok(n as usize),
        Some(_) => Err(Error::invalid("\"offset\" must be 0 or more.")
            .with_next("Send an offset of 0 or more.")),
    }
}

/// The filters that `list` and `search` share: `disputed`, `author`, `updated_after`, and
/// `updated_before` (see `search-filters`).
pub struct Filter {
    disputed: bool,
    author: Option<String>,
    after: Option<String>,
    before: Option<String>,
}

impl Filter {
    pub fn from_args(args: &Value) -> Result<Filter> {
        let date = |key: &str| -> Result<Option<String>> {
            match crate::writes::get_str(args, key) {
                None => Ok(None),
                Some(s) if crate::format::fields::is_valid_date(s) => Ok(Some(s.to_string())),
                Some(_) => Err(Error::invalid(format!("\"{key}\" is not a date."))
                    .with_next("Use the form YYYY-MM-DD.")),
            }
        };
        let after = date("updated_after")?;
        let before = date("updated_before")?;
        if let (Some(a), Some(b)) = (&after, &before) {
            if a > b {
                return Err(
                    Error::invalid("\"updated_after\" is later than \"updated_before\".")
                        .with_next("Swap the two dates, or change one of them."),
                );
            }
        }
        Ok(Filter {
            disputed: crate::writes::get_bool(args, "disputed"),
            author: crate::writes::get_str(args, "author")
                .map(|a| a.trim().to_lowercase())
                .filter(|a| !a.is_empty()),
            after,
            before,
        })
    }

    pub fn matches(&self, d: &Doc) -> bool {
        if self.disputed && !d.disputed() {
            return false;
        }
        if let Some(a) = &self.author {
            let by = |key: &str| {
                d.text(key)
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(a.as_str())
            };
            if !by("created_by") && !by("updated_by") {
                return false;
            }
        }
        let updated = d.updated();
        if let Some(a) = &self.after {
            if updated < a.as_str() {
                return false;
            }
        }
        if let Some(b) = &self.before {
            if updated > b.as_str() {
                return false;
            }
        }
        true
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

    fn doc_of(updated: &str, by: &str, disputed: bool) -> Doc {
        let extra = if disputed {
            "disputed: true\ndispute_reason: Wrong.\n"
        } else {
            ""
        };
        let raw = format!(
            "---\ntitle: \"T\"\ndescription: D.\nstatus: active\ncreated: 2026-01-01\nupdated: {updated}\ncreated_by: \"{by}\"\nupdated_by: \"{by}\"\n{extra}---\n\nbody\n"
        );
        let parsed = crate::format::parse(&raw).unwrap();
        Doc {
            id: "a".into(),
            raw,
            front: Some(parsed.frontmatter),
            body: parsed.body,
        }
    }

    #[test]
    fn the_filter_matches_author_range_and_disputed_state() {
        let d = doc_of("2026-11-05", "Bob Builder <bob@example.org>", false);
        let f = |a: Value| Filter::from_args(&a).unwrap();
        assert!(f(json!({})).matches(&d));
        assert!(f(json!({"author": "BOB"})).matches(&d));
        assert!(f(json!({"author": "example.org"})).matches(&d));
        assert!(!f(json!({"author": "anna"})).matches(&d));
        assert!(
            f(json!({"updated_after": "2026-11-05", "updated_before": "2026-11-05"})).matches(&d)
        );
        assert!(!f(json!({"updated_after": "2026-11-06"})).matches(&d));
        assert!(!f(json!({"updated_before": "2026-11-04"})).matches(&d));
        assert!(!f(json!({"disputed": true})).matches(&d));
        let disputed = doc_of("2026-11-05", "Bob Builder <bob@example.org>", true);
        assert!(f(json!({"disputed": true})).matches(&disputed));
        assert!(Filter::from_args(&json!({"updated_after": "x"})).is_err());
        assert!(Filter::from_args(
            &json!({"updated_after": "2026-11-06", "updated_before": "2026-11-05"})
        )
        .is_err());
    }

    #[test]
    fn parse_when_takes_a_date_or_an_rfc_3339_time_and_refuses_the_future_and_text() {
        assert!(parse_when("since", "2020-01-01", false).is_ok());
        assert!(parse_when("since", "2020-01-01T09:00:00Z", false).is_ok());
        assert!(parse_when("since", "2020-01-01T09:00:00+02:00", false).is_ok());
        assert!(parse_when("since", "2999-01-01", false).is_err());
        assert!(parse_when("since", "2999-01-01", true).is_ok());
        assert!(parse_when("since", "yesterday", true).is_err());
        assert!(parse_when("since", "2020-13-40", true).is_err());
    }
}
