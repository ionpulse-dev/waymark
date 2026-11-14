//! Document format: the Waymark Memory Standard (see `standard/waymark-1.0.md`).
//!
//! This module has no I/O. A caller gives it text and gets findings back.

pub mod count;
pub mod fields;
pub mod finding;
pub mod frontmatter;
pub mod id;
pub mod limits;
pub mod links;

pub use finding::{has_errors, Finding, Severity};
pub use frontmatter::{parse, Document, ParseError, Value};
pub use limits::Limits;
pub use links::DocSet;

use regex::Regex;
use std::sync::OnceLock;

fn inline_tag_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?:^|[\s(])#([A-Za-z][A-Za-z0-9_/-]*)").unwrap())
}

/// Check one document given as bytes. `path` is the path below the root, for example
/// `backend/payments/idempotency-keys.md`. With a `DocSet`, links are checked too.
/// All findings are returned, not only the first one.
pub fn check_bytes(
    path: &str,
    bytes: &[u8],
    limits: &Limits,
    docs: Option<&dyn DocSet>,
) -> Vec<Finding> {
    match std::str::from_utf8(bytes) {
        Ok(text) => check(path, text, limits, docs),
        Err(_) => {
            let mut out = id::check_path(path);
            out.push(Finding::error(
                path,
                "not-utf8",
                "The file is not valid UTF-8 text.",
            ));
            out
        }
    }
}

/// Check one document given as text. See `check_bytes`.
pub fn check(path: &str, text: &str, limits: &Limits, docs: Option<&dyn DocSet>) -> Vec<Finding> {
    let mut out = id::check_path(path);

    let size = count::bytes(text);
    if size > limits.max_bytes {
        out.push(Finding::error(
            path,
            "document-too-large",
            format!(
                "The document has {size} bytes. The cap is {} bytes. Split the document, or move details into a linked source.",
                limits.max_bytes
            ),
        ));
    }
    if text.contains('\r') {
        out.push(Finding::error(
            path,
            "bad-line-ends",
            "The document must use LF line ends.",
        ));
    }

    match parse(text) {
        Err(ParseError::NoFrontmatter) => {
            out.push(Finding::error(
                path,
                "no-frontmatter",
                "The document must start with frontmatter between two --- lines.",
            ));
        }
        Err(ParseError::Unclosed) => {
            out.push(Finding::error(
                path,
                "unclosed-frontmatter",
                "The frontmatter has no closing --- line.",
            ));
        }
        Err(ParseError::BadYaml(e)) => {
            out.push(Finding::error(
                path,
                "bad-yaml",
                format!("The frontmatter is not valid YAML: {e}"),
            ));
        }
        Err(ParseError::NotAMap) => {
            out.push(Finding::error(
                path,
                "bad-yaml",
                "The frontmatter must be a list of properties.",
            ));
        }
        Ok(doc) => {
            out.extend(fields::check_fields(path, &doc.frontmatter, limits));
            if let Some(docs) = docs {
                out.extend(links::check_links(path, &doc.frontmatter, &doc.body, docs));
            }
            let body = links::strip_code(&doc.body);
            if let Some(m) = inline_tag_re().captures(&body) {
                out.push(Finding::warning(
                    path,
                    "inline-tag",
                    format!(
                        "The body has the inline tag #{}. Use the tags property.",
                        &m[1]
                    ),
                ));
            }
        }
    }
    out
}
