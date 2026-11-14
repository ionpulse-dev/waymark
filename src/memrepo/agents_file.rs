//! The `AGENTS.md` of a new knowledge base: an introduction and the plain-Git rules
//! (see `specs/features/plain-git-access`).
//!
//! The rules are the standalone skills in `plain-git/`. They never depend on waymark.

const INTRO: &str = include_str!("templates/agents-intro.md");
const PERSONAL: &str = include_str!("../../plain-git/personal-agent-memory/SKILL.md");
const TEAM: &str = include_str!("../../plain-git/team-agent-memory/SKILL.md");

const UPDATE_HEADING: &str = "### Update of this skill";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Audience {
    Personal,
    Team,
}

impl Audience {
    pub fn skill_name(self) -> &'static str {
        match self {
            Audience::Personal => "personal-agent-memory",
            Audience::Team => "team-agent-memory",
        }
    }

    fn skill(self) -> &'static str {
        match self {
            Audience::Personal => PERSONAL,
            Audience::Team => TEAM,
        }
    }
}

/// The value of a key in the frontmatter of a skill.
pub fn frontmatter_value(skill: &str, key: &str) -> Option<String> {
    let mut lines = skill.lines();
    if lines.next()? != "---" {
        return None;
    }
    for l in lines {
        if l == "---" {
            break;
        }
        if let Some(v) = l.strip_prefix(&format!("{key}:")) {
            return Some(v.trim().to_string());
        }
    }
    None
}

/// The version and the body of a skill: no frontmatter and no section "Update of this skill".
pub fn strip(skill: &str) -> (String, String) {
    let version = frontmatter_value(skill, "version").expect("a skill has a version");
    let after = skill
        .strip_prefix("---\n")
        .and_then(|r| r.split_once("\n---\n"))
        .map(|(_, body)| body)
        .expect("a skill has a frontmatter");
    let body = match after.find(&format!("\n{UPDATE_HEADING}")) {
        Some(i) => &after[..i + 1],
        None => panic!("a skill has the section \"Update of this skill\""),
    };
    (version, body.trim_matches('\n').to_string())
}

/// The version of the skill that a release carries.
pub fn skill_version(audience: Audience) -> String {
    strip(audience.skill()).0
}

/// Build the text of `AGENTS.md`.
pub fn render(audience: Audience, email_domains: &[String]) -> String {
    let (version, body) = strip(audience.skill());
    let mut out = String::new();
    out.push_str(INTRO.trim_end_matches('\n'));
    out.push_str("\n\n");
    if audience == Audience::Team && !email_domains.is_empty() {
        out.push_str(&format!(
            "## Team settings\n\nAllowed email domains for commits: {}.\n\n",
            email_domains.join(", ")
        ));
    }
    out.push_str("## Plain-Git rules\n\n");
    out.push_str(&format!(
        "<!-- plain-git-rules begin: {} {version} -->\n",
        audience.skill_name()
    ));
    out.push_str(&body);
    out.push_str("\n<!-- plain-git-rules end -->\n\n");
    out.push_str("## Rules of this team\n\n");
    out.push_str("Add your own rules here. waymark never changes the text of this file.\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_skills_have_the_frontmatter_fields() {
        for s in [PERSONAL, TEAM] {
            for k in ["name", "description", "version"] {
                assert!(frontmatter_value(s, k).is_some(), "{k}");
            }
        }
    }

    #[test]
    fn strip_removes_the_frontmatter_and_the_update_section() {
        for a in [Audience::Personal, Audience::Team] {
            let (v, body) = strip(a.skill());
            assert!(!v.is_empty());
            assert!(!body.starts_with("---"));
            assert!(!body.lines().any(|l| l.starts_with("name:")));
            assert!(!body
                .lines()
                .any(|l| l.starts_with('#') && l.contains("Update of this skill")));
            assert!(body.contains("### Secrets"));
        }
    }

    #[test]
    fn the_skills_do_not_depend_on_waymark() {
        for s in [PERSONAL, TEAM] {
            assert!(!s.contains("settings.toml"));
            // The skills may name the marker folder and the standard, but not the program.
            let without = s
                .replace("waymark-trail", "")
                .replace(".waymark", "")
                .replace("Waymark Memory Standard", "");
            assert!(!without.to_lowercase().contains("waymark"));
        }
    }

    #[test]
    fn the_team_skill_has_every_heading_of_the_personal_skill_in_order() {
        let heads = |s: &str| -> Vec<String> {
            s.lines()
                .filter(|l| l.starts_with("### "))
                .map(String::from)
                .collect()
        };
        assert_eq!(heads(PERSONAL), heads(TEAM));
    }

    #[test]
    fn render_has_markers_domains_and_a_sane_size() {
        let none: Vec<String> = vec![];
        let p = render(Audience::Personal, &["example.org".into()]);
        assert!(p.starts_with("# Rules for agents"));
        assert!(!p.contains("Team settings"));
        assert!(p.contains("begin: personal-agent-memory "));
        let t = render(
            Audience::Team,
            &["example.org".into(), "example.net".into()],
        );
        assert!(t.contains("Allowed email domains for commits: example.org, example.net.\n"));
        assert!(!render(Audience::Team, &none).contains("Team settings"));
        for text in [&p, &t] {
            assert!(text.len() < 40 * 1024);
            assert!(!text.contains('\r') && text.ends_with('\n') && !text.ends_with("\n\n"));
            assert!(text.contains("<!-- plain-git-rules end -->"));
            assert!(text
                .trim_end()
                .ends_with("never changes the text of this file."));
        }
    }
}
