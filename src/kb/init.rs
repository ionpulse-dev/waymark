//! The files of a new knowledge base (see `specs/features/create-knowledge-base`).

use super::agents_file::{self, Audience};
use super::layout::{MARKER, SETTINGS_FILE};
use super::settings;
use crate::error::{Error, Result};

const GITIGNORE_TEMPLATE: &str = include_str!("templates/gitignore");
const OBSIDIAN_TEMPLATE: &str = include_str!("templates/obsidian-app.json");
const SETTINGS_TEMPLATE: &str = include_str!("templates/waymark.toml");
const EXCLUDE_TEMPLATE: &str =
    "# Files and folders below the root that are not part of the knowledge base.\n";

/// What a person can set when a knowledge base is created.
#[derive(Debug, Clone)]
pub struct InitInput {
    pub name: String,
    pub slug: String,
    pub email_domains: Vec<String>,
    pub audience: Audience,
    /// The keys of `[limits]`, as TOML values.
    pub limits: toml::Table,
    /// The keys of `[approval]`.
    pub approval: toml::Table,
    /// Folders below `knowledge/`.
    pub folders: Vec<String>,
}

impl InitInput {
    pub fn plain(name: &str, slug: &str, audience: Audience, email_domains: &[String]) -> Self {
        InitInput {
            name: name.to_string(),
            slug: slug.to_string(),
            email_domains: email_domains.to_vec(),
            audience,
            limits: toml::Table::new(),
            approval: toml::Table::new(),
            folders: Vec::new(),
        }
    }
}

/// A short URL-safe name from a display name: at most 40 characters. `knowledge-base` if nothing
/// is left.
pub fn derive_slug(name: &str) -> String {
    let mut out = String::new();
    for c in name.to_lowercase().chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            out.push(c);
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    let out: String = out.trim_matches('-').chars().take(40).collect();
    let out = out.trim_matches('-').to_string();
    if out.is_empty() {
        "knowledge-base".to_string()
    } else {
        out
    }
}

/// The text of `.mkbase/waymark.toml`. With no extra input it is the commented template.
/// Otherwise it is written with a TOML library, so that every value is escaped.
pub fn settings_text(input: &InitInput, id: &str) -> Result<String> {
    let plain =
        input.limits.is_empty() && input.approval.is_empty() && input.email_domains.is_empty();
    if plain {
        let name_toml = toml::Value::String(input.name.clone()).to_string();
        return Ok(SETTINGS_TEMPLATE
            .replace("{name}", &name_toml)
            .replace("{id}", id)
            .replace("{slug}", &input.slug));
    }
    let mut t = toml::Table::new();
    t.insert("name".into(), input.name.clone().into());
    t.insert("id".into(), id.to_string().into());
    t.insert("slug".into(), input.slug.clone().into());
    t.insert("verification".into(), false.into());
    if !input.email_domains.is_empty() {
        let mut p = toml::Table::new();
        p.insert(
            "email_domains".into(),
            toml::Value::Array(
                input
                    .email_domains
                    .iter()
                    .map(|d| d.clone().into())
                    .collect(),
            ),
        );
        t.insert("people".into(), p.into());
    }
    if !input.limits.is_empty() {
        t.insert("limits".into(), input.limits.clone().into());
    }
    if !input.approval.is_empty() {
        t.insert("approval".into(), input.approval.clone().into());
    }
    let text = toml::to_string(&t)
        .map_err(|e| Error::invalid(format!("The settings cannot be written: {e}")))?;
    // The new settings must parse, with the same rules as a read.
    settings::parse(&text, SETTINGS_FILE).map_err(|e| {
        Error::invalid(format!(
            "The values for limits or approval are wrong: {}",
            e.message
        ))
        .with_next("Check the fields `limits` and `approval` against the settings keys.")
    })?;
    Ok(text)
}

/// The files that `init` writes, as (path below the repository folder, content).
pub fn files(input: &InitInput, id: &str) -> Result<Vec<(String, String)>> {
    let mut out = vec![
        (
            "AGENTS.md".to_string(),
            agents_file::render(input.audience, &input.email_domains),
        ),
        (".gitignore".to_string(), GITIGNORE_TEMPLATE.to_string()),
        (
            ".obsidian/app.json".to_string(),
            OBSIDIAN_TEMPLATE.to_string(),
        ),
        (format!("knowledge/{MARKER}/version"), "1.0\n".to_string()),
        (
            format!("knowledge/{MARKER}/{SETTINGS_FILE}"),
            settings_text(input, id)?,
        ),
        (
            format!("knowledge/{MARKER}/exclude"),
            EXCLUDE_TEMPLATE.to_string(),
        ),
    ];
    for f in &input.folders {
        out.push((format!("knowledge/{f}/.gitkeep"), String::new()));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs() {
        assert_eq!(derive_slug("Example Company"), "example-company");
        assert_eq!(derive_slug("  Ünï Corp "), "n-corp");
        assert_eq!(derive_slug("!!!"), "knowledge-base");
        assert_eq!(derive_slug(&"a".repeat(60)).len(), 40);
    }

    #[test]
    fn a_name_with_a_quote_gives_valid_settings() {
        let mut i = InitInput::plain("Ann \"The\" \\ Corp", "ann", Audience::Team, &[]);
        i.email_domains = vec!["example.org".into()];
        let id = uuid::Uuid::new_v4().to_string();
        let text = settings_text(&i, &id).unwrap();
        let s = settings::parse(&text, "x").unwrap();
        assert_eq!(s.name, "Ann \"The\" \\ Corp");
        assert_eq!(s.people.email_domains, vec!["example.org"]);
        let plain = InitInput::plain("Ann \"The\"", "ann", Audience::Team, &[]);
        let s = settings::parse(&settings_text(&plain, &id).unwrap(), "x").unwrap();
        assert_eq!(s.name, "Ann \"The\"");
    }
}
