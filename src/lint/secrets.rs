//! The secret scan. It reports the rule and the line, never the matched text.

use crate::error::{Error, Result};
use regex::Regex;
use std::sync::OnceLock;

struct Builtin {
    name: &'static str,
    re: Regex,
}

fn builtin() -> &'static Vec<Builtin> {
    static RE: OnceLock<Vec<Builtin>> = OnceLock::new();
    RE.get_or_init(|| {
        let p = |name, re: &str| Builtin {
            name,
            re: Regex::new(re).unwrap(),
        };
        vec![
            p("private key", r"-----BEGIN [A-Z ]*PRIVATE KEY-----"),
            p("AWS access key ID", r"AKIA[0-9A-Z]{16}"),
            p("GitHub token", r"gh[pousr]_[A-Za-z0-9]{36,}"),
            p("GitHub token", r"github_pat_[A-Za-z0-9_]{22,}"),
            p("Slack token", r"xox[abprs]-[A-Za-z0-9-]{10,}"),
            p("API key", r"sk-[A-Za-z0-9_-]{20,}"),
            p("API key", r"[sr]k_live_[A-Za-z0-9]{16,}"),
            p("Google API key", r"AIza[0-9A-Za-z_-]{35}"),
            p(
                "JSON Web Token",
                r"eyJ[A-Za-z0-9_-]{10,}\.eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}",
            ),
            p(
                "connection string with a password",
                r"[a-z][a-z0-9+.-]*://[^\s/:@]+:[^\s/@<>*]+@",
            ),
        ]
    })
}

fn assignment() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"(?i)\b(?:password|passwd|secret|api[_-]?key|token)\s*[:=]\s*["']?([^\s"'`<>*]{8,})"#,
        )
        .unwrap()
    })
}

/// A match: the pattern name and the 1-based line. It holds no matched text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    pub rule: String,
    pub line: usize,
}

/// Compile the extra patterns of a knowledge base. A bad expression is a `setup_problem`.
pub fn compile(patterns: &[String]) -> Result<Vec<Regex>> {
    patterns
        .iter()
        .map(|p| {
            Regex::new(p).map_err(|_| {
                Error::setup(format!(
                    "The entry \"{p}\" in lint.extra_secret_patterns is not a valid regular expression."
                ))
                .with_next("Fix or remove that entry in .mkbase/waymark.toml.")
            })
        })
        .collect()
}

fn is_placeholder(value: &str) -> bool {
    let v = value.to_lowercase();
    v.contains("example") || v.contains("placeholder") || v.starts_with('$') || v.starts_with('{')
}

/// Scan a text. At most one hit per pattern and line.
pub fn scan(text: &str, custom: &[Regex]) -> Vec<Hit> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let mut push = |rule: &str| {
            let hit = Hit {
                rule: rule.to_string(),
                line: i + 1,
            };
            if !out.contains(&hit) {
                out.push(hit);
            }
        };
        for b in builtin() {
            if b.re.is_match(line) {
                push(b.name);
            }
        }
        if assignment()
            .captures_iter(line)
            .any(|c| !is_placeholder(&c[1]))
        {
            push("credential assignment");
        }
        for re in custom {
            if re.is_match(line) {
                push("custom pattern");
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules(t: &str) -> Vec<String> {
        scan(t, &[]).into_iter().map(|h| h.rule).collect()
    }

    #[test]
    fn built_in_patterns_match() {
        let gh = format!("ghp_{}", "a".repeat(36));
        let jwt = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.abcdefghij1234567890";
        assert_eq!(
            rules("-----BEGIN RSA PRIVATE KEY-----"),
            vec!["private key"]
        );
        assert_eq!(
            rules("key AKIAABCDEFGHIJKLMNOP here"),
            vec!["AWS access key ID"]
        );
        assert_eq!(rules(&gh), vec!["GitHub token"]);
        assert_eq!(
            rules("github_pat_abcdefghijklmnopqrstuvwxyz"),
            vec!["GitHub token"]
        );
        assert_eq!(rules("xoxb-1234567890-abcdef"), vec!["Slack token"]);
        assert_eq!(rules("sk-abcdefghijklmnopqrstuvwx"), vec!["API key"]);
        assert_eq!(rules(jwt), vec!["JSON Web Token"]);
        assert_eq!(
            rules("postgres://user:hunter2@db/x"),
            vec!["connection string with a password"]
        );
        assert_eq!(
            rules("password = hunter22xx"),
            vec!["credential assignment"]
        );
        assert_eq!(
            rules("API_KEY: abcd1234efgh"),
            vec!["credential assignment"]
        );
    }

    #[test]
    fn plain_text_short_values_and_placeholders_pass() {
        assert!(rules("We rotate the token every week.").is_empty());
        assert!(rules("password: <your password>").is_empty());
        assert!(rules("password = ***********").is_empty());
        assert!(rules("token = $TOKEN").is_empty());
        assert!(rules("password = short").is_empty());
        // Prose that names a token and then opens a code span is not an assignment.
        assert!(rules("as a bearer token: `Authorization: Bearer x`").is_empty());
        assert!(rules("secret: example-value-123").is_empty());
        assert!(rules("https://example.org/a/b").is_empty());
    }

    #[test]
    fn the_line_is_reported_and_never_the_match() {
        let hits = scan("fine\nstill fine\npassword: s3cretvalue", &[]);
        assert_eq!(
            hits,
            vec![Hit {
                rule: "credential assignment".into(),
                line: 3
            }]
        );
    }

    #[test]
    fn custom_patterns_work_and_a_bad_one_is_a_setup_problem() {
        let custom = compile(&["CORP-[0-9]{6}".to_string()]).unwrap();
        assert_eq!(scan("id CORP-123456", &custom)[0].rule, "custom pattern");
        let e = compile(&["[".to_string()]).unwrap_err();
        assert_eq!(e.code, crate::error::Code::SetupProblem);
        assert!(e.message.contains("["));
    }
}
