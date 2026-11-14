//! Automatic repairs of the settings file `.waymark/settings.toml` (see `memory-repository`,
//! section "Versions and updates").
//!
//! A repair adds what is missing and never changes a value that exists. It comes from this list
//! only, and it never touches the limits or the approval rules that an owner set. A repair is safe
//! to run twice. A change that older builds cannot read is a migration, not a repair.

use super::settings::SCHEMA_VERSION;

pub struct Repair {
    pub name: &'static str,
    /// The sentence for the notice of the call that made the repair.
    pub notice: &'static str,
    /// The fixed text, or `None` if the file needs nothing from this repair.
    pub apply: fn(&str) -> Option<String>,
}

/// A file without the key `schema` is schema 1. The repair writes the key, after the header
/// comment if the file has one.
fn add_schema_key(text: &str) -> Option<String> {
    let table: toml::Table = text.parse().ok()?;
    if table.contains_key("schema") {
        return None;
    }
    // Keep a header comment on top: the key goes after the leading comment lines and the empty
    // lines that follow them.
    let mut at = 0;
    for line in text.split_inclusive('\n') {
        if line.trim_start().starts_with('#') {
            at += line.len();
        } else {
            break;
        }
    }
    for line in text[at..].split_inclusive('\n') {
        if line.trim().is_empty() {
            at += line.len();
        } else {
            break;
        }
    }
    Some(format!(
        "{}schema = {SCHEMA_VERSION}\n{}",
        &text[..at],
        &text[at..]
    ))
}

pub const REPAIRS: &[Repair] = &[Repair {
    name: "add-schema-key",
    notice: "Repaired the settings file: added the missing schema key.",
    apply: add_schema_key,
}];

/// Run every repair, in order. Returns the new text and the repairs that changed it, or `None`.
pub fn repair(text: &str) -> Option<(String, Vec<&'static Repair>)> {
    let mut current = text.to_string();
    let mut done = Vec::new();
    for r in REPAIRS {
        if let Some(next) = (r.apply)(&current) {
            current = next;
            done.push(r);
        }
    }
    if done.is_empty() {
        None
    } else {
        Some((current, done))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID: &str = "123e4567-e89b-42d3-a456-426614174000";

    #[test]
    fn a_file_without_schema_gets_the_key_and_keeps_everything_else() {
        let text = format!("id = \"{ID}\"\n[limits]\nmax_results = 3 # tight\n");
        let (fixed, done) = repair(&text).unwrap();
        assert_eq!(done.len(), 1);
        assert_eq!(fixed, format!("schema = 1\n{text}"));
        let table: toml::Table = fixed.parse().unwrap();
        assert_eq!(table["schema"].as_integer(), Some(1));
    }

    #[test]
    fn a_header_comment_stays_on_top() {
        let text = format!("# My header.\n# Second line.\n\nname = \"X\"\nid = \"{ID}\"\n");
        let (fixed, _) = repair(&text).unwrap();
        assert_eq!(
            fixed,
            format!("# My header.\n# Second line.\n\nschema = 1\nname = \"X\"\nid = \"{ID}\"\n")
        );
        let text = format!("# Header.\nname = \"X\"\nid = \"{ID}\"\n");
        let (fixed, _) = repair(&text).unwrap();
        assert!(fixed.starts_with("# Header.\nschema = 1\nname"), "{fixed}");
    }

    #[test]
    fn a_current_file_needs_no_repair_and_a_repair_is_safe_twice() {
        let text = format!("schema = 1\nid = \"{ID}\"\n");
        assert!(repair(&text).is_none());
        let (fixed, _) = repair(&format!("id = \"{ID}\"\n")).unwrap();
        assert!(repair(&fixed).is_none());
    }

    #[test]
    fn a_file_that_is_not_toml_is_left_alone() {
        assert!(repair("this is = = not toml").is_none());
    }

    #[test]
    fn a_higher_or_bad_schema_is_not_touched() {
        assert!(repair("schema = 2\n").is_none());
        assert!(repair("schema = \"x\"\n").is_none());
    }
}
