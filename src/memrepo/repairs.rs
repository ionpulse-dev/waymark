//! Automatic repairs (see `memory-repository`, section "Versions and updates", and
//! `clean-git-status`): of the settings file `.waymark/settings.toml`, and of the files
//! `.gitignore` and `.obsidian/app.json` that an older `init` wrote.
//!
//! A repair adds what is missing and never changes a value that exists. It comes from this list
//! only, and it never touches the limits or the approval rules that an owner set. A repair is safe
//! to run twice. A change that older builds cannot read is a migration, not a repair.

use super::init::OBSIDIAN_TEMPLATE;
use super::settings::SCHEMA_VERSION;

/// The file that a repair fixes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// `.waymark/settings.toml`, below the root of the memory repository.
    Settings,
    /// `.gitignore` at the repository folder.
    Gitignore,
    /// `.obsidian/app.json` at the repository folder.
    ObsidianApp,
}

impl Target {
    /// All targets, in the order of the repair step.
    pub const ALL: [Target; 3] = [Target::Settings, Target::Gitignore, Target::ObsidianApp];
}

pub struct Repair {
    pub name: &'static str,
    pub target: Target,
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

const DS_STORE_COMMENT: &str = "# macOS: Finder writes these files in every folder that it opens.";

/// A `.gitignore` without a line for `.DS_Store` gets the comment and the line at the end.
fn add_ds_store(text: &str) -> Option<String> {
    if text
        .lines()
        .any(|l| matches!(l.trim(), ".DS_Store" | "**/.DS_Store"))
    {
        return None;
    }
    let mut out = text.to_string();
    if !out.is_empty() {
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out.push('\n');
    }
    out.push_str(DS_STORE_COMMENT);
    out.push_str("\n.DS_Store\n");
    Some(out)
}

/// An `app.json` with exactly the two waymark settings, but not in the bytes that Obsidian writes,
/// gets those bytes. A file with other keys or values belongs to the person.
fn obsidian_format(text: &str) -> Option<String> {
    if text == OBSIDIAN_TEMPLATE {
        return None;
    }
    let v: serde_json::Value = serde_json::from_str(text).ok()?;
    let map = v.as_object()?;
    let only_waymark = map.len() == 2
        && map.get("useMarkdownLinks") == Some(&serde_json::Value::Bool(true))
        && map.get("newLinkFormat").and_then(|v| v.as_str()) == Some("relative");
    only_waymark.then(|| OBSIDIAN_TEMPLATE.to_string())
}

pub const REPAIRS: &[Repair] = &[
    Repair {
        name: "add-schema-key",
        target: Target::Settings,
        notice: "Repaired the settings file: added the missing schema key.",
        apply: add_schema_key,
    },
    Repair {
        name: "gitignore-ds-store",
        target: Target::Gitignore,
        notice: "Repaired .gitignore: Git now ignores the .DS_Store files of macOS.",
        apply: add_ds_store,
    },
    Repair {
        name: "obsidian-app-json-format",
        target: Target::ObsidianApp,
        notice: "Repaired .obsidian/app.json: it now has the format that Obsidian writes.",
        apply: obsidian_format,
    },
];

/// Run every repair of one file, in order. Returns the new text and the repairs that changed it,
/// or `None`.
pub fn repair(target: Target, text: &str) -> Option<(String, Vec<&'static Repair>)> {
    let mut current = text.to_string();
    let mut done = Vec::new();
    for r in REPAIRS.iter().filter(|r| r.target == target) {
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
        let (fixed, done) = repair(Target::Settings, &text).unwrap();
        assert_eq!(done.len(), 1);
        assert_eq!(fixed, format!("schema = 1\n{text}"));
        let table: toml::Table = fixed.parse().unwrap();
        assert_eq!(table["schema"].as_integer(), Some(1));
    }

    #[test]
    fn a_header_comment_stays_on_top() {
        let text = format!("# My header.\n# Second line.\n\nname = \"X\"\nid = \"{ID}\"\n");
        let (fixed, _) = repair(Target::Settings, &text).unwrap();
        assert_eq!(
            fixed,
            format!("# My header.\n# Second line.\n\nschema = 1\nname = \"X\"\nid = \"{ID}\"\n")
        );
        let text = format!("# Header.\nname = \"X\"\nid = \"{ID}\"\n");
        let (fixed, _) = repair(Target::Settings, &text).unwrap();
        assert!(fixed.starts_with("# Header.\nschema = 1\nname"), "{fixed}");
    }

    #[test]
    fn a_current_file_needs_no_repair_and_a_repair_is_safe_twice() {
        let text = format!("schema = 1\nid = \"{ID}\"\n");
        assert!(repair(Target::Settings, &text).is_none());
        let (fixed, _) = repair(Target::Settings, &format!("id = \"{ID}\"\n")).unwrap();
        assert!(repair(Target::Settings, &fixed).is_none());
    }

    #[test]
    fn a_file_that_is_not_toml_is_left_alone() {
        assert!(repair(Target::Settings, "this is = = not toml").is_none());
    }

    #[test]
    fn a_higher_or_bad_schema_is_not_touched() {
        assert!(repair(Target::Settings, "schema = 2\n").is_none());
        assert!(repair(Target::Settings, "schema = \"x\"\n").is_none());
    }

    const OLD_GITIGNORE: &str = "# Obsidian: share only the link settings. All other files are personal.\n.obsidian/*\n!.obsidian/app.json\n";
    const OLD_APP_JSON: &str =
        "{\n  \"useMarkdownLinks\": true,\n  \"newLinkFormat\": \"relative\"\n}\n";

    // U1
    #[test]
    fn a_gitignore_without_ds_store_gets_the_lines_at_the_end() {
        let (fixed, done) = repair(Target::Gitignore, OLD_GITIGNORE).unwrap();
        assert_eq!(done.len(), 1);
        assert_eq!(
            fixed,
            format!("{OLD_GITIGNORE}\n{DS_STORE_COMMENT}\n.DS_Store\n")
        );
        // A text with no final line end gets one first.
        let (fixed, _) = repair(Target::Gitignore, ".obsidian/*").unwrap();
        assert_eq!(
            fixed,
            format!(".obsidian/*\n\n{DS_STORE_COMMENT}\n.DS_Store\n")
        );
        // An empty file gets the lines only.
        let (fixed, _) = repair(Target::Gitignore, "").unwrap();
        assert_eq!(fixed, format!("{DS_STORE_COMMENT}\n.DS_Store\n"));
        // The new template of `init` is what the repair gives for the old one.
        let (fixed, _) = repair(Target::Gitignore, OLD_GITIGNORE).unwrap();
        assert_eq!(fixed, include_str!("templates/gitignore"));
    }

    // U2
    #[test]
    fn a_gitignore_with_ds_store_needs_no_repair() {
        for line in [".DS_Store", "**/.DS_Store", "  .DS_Store  "] {
            let text = format!("target/\n{line}\nnode_modules/\n");
            assert!(repair(Target::Gitignore, &text).is_none(), "{line}");
        }
        assert!(repair(Target::Gitignore, include_str!("templates/gitignore")).is_none());
    }

    // U3
    #[test]
    fn the_old_app_json_gets_the_bytes_of_obsidian() {
        let (fixed, done) = repair(Target::ObsidianApp, OLD_APP_JSON).unwrap();
        assert_eq!(done.len(), 1);
        assert_eq!(fixed, OBSIDIAN_TEMPLATE);
        // Another key order or indent is the same file.
        let other = "{\"newLinkFormat\":\"relative\",\n\t\"useMarkdownLinks\":true}";
        assert_eq!(
            repair(Target::ObsidianApp, other).unwrap().0,
            OBSIDIAN_TEMPLATE
        );
    }

    // U4
    #[test]
    fn an_app_json_of_the_person_is_left_alone() {
        for text in [
            "{\n  \"useMarkdownLinks\": true,\n  \"newLinkFormat\": \"relative\",\n  \"vimMode\": true\n}",
            "{\n  \"useMarkdownLinks\": false,\n  \"newLinkFormat\": \"relative\"\n}\n",
            "{\n  \"useMarkdownLinks\": true,\n  \"newLinkFormat\": \"shortest\"\n}\n",
            "{\n  \"useMarkdownLinks\": true\n}\n",
            "not json",
            "[]",
            OBSIDIAN_TEMPLATE,
        ] {
            assert!(repair(Target::ObsidianApp, text).is_none(), "{text}");
        }
    }

    // U5
    #[test]
    fn every_repair_is_safe_to_run_twice() {
        for (target, text) in [
            (Target::Gitignore, OLD_GITIGNORE),
            (Target::Gitignore, ""),
            (Target::ObsidianApp, OLD_APP_JSON),
        ] {
            let (fixed, _) = repair(target, text).unwrap();
            assert!(repair(target, &fixed).is_none(), "{target:?}");
        }
    }

    #[test]
    fn a_repair_touches_only_its_own_file() {
        // The settings repair does not run on a `.gitignore`, and the reverse.
        assert!(repair(Target::Settings, OLD_GITIGNORE).is_none());
        assert!(repair(Target::Gitignore, "id = \"x\"\n.DS_Store\n").is_none());
        assert!(REPAIRS.iter().all(|r| Target::ALL.contains(&r.target)));
    }
}
