//! The commit message format: `waymark: <verb> "<title>"`. No agent tag.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verb {
    Add,
    Update,
    Dispute,
    Deprecate,
    Restore,
    Delete,
    Move,
    Undo,
    Sync,
    Init,
    AddFolder,
    RemoveFolder,
    RenameTag,
    Repair,
}

impl Verb {
    pub fn as_str(self) -> &'static str {
        match self {
            Verb::Add => "add",
            Verb::Update => "update",
            Verb::Dispute => "dispute",
            Verb::Deprecate => "deprecate",
            Verb::Restore => "restore",
            Verb::Delete => "delete",
            Verb::Move => "move",
            Verb::Undo => "undo",
            Verb::Sync => "sync",
            Verb::Init => "init",
            Verb::Repair => "repair",
            Verb::AddFolder => "add-folder",
            Verb::RemoveFolder => "remove-folder",
            Verb::RenameTag => "rename-tag",
        }
    }

    pub fn parse(s: &str) -> Option<Verb> {
        Some(match s {
            "add" => Verb::Add,
            "update" => Verb::Update,
            "dispute" => Verb::Dispute,
            "deprecate" => Verb::Deprecate,
            "restore" => Verb::Restore,
            "delete" => Verb::Delete,
            "move" => Verb::Move,
            "undo" => Verb::Undo,
            "sync" => Verb::Sync,
            "init" => Verb::Init,
            "repair" => Verb::Repair,
            "add-folder" => Verb::AddFolder,
            "remove-folder" => Verb::RemoveFolder,
            "rename-tag" => Verb::RenameTag,
            _ => return None,
        })
    }
}

pub const PREFIX: &str = "waymark: ";

/// The message of a commit. The title goes on one line.
pub fn commit_message(verb: Verb, title: &str) -> String {
    let one_line = title
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace('"', "'");
    format!("{PREFIX}{} \"{one_line}\"", verb.as_str())
}

/// Read a commit subject made by waymark: (verb, title).
pub fn parse_subject(subject: &str) -> Option<(Verb, String)> {
    let rest = subject.strip_prefix(PREFIX)?;
    let (verb, title) = rest.split_once(' ')?;
    let title = title.strip_prefix('"')?.strip_suffix('"')?;
    Some((Verb::parse(verb)?, title.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_has_the_fixed_format_and_no_agent_tag() {
        let m = commit_message(Verb::Add, "Payment API v3 uses idempotency keys");
        assert_eq!(m, "waymark: add \"Payment API v3 uses idempotency keys\"");
        assert!(!m.to_lowercase().contains("claude") && !m.contains("Co-Authored"));
    }

    #[test]
    fn the_title_is_one_line_without_double_quotes() {
        assert_eq!(
            commit_message(Verb::Update, "A\nB  \"C\""),
            "waymark: update \"A B 'C'\""
        );
    }

    #[test]
    fn subjects_can_be_read_back() {
        assert_eq!(
            parse_subject("waymark: move \"A title\""),
            Some((Verb::Move, "A title".to_string()))
        );
        assert_eq!(parse_subject("Merge branch x"), None);
        assert_eq!(parse_subject("waymark: bogus \"x\""), None);
    }

    #[test]
    fn every_verb_round_trips() {
        for v in [
            Verb::Add,
            Verb::Update,
            Verb::Dispute,
            Verb::Deprecate,
            Verb::Delete,
            Verb::Move,
            Verb::Undo,
            Verb::Sync,
            Verb::AddFolder,
            Verb::RemoveFolder,
            Verb::Restore,
            Verb::RenameTag,
        ] {
            assert_eq!(Verb::parse(v.as_str()), Some(v));
        }
    }
}
