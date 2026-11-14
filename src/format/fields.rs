//! Field rules (section 7 of the standard).

use super::count;
use super::finding::Finding;
use super::frontmatter::{Frontmatter, Value};
use super::limits::Limits;
use regex::Regex;
use std::sync::OnceLock;

const REQUIRED: [&str; 7] = [
    "title",
    "description",
    "status",
    "created",
    "updated",
    "created_by",
    "updated_by",
];
const LIST_FIELDS: [&str; 5] = ["sources", "tags", "related", "supersedes", "anchors"];

fn missing_rule(field: &str) -> &'static str {
    match field {
        "title" => "missing-title",
        "description" => "missing-description",
        "status" => "missing-status",
        "created" => "missing-created",
        "updated" => "missing-updated",
        "created_by" => "missing-created-by",
        _ => "missing-updated-by",
    }
}

fn author_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[^<>]*[^<>\s] <[^<>\s@]+@[^<>\s@]+>$").unwrap())
}

fn wikilink_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^\[\[([^\[\]|#]+)\]\]$").unwrap())
}

/// The target of a full-path wikilink in a property, such as `[[backend/a]]`.
pub fn property_link_target(value: &str) -> Option<&str> {
    wikilink_re()
        .captures(value)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str())
}

/// A date in the form `YYYY-MM-DD` that exists in the calendar.
pub fn is_valid_date(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return false;
    }
    let digits = |r: std::ops::Range<usize>| s[r].chars().all(|c| c.is_ascii_digit());
    if !(digits(0..4) && digits(5..7) && digits(8..10)) {
        return false;
    }
    let y: u32 = s[0..4].parse().unwrap();
    let m: u32 = s[5..7].parse().unwrap();
    let d: u32 = s[8..10].parse().unwrap();
    let leap = (y.is_multiple_of(4) && !y.is_multiple_of(100)) || y.is_multiple_of(400);
    let dim = match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if leap {
                29
            } else {
                28
            }
        }
        _ => return false,
    };
    y >= 1 && d >= 1 && d <= dim
}

fn type_name(v: &Value) -> &'static str {
    match v {
        Value::Text(_) => "text",
        Value::Number(_) => "a number",
        Value::Bool(_) => "a checkbox",
        Value::List(_) => "a list",
        Value::Empty => "empty",
        Value::Nested => "an object",
    }
}

pub fn check_fields(path: &str, fm: &Frontmatter, limits: &Limits) -> Vec<Finding> {
    let mut out = Vec::new();

    for key in fm.keys() {
        if matches!(fm.get(key), Some(Value::Nested)) {
            out.push(Finding::error(
                path,
                "nested-property",
                format!("The property \"{key}\" holds an object or a list of objects. Use flat values: text, a number, true or false, a date, or a list of text."),
            ));
        }
    }

    for field in REQUIRED {
        match fm.get(field) {
            None | Some(Value::Empty) => out.push(Finding::error(
                path,
                missing_rule(field),
                format!("Add the required field \"{field}\"."),
            )),
            Some(Value::Text(s)) if s.trim().is_empty() => out.push(Finding::error(
                path,
                missing_rule(field),
                format!("The required field \"{field}\" is empty."),
            )),
            Some(Value::Text(_)) => {}
            Some(Value::Nested) => {}
            Some(other) => out.push(Finding::error(
                path,
                "bad-type",
                format!(
                    "The field \"{field}\" must be text, but it is {}.",
                    type_name(other)
                ),
            )),
        }
    }

    if let Some(title) = fm.get_text("title") {
        let n = count::words(title);
        if n > limits.title_max_words {
            out.push(Finding::error(
                path,
                "title-too-long",
                format!(
                    "The title has {n} words. Use at most {}.",
                    limits.title_max_words
                ),
            ));
        }
    }
    if let Some(desc) = fm.get_text("description") {
        let n = count::words(desc);
        if n > limits.description_max_words {
            out.push(Finding::error(
                path,
                "description-too-long",
                format!(
                    "The description has {n} words. Use at most {}.",
                    limits.description_max_words
                ),
            ));
        }
    }

    if let Some(status) = fm.get_text("status") {
        if status != "active" && status != "deprecated" {
            out.push(Finding::error(
                path,
                "bad-status",
                format!("The status \"{status}\" is not allowed. Use active or deprecated."),
            ));
        }
    }

    let mut dates = Vec::new();
    for field in ["created", "updated"] {
        if let Some(d) = fm.get_text(field) {
            if is_valid_date(d) {
                dates.push((field, d));
            } else {
                out.push(Finding::error(
                    path,
                    "bad-date",
                    format!(
                        "The field \"{field}\" must be a date in the form YYYY-MM-DD, not \"{d}\"."
                    ),
                ));
            }
        }
    }
    if dates.len() == 2 && dates[1].1 < dates[0].1 {
        out.push(Finding::error(
            path,
            "updated-before-created",
            "The date \"updated\" is earlier than \"created\".",
        ));
    }

    for field in ["created_by", "updated_by"] {
        if let Some(a) = fm.get_text(field) {
            if !a.trim().is_empty() && !author_re().is_match(a) {
                out.push(Finding::error(
                    path,
                    "bad-author",
                    format!("The field \"{field}\" must have the form \"Name <email>\"."),
                ));
            }
        }
    }

    // disputed and dispute_reason
    let disputed = match fm.get("disputed") {
        None | Some(Value::Empty) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::Nested) => false,
        Some(other) => {
            out.push(Finding::error(
                path,
                "bad-type",
                format!(
                    "The field \"disputed\" must be true or false, but it is {}.",
                    type_name(other)
                ),
            ));
            false
        }
    };
    let reason_present = fm
        .get_text("dispute_reason")
        .is_some_and(|r| !r.trim().is_empty());
    if disputed && !reason_present {
        out.push(Finding::error(
            path,
            "disputed-without-reason",
            "A disputed document needs a dispute_reason.",
        ));
    }
    if !disputed && reason_present {
        out.push(Finding::warning(
            path,
            "reason-without-dispute",
            "The field dispute_reason is set, but disputed is not true.",
        ));
    }

    for field in LIST_FIELDS {
        match fm.get(field) {
            None | Some(Value::Empty) | Some(Value::Nested) => {}
            Some(Value::List(items)) => check_list(path, field, items, &mut out),
            Some(other) => out.push(Finding::error(
                path,
                "bad-type",
                format!(
                    "The field \"{field}\" must be a list of text, but it is {}.",
                    type_name(other)
                ),
            )),
        }
    }
    out
}

fn check_list(path: &str, field: &str, items: &[String], out: &mut Vec<Finding>) {
    for item in items {
        match field {
            "tags" => {
                if item.starts_with('#') || item.chars().any(char::is_whitespace) || item.is_empty()
                {
                    out.push(Finding::error(
                        path,
                        "bad-tag",
                        format!("The tag \"{item}\" must have no # and no spaces."),
                    ));
                }
            }
            "related" | "supersedes" => {
                if property_link_target(item).is_none() {
                    out.push(Finding::error(
                        path,
                        "bad-property-link",
                        format!("The value \"{item}\" in \"{field}\" must be a full-path wikilink, for example \"[[backend/payments/retry-policy]]\"."),
                    ));
                }
            }
            "anchors" if !(item.starts_with("git:") || item.starts_with("url:")) => {
                out.push(Finding::error(
                    path,
                    "bad-anchor",
                    format!("The anchor \"{item}\" must start with git: or url:."),
                ));
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::frontmatter::parse;

    fn rules(text: &str) -> Vec<&'static str> {
        let d = parse(text).unwrap();
        check_fields("x.md", &d.frontmatter, &Limits::default())
            .iter()
            .map(|f| f.rule)
            .collect()
    }

    const OK: &str = "---\ntitle: A title\ndescription: A description\nstatus: active\ncreated: 2026-11-08\nupdated: 2026-11-08\ncreated_by: \"Jane Doe <jane@example.org>\"\nupdated_by: \"Jane Doe <jane@example.org>\"\n---\nBody\n";

    #[test]
    fn a_valid_document_has_no_findings() {
        assert!(rules(OK).is_empty());
    }

    #[test]
    fn dates() {
        assert!(is_valid_date("2026-02-28"));
        assert!(!is_valid_date("2026-02-29"));
        assert!(is_valid_date("2028-02-29"));
        assert!(!is_valid_date("08.11.2026"));
        assert!(!is_valid_date("2026-13-01"));
        assert!(!is_valid_date("2026-1-01"));
    }

    #[test]
    fn missing_and_empty_required_fields() {
        let t = OK.replace("description: A description\n", "");
        assert_eq!(rules(&t), vec!["missing-description"]);
        let t = OK.replace("title: A title", "title: \"\"");
        assert_eq!(rules(&t), vec!["missing-title"]);
    }

    #[test]
    fn a_number_as_title_is_a_type_finding() {
        let t = OK.replace("title: A title", "title: 123");
        assert_eq!(rules(&t), vec!["bad-type"]);
    }

    #[test]
    fn status_values() {
        assert_eq!(
            rules(&OK.replace("status: active", "status: draft")),
            vec!["bad-status"]
        );
        assert!(rules(&OK.replace("status: active", "status: deprecated")).is_empty());
        // YAML 1.2: yes is text, so it is a bad status and not a type error.
        assert_eq!(
            rules(&OK.replace("status: active", "status: yes")),
            vec!["bad-status"]
        );
    }

    #[test]
    fn updated_before_created() {
        let t = OK.replace("updated: 2026-11-08", "updated: 2026-11-07");
        assert_eq!(rules(&t), vec!["updated-before-created"]);
    }

    #[test]
    fn authors_need_name_and_email() {
        assert_eq!(
            rules(&OK.replace(
                "created_by: \"Jane Doe <jane@example.org>\"",
                "created_by: Jane Doe"
            )),
            vec!["bad-author"]
        );
        assert_eq!(rules(&OK.replace("<jane@example.org>", "<jane>")).len(), 2);
    }

    #[test]
    fn disputed_needs_a_reason() {
        let t = OK.replace("---\nBody", "disputed: true\n---\nBody");
        assert_eq!(rules(&t), vec!["disputed-without-reason"]);
        let t = OK.replace(
            "---\nBody",
            "disputed: true\ndispute_reason: Wrong since v3\n---\nBody",
        );
        assert!(rules(&t).is_empty());
        let t = OK.replace("---\nBody", "dispute_reason: Wrong\n---\nBody");
        assert_eq!(rules(&t), vec!["reason-without-dispute"]);
    }

    #[test]
    fn disputed_yes_is_not_a_checkbox() {
        let t = OK.replace("---\nBody", "disputed: \"yes\"\n---\nBody");
        assert_eq!(rules(&t), vec!["bad-type"]);
    }

    #[test]
    fn list_fields() {
        let t = OK.replace(
            "---\nBody",
            "tags:\n  - ok\n  - \"#bad\"\n  - \"with space\"\n---\nBody",
        );
        assert_eq!(rules(&t), vec!["bad-tag", "bad-tag"]);
        let t = OK.replace("---\nBody", "related:\n  - \"[[a/b]]\"\n  - other\nanchors:\n  - nothing\n  - \"url:https://example.org\"\n---\nBody");
        assert_eq!(rules(&t), vec!["bad-property-link", "bad-anchor"]);
        let t = OK.replace("---\nBody", "tags: single\n---\nBody");
        assert_eq!(rules(&t), vec!["bad-type"]);
    }

    #[test]
    fn nested_properties_fail() {
        let t = OK.replace("---\nBody", "meta:\n  owner: x\n---\nBody");
        assert_eq!(rules(&t), vec!["nested-property"]);
    }

    #[test]
    fn word_limits_at_the_boundary() {
        let t15 = vec!["w"; 15].join(" ");
        let t16 = vec!["w"; 16].join(" ");
        assert!(rules(&OK.replace("A title", &t15)).is_empty());
        assert_eq!(rules(&OK.replace("A title", &t16)), vec!["title-too-long"]);
        let d50 = vec!["w"; 50].join(" ");
        let d51 = vec!["w"; 51].join(" ");
        assert!(rules(&OK.replace("A description", &d50)).is_empty());
        assert_eq!(
            rules(&OK.replace("A description", &d51)),
            vec!["description-too-long"]
        );
    }

    #[test]
    fn unknown_fields_are_allowed() {
        let t = OK.replace("---\nBody", "owner_x: something\n---\nBody");
        assert!(rules(&t).is_empty());
    }
}
