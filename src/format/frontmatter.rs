//! Frontmatter: split, parse, read, and edit.
//!
//! A document keeps its frontmatter as groups of lines, one group per top-level key.
//! This keeps comments, unknown fields, and the original style, so that a
//! document that waymark reads and renders again is byte for byte the same.
//! The values come from `yaml-rust2`, which gives the raw type of each value.

use yaml_rust2::{Yaml, YamlLoader};

/// A property value, as the standard allows it (section 7), plus what it forbids.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Text(String),
    /// A number. The text is kept as written.
    Number(String),
    Bool(bool),
    List(Vec<String>),
    /// A key with no value.
    Empty,
    /// An object, or a list that holds something other than strings. Not allowed.
    Nested,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    NoFrontmatter,
    Unclosed,
    BadYaml(String),
    NotAMap,
}

#[derive(Debug, Clone)]
struct Group {
    key: String,
    lines: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Frontmatter {
    groups: Vec<Group>,
    values: Vec<(String, Value)>,
}

#[derive(Debug, Clone)]
pub struct Document {
    pub frontmatter: Frontmatter,
    pub body: String,
    closing_has_newline: bool,
}

/// The canonical order of the fields (section 7). Unknown fields go after these.
pub const FIELD_ORDER: [&str; 17] = [
    "title",
    "description",
    "status",
    "created",
    "updated",
    "created_by",
    "updated_by",
    "disputed",
    "dispute_reason",
    "sources",
    "tags",
    "related",
    "supersedes",
    "anchors",
    "trust",
    "verified_by",
    "verified",
];

fn order_index(key: &str) -> usize {
    FIELD_ORDER
        .iter()
        .position(|k| *k == key)
        .unwrap_or(FIELD_ORDER.len())
}

fn is_key_line(line: &str) -> Option<String> {
    let first = line.chars().next()?;
    if !(first.is_ascii_alphabetic() || first == '_') {
        return None;
    }
    let colon = line.find(':')?;
    let key = &line[..colon];
    if !key
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return None;
    }
    let after = &line[colon + 1..];
    if after.is_empty() || after.starts_with(' ') || after.starts_with('\t') {
        Some(key.to_string())
    } else {
        None
    }
}

fn to_value(y: &Yaml) -> Value {
    match y {
        Yaml::String(s) => Value::Text(s.clone()),
        Yaml::Integer(i) => Value::Number(i.to_string()),
        Yaml::Real(r) => Value::Number(r.clone()),
        Yaml::Boolean(b) => Value::Bool(*b),
        Yaml::Null => Value::Empty,
        Yaml::Array(items) => {
            let mut out = Vec::new();
            for item in items {
                match item {
                    Yaml::String(s) => out.push(s.clone()),
                    Yaml::Integer(i) => out.push(i.to_string()),
                    Yaml::Real(r) => out.push(r.clone()),
                    Yaml::Boolean(b) => out.push(b.to_string()),
                    _ => return Value::Nested,
                }
            }
            Value::List(out)
        }
        _ => Value::Nested,
    }
}

/// Split a document into the frontmatter text and the body.
/// Returns (frontmatter text with its line ends, body, whether the closing line ended in a newline).
fn split(text: &str) -> Result<(&str, &str, bool), ParseError> {
    let rest = text
        .strip_prefix("---\n")
        .ok_or(ParseError::NoFrontmatter)?;
    let mut pos = 0;
    for line in rest.split_inclusive('\n') {
        let bare = line.strip_suffix('\n').unwrap_or(line);
        if bare == "---" {
            let has_newline = line.ends_with('\n');
            return Ok((&rest[..pos], &rest[pos + line.len()..], has_newline));
        }
        pos += line.len();
    }
    Err(ParseError::Unclosed)
}

/// Parse a document. Structural problems are errors. Rule checks are in `fields`.
pub fn parse(text: &str) -> Result<Document, ParseError> {
    let (fm_text, body, closing_has_newline) = split(text)?;
    let values = if fm_text.trim().is_empty() {
        Vec::new()
    } else {
        let docs =
            YamlLoader::load_from_str(fm_text).map_err(|e| ParseError::BadYaml(e.to_string()))?;
        match docs.into_iter().next() {
            Some(Yaml::Hash(h)) => {
                let mut v = Vec::new();
                for (k, val) in h.iter() {
                    match k {
                        Yaml::String(key) => v.push((key.clone(), to_value(val))),
                        _ => return Err(ParseError::NotAMap),
                    }
                }
                v
            }
            Some(Yaml::Null) | None => Vec::new(),
            _ => return Err(ParseError::NotAMap),
        }
    };
    let mut groups: Vec<Group> = Vec::new();
    for line in fm_text.lines() {
        if let Some(key) = is_key_line(line) {
            groups.push(Group {
                key,
                lines: vec![line.to_string()],
            });
        } else if let Some(last) = groups.last_mut() {
            last.lines.push(line.to_string());
        } else {
            groups.push(Group {
                key: String::new(),
                lines: vec![line.to_string()],
            });
        }
    }
    Ok(Document {
        frontmatter: Frontmatter { groups, values },
        body: body.to_string(),
        closing_has_newline,
    })
}

impl Document {
    /// Write the document as text. An unchanged document gives the original text.
    pub fn render(&self) -> String {
        let mut out = String::from("---\n");
        for g in &self.frontmatter.groups {
            for l in &g.lines {
                out.push_str(l);
                out.push('\n');
            }
        }
        out.push_str("---");
        if self.closing_has_newline {
            out.push('\n');
        }
        out.push_str(&self.body);
        out
    }

    /// A new document with an empty frontmatter and a body.
    pub fn new(body: &str) -> Document {
        Document {
            frontmatter: Frontmatter {
                groups: Vec::new(),
                values: Vec::new(),
            },
            body: body.to_string(),
            closing_has_newline: true,
        }
    }
}

/// A string as a YAML scalar. Plain when that is safe, double-quoted otherwise.
pub fn scalar(s: &str) -> String {
    let plain_ok = !s.is_empty()
        && s.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || " ,.()/+_-".contains(c))
        && !s.ends_with(' ')
        && !matches!(
            s.to_ascii_lowercase().as_str(),
            "true" | "false" | "null" | "yes" | "no" | "on" | "off" | "y" | "n"
        );
    if plain_ok {
        s.to_string()
    } else {
        let mut out = String::from("\"");
        for c in s.chars() {
            match c {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\t' => out.push_str("\\t"),
                c => out.push(c),
            }
        }
        out.push('"');
        out
    }
}

impl Frontmatter {
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.values.iter().map(|(k, _)| k.as_str())
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.values.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    pub fn get_text(&self, key: &str) -> Option<&str> {
        match self.get(key) {
            Some(Value::Text(s)) => Some(s),
            _ => None,
        }
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.get(key) {
            Some(Value::Bool(b)) => Some(*b),
            _ => None,
        }
    }

    pub fn get_list(&self, key: &str) -> Option<&[String]> {
        match self.get(key) {
            Some(Value::List(l)) => Some(l),
            _ => None,
        }
    }

    /// Remove a field. Returns true if it existed.
    pub fn remove(&mut self, key: &str) -> bool {
        let before = self.groups.len();
        self.groups.retain(|g| g.key != key);
        self.values.retain(|(k, _)| k != key);
        before != self.groups.len()
    }

    fn put(&mut self, key: &str, lines: Vec<String>, value: Value) {
        if let Some(g) = self.groups.iter_mut().find(|g| g.key == key) {
            g.lines = lines;
            if let Some(v) = self.values.iter_mut().find(|(k, _)| k == key) {
                v.1 = value;
            }
            return;
        }
        // Insert after the last group that comes before this key in the canonical order.
        let idx = order_index(key);
        let mut at = self.groups.len();
        if idx < FIELD_ORDER.len() {
            at = 0;
            for (i, g) in self.groups.iter().enumerate() {
                if !g.key.is_empty() && order_index(&g.key) < idx {
                    at = i + 1;
                }
            }
            // A field with no known field before it goes first, but after a comment preamble.
            if at == 0 && self.groups.first().is_some_and(|g| g.key.is_empty()) {
                at = 1;
            }
            // Never put a canonical field after an unknown field that follows only unknown fields.
            if at < self.groups.len()
                && self.groups[at..]
                    .iter()
                    .all(|g| order_index(&g.key) == FIELD_ORDER.len())
            {
                at = self.groups.len().min(at);
            }
        }
        self.groups.insert(
            at,
            Group {
                key: key.to_string(),
                lines,
            },
        );
        let vpos = self
            .values
            .iter()
            .position(|(k, _)| order_index(k) > idx && idx < FIELD_ORDER.len())
            .unwrap_or(self.values.len());
        self.values.insert(vpos, (key.to_string(), value));
    }

    pub fn set_text(&mut self, key: &str, value: &str) {
        self.put(
            key,
            vec![format!("{key}: {}", scalar(value))],
            Value::Text(value.to_string()),
        );
    }

    /// A date as `YYYY-MM-DD`, written without quotes. The caller checks the form.
    pub fn set_date(&mut self, key: &str, value: &str) {
        self.put(
            key,
            vec![format!("{key}: {value}")],
            Value::Text(value.to_string()),
        );
    }

    pub fn set_bool(&mut self, key: &str, value: bool) {
        self.put(key, vec![format!("{key}: {value}")], Value::Bool(value));
    }

    pub fn set_list(&mut self, key: &str, items: &[String]) {
        let lines = if items.is_empty() {
            vec![format!("{key}: []")]
        } else {
            let mut l = vec![format!("{key}:")];
            for i in items {
                l.push(format!("  - {}", scalar(i)));
            }
            l
        };
        self.put(key, lines, Value::List(items.to_vec()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOC: &str = "---\n# a comment\ntitle: A title\ndescription: \"Quoted: text\"\nstatus: active\ntags:\n  - one\n  - two\nowner_x: keep me\n---\n\n# Heading\n\nBody text.\n";

    #[test]
    fn an_unchanged_document_renders_the_same() {
        let d = parse(DOC).unwrap();
        assert_eq!(d.render(), DOC);
    }

    #[test]
    fn a_document_without_final_newline_renders_the_same() {
        let t = "---\ntitle: x\n---";
        assert_eq!(parse(t).unwrap().render(), t);
    }

    #[test]
    fn values_have_their_raw_types() {
        let d = parse(DOC).unwrap();
        assert_eq!(d.frontmatter.get_text("title"), Some("A title"));
        assert_eq!(d.frontmatter.get_text("description"), Some("Quoted: text"));
        assert_eq!(
            d.frontmatter.get_list("tags"),
            Some(&["one".to_string(), "two".to_string()][..])
        );
        assert_eq!(d.body, "\n# Heading\n\nBody text.\n");
    }

    #[test]
    fn yaml_traps_keep_their_raw_type() {
        let d = parse("---\ntitle: 123\nstatus: yes\ndisputed: true\ncreated: 2026-11-08\n---\n")
            .unwrap();
        assert_eq!(
            d.frontmatter.get("title"),
            Some(&Value::Number("123".into()))
        );
        // YAML 1.2: `yes` is text, not a boolean.
        assert_eq!(
            d.frontmatter.get("status"),
            Some(&Value::Text("yes".into()))
        );
        assert_eq!(d.frontmatter.get_bool("disputed"), Some(true));
        // A date is text in YAML 1.2.
        assert_eq!(d.frontmatter.get_text("created"), Some("2026-11-08"));
    }

    #[test]
    fn nested_values_are_marked() {
        let d = parse("---\nmeta:\n  owner: x\nlist:\n  - a: b\n---\n").unwrap();
        assert_eq!(d.frontmatter.get("meta"), Some(&Value::Nested));
        assert_eq!(d.frontmatter.get("list"), Some(&Value::Nested));
    }

    #[test]
    fn structural_errors() {
        assert_eq!(
            parse("no frontmatter").unwrap_err(),
            ParseError::NoFrontmatter
        );
        assert_eq!(parse("---\ntitle: x\n").unwrap_err(), ParseError::Unclosed);
        assert!(matches!(
            parse("---\ntitle: [x\n---\n").unwrap_err(),
            ParseError::BadYaml(_)
        ));
        assert_eq!(
            parse("---\n- a\n- b\n---\n").unwrap_err(),
            ParseError::NotAMap
        );
    }

    #[test]
    fn set_replaces_in_place_and_keeps_other_lines() {
        let mut d = parse(DOC).unwrap();
        d.frontmatter.set_text("status", "deprecated");
        let out = d.render();
        assert!(out.contains("status: deprecated\n"));
        assert!(out.contains("# a comment\n"));
        assert!(out.contains("owner_x: keep me\n"));
    }

    #[test]
    fn set_inserts_in_canonical_order() {
        let mut d = parse("---\ntitle: A\nstatus: active\nowner_x: keep\n---\n").unwrap();
        d.frontmatter.set_text("description", "Desc");
        d.frontmatter.set_date("created", "2026-11-14");
        let keys: Vec<&str> = d.frontmatter.keys().collect();
        assert_eq!(
            keys,
            vec!["title", "description", "status", "created", "owner_x"]
        );
        let again = parse(&d.render()).unwrap();
        let keys2: Vec<&str> = again.frontmatter.keys().collect();
        assert_eq!(keys2, keys);
    }

    #[test]
    fn remove_deletes_the_whole_group() {
        let mut d = parse(DOC).unwrap();
        assert!(d.frontmatter.remove("tags"));
        assert!(!d.render().contains("  - one"));
    }

    #[test]
    fn written_values_parse_back() {
        let mut d = Document::new("Body\n");
        d.frontmatter
            .set_text("title", "Payment API: v3 \"uses\" keys");
        d.frontmatter.set_text("status", "active");
        d.frontmatter
            .set_list("related", &["[[backend/a]]".to_string()]);
        d.frontmatter.set_bool("disputed", false);
        let back = parse(&d.render()).unwrap();
        assert_eq!(
            back.frontmatter.get_text("title"),
            Some("Payment API: v3 \"uses\" keys")
        );
        assert_eq!(
            back.frontmatter.get_list("related"),
            Some(&["[[backend/a]]".to_string()][..])
        );
        assert_eq!(back.frontmatter.get_bool("disputed"), Some(false));
    }

    #[test]
    fn scalar_quotes_when_needed() {
        assert_eq!(scalar("Plain text"), "Plain text");
        assert_eq!(scalar("true"), "\"true\"");
        assert_eq!(scalar("a: b"), "\"a: b\"");
        assert_eq!(scalar("[[x]]"), "\"[[x]]\"");
        assert_eq!(scalar("3 ways"), "\"3 ways\"");
        assert_eq!(
            scalar("Jane Doe <jane@example.org>"),
            "\"Jane Doe <jane@example.org>\""
        );
    }
}
