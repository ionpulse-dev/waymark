//! The shared settings file `.mkbase/waymark.toml` (see `knowledge-base-repo`).

use crate::config::{layers, Limits};
use crate::error::{Error, Result};
use crate::format;

/// The highest schema of `waymark.toml` that this build supports (see `knowledge-base-repo`).
pub const SCHEMA_VERSION: i64 = 1;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct People {
    pub email_domains: Vec<String>,
    pub email_blocklist: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Settings {
    /// The schema of the file. A file without the key has schema 1.
    pub schema: i64,
    pub name: String,
    /// The UUID that identifies the knowledge base.
    pub id: String,
    pub slug: String,
    pub verification: bool,
    /// The shared layer of limits: the sections `[limits]` and `[approval]`.
    pub limits: Limits,
    pub people: People,
    pub documents: format::Limits,
    pub extra_secret_patterns: Vec<String>,
    /// Keys that this version does not know. They are kept in the file and reported.
    pub notices: Vec<String>,
}

const TOP_KEYS: [&str; 10] = [
    "schema",
    "name",
    "id",
    "slug",
    "verification",
    "limits",
    "approval",
    "people",
    "documents",
    "lint",
];
const LIMIT_KEYS: [&str; 7] = [
    "access",
    "paused",
    "tools_denied",
    "folders",
    "max_results",
    "max_tokens",
    "max_documents_opened",
];

fn str_list(t: &toml::Table, key: &str) -> Vec<String> {
    t.get(key)
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

fn section<'a>(t: &'a toml::Table, name: &str) -> Option<&'a toml::Table> {
    t.get(name).and_then(|v| v.as_table())
}

fn is_uuid(s: &str) -> bool {
    uuid::Uuid::parse_str(s).is_ok()
}

/// Read the text of `.mkbase/waymark.toml`. A missing or wrong `id` is a `setup_problem`.
pub fn parse(text: &str, path_for_messages: &str) -> Result<Settings> {
    let table: toml::Table = text.parse().map_err(|e| {
        Error::setup(format!("{path_for_messages} is not valid TOML: {e}"))
            .with_next("Fix the file, or restore it from Git.")
    })?;
    // The schema is checked first: a file of a newer schema may have keys that mean something else.
    let schema = match table.get("schema") {
        None => 1,
        Some(v) => match v.as_integer() {
            Some(n) if n >= 1 => n,
            _ => {
                return Err(Error::setup(format!(
                    "The key schema in {path_for_messages} must be a whole number of 1 or more."
                ))
                .with_next("Fix the file, or restore it from Git."))
            }
        },
    };
    if schema > SCHEMA_VERSION {
        return Err(Error::needs_update(
            format!(
                "This knowledge base needs a newer waymark. Its settings file has schema {schema}, and this waymark supports schema {SCHEMA_VERSION}."
            ),
            "Update waymark with `brew upgrade waymark`, then restart your agent tool.",
        ));
    }
    let limits = layers::parse_layer(text).map_err(|e| {
        Error::setup(format!(
            "{path_for_messages} has a wrong value in [limits] or [approval]: {e}"
        ))
        .with_next("Fix the file, or restore it from Git.")
    })?;

    let id = table
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if id.is_empty() {
        return Err(Error::setup(format!("{path_for_messages} has no id."))
            .with_next("Add an id (a UUID) to the file by hand, then commit it."));
    }
    if !is_uuid(&id) {
        return Err(
            Error::setup(format!("The id in {path_for_messages} is not a UUID."))
                .with_next("Use a UUID as the id."),
        );
    }

    let mut notices = Vec::new();
    for k in table.keys() {
        if !TOP_KEYS.contains(&k.as_str()) {
            notices.push(format!(
                "{path_for_messages} has the unknown key \"{k}\". It is kept and ignored."
            ));
        }
    }
    if let Some(l) = section(&table, "limits") {
        for k in l.keys() {
            if !LIMIT_KEYS.contains(&k.as_str()) {
                notices.push(format!("{path_for_messages} has the unknown key \"limits.{k}\". It is kept and ignored."));
            }
        }
    }

    let mut documents = format::Limits::default();
    if let Some(d) = section(&table, "documents") {
        if let Some(n) = d
            .get("max_bytes")
            .and_then(|v| v.as_integer())
            .filter(|n| *n > 0)
        {
            documents.max_bytes = n as usize;
        }
        if let Some(n) = d
            .get("title_max_words")
            .and_then(|v| v.as_integer())
            .filter(|n| *n > 0)
        {
            documents.title_max_words = n as usize;
        }
        if let Some(n) = d
            .get("description_max_words")
            .and_then(|v| v.as_integer())
            .filter(|n| *n > 0)
        {
            documents.description_max_words = n as usize;
        }
    }
    let people = section(&table, "people")
        .map(|p| People {
            email_domains: str_list(p, "email_domains"),
            email_blocklist: str_list(p, "email_blocklist"),
        })
        .unwrap_or_default();
    let extra_secret_patterns = section(&table, "lint")
        .map(|l| str_list(l, "extra_secret_patterns"))
        .unwrap_or_default();

    Ok(Settings {
        schema,
        name: table
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        id,
        slug: table
            .get("slug")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        verification: table
            .get("verification")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        limits,
        people,
        documents,
        extra_secret_patterns,
        notices,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Code;

    const ID: &str = "123e4567-e89b-42d3-a456-426614174000";

    #[test]
    fn parses_a_full_file() {
        let t = format!(
            "name = \"Example\"\nid = \"{ID}\"\nslug = \"example\"\nverification = true\n[limits]\naccess = \"read-only\"\nmax_results = 3\n[approval]\nvia = \"console\"\n[people]\nemail_domains = [\"example.org\"]\n[documents]\nmax_bytes = 5000\ntitle_max_words = 10\n[lint]\nextra_secret_patterns = [\"IBAN\"]\n"
        );
        let s = parse(&t, "waymark.toml").unwrap();
        assert_eq!(s.name, "Example");
        assert!(s.verification);
        assert_eq!(s.limits.max_results, Some(3));
        assert_eq!(s.documents.max_bytes, 5000);
        assert_eq!(s.documents.title_max_words, 10);
        assert_eq!(s.documents.description_max_words, 50);
        assert_eq!(s.people.email_domains, vec!["example.org"]);
        assert_eq!(s.extra_secret_patterns, vec!["IBAN"]);
        assert!(s.notices.is_empty());
    }

    #[test]
    fn the_schema_defaults_to_1_and_a_higher_one_is_refused() {
        let t = format!("id = \"{ID}\"\n");
        assert_eq!(parse(&t, "waymark.toml").unwrap().schema, 1);
        let t = format!("schema = 1\nid = \"{ID}\"\n");
        assert_eq!(parse(&t, "waymark.toml").unwrap().schema, 1);
        let t = format!("schema = 2\nid = \"{ID}\"\n");
        let e = parse(&t, "waymark.toml").unwrap_err();
        assert_eq!(e.code, Code::SetupProblem);
        assert!(e.message.contains("needs a newer waymark"), "{}", e.message);
        assert!(e.message.contains("schema 2") && e.message.contains("schema 1"));
        assert!(e.next.unwrap().contains("brew upgrade waymark"));
    }

    #[test]
    fn a_bad_schema_is_a_setup_problem() {
        for bad in ["0", "-1", "\"x\"", "1.5"] {
            let t = format!("schema = {bad}\nid = \"{ID}\"\n");
            let e = parse(&t, "waymark.toml").unwrap_err();
            assert_eq!(e.code, Code::SetupProblem, "{bad}");
        }
    }

    #[test]
    fn a_missing_or_bad_id_is_a_setup_problem() {
        let e = parse("name = \"x\"\n", "waymark.toml").unwrap_err();
        assert_eq!(e.code, Code::SetupProblem);
        assert!(e.next.is_some());
        let e = parse("id = \"not-a-uuid\"\n", "waymark.toml").unwrap_err();
        assert_eq!(e.code, Code::SetupProblem);
    }

    #[test]
    fn a_syntax_error_is_a_setup_problem() {
        assert_eq!(
            parse("id = = =", "waymark.toml").unwrap_err().code,
            Code::SetupProblem
        );
    }

    #[test]
    fn unknown_keys_give_notices_and_no_error() {
        let t = format!("id = \"{ID}\"\nfuture = 1\n[limits]\nnew_limit = 2\n");
        let s = parse(&t, "waymark.toml").unwrap();
        assert_eq!(s.notices.len(), 2);
    }

    #[test]
    fn defaults_apply() {
        let s = parse(&format!("id = \"{ID}\"\n"), "waymark.toml").unwrap();
        assert!(!s.verification);
        assert_eq!(s.documents, format::Limits::default());
        assert_eq!(s.limits, Limits::default());
    }
}
