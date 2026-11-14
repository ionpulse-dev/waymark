//! The secret scan. It reports the rule and the line, never the matched text.

use regex::Regex;
use std::sync::OnceLock;

fn builtin() -> &'static Vec<(&'static str, Regex)> {
    static RE: OnceLock<Vec<(&'static str, Regex)>> = OnceLock::new();
    RE.get_or_init(|| {
        let p = |name, re: &str| (name, Regex::new(re).unwrap());
        vec![
            p("private-key", r"-----BEGIN [A-Z ]*PRIVATE KEY-----"),
            p("aws-access-key", r"AKIA[0-9A-Z]{16}"),
            p("github-token", r"(?:ghp_|github_pat_)[A-Za-z0-9_]{16,}"),
            p("api-key", r"sk-[A-Za-z0-9]{20,}"),
            p(
                "credential-assignment",
                r#"(?i)\b(?:password|passwd|secret|token|api_key)\b\s*[=:]\s*["']?[^\s"'<$\{\[*.]{4,}"#,
            ),
            p("url-password", r"://[^/\s:@]+:[^/\s@]+@"),
        ]
    })
}

/// A match: the rule and the 1-based line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    pub rule: String,
    pub line: usize,
}

/// Compile custom patterns. A pattern that is not a valid regular expression is skipped.
pub fn compile(patterns: &[String]) -> Vec<Regex> {
    patterns.iter().filter_map(|p| Regex::new(p).ok()).collect()
}

/// Scan a text. At most one hit per rule and line.
pub fn scan(text: &str, custom: &[Regex]) -> Vec<Hit> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        for (name, re) in builtin() {
            if re.is_match(line) {
                out.push(Hit {
                    rule: (*name).to_string(),
                    line: i + 1,
                });
            }
        }
        for re in custom {
            if re.is_match(line) {
                out.push(Hit {
                    rule: "custom-pattern".to_string(),
                    line: i + 1,
                });
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
        assert_eq!(
            rules("-----BEGIN RSA PRIVATE KEY-----"),
            vec!["private-key"]
        );
        assert_eq!(
            rules("key AKIAABCDEFGHIJKLMNOP here"),
            vec!["aws-access-key"]
        );
        assert_eq!(rules("ghp_abcdefghijklmnopqrstuv"), vec!["github-token"]);
        assert_eq!(rules("sk-abcdefghijklmnopqrstuvwx"), vec!["api-key"]);
        assert_eq!(rules("password = hunter22"), vec!["credential-assignment"]);
        assert_eq!(rules("API_KEY: abc12345"), vec!["credential-assignment"]);
        assert_eq!(rules("https://user:pw@host/x"), vec!["url-password"]);
    }

    #[test]
    fn plain_text_and_placeholders_pass() {
        assert!(rules("We rotate the token every week.").is_empty());
        assert!(rules("password: <your password>").is_empty());
        assert!(rules("token = $TOKEN").is_empty());
        assert!(rules("https://example.org/a/b").is_empty());
    }

    #[test]
    fn the_line_is_reported_and_never_the_match() {
        let hits = scan("fine\nstill fine\npassword: s3cretvalue", &[]);
        assert_eq!(
            hits,
            vec![Hit {
                rule: "credential-assignment".into(),
                line: 3
            }]
        );
    }

    #[test]
    fn custom_patterns_work_and_bad_ones_are_skipped() {
        let custom = compile(&["CORP-[0-9]{6}".to_string(), "(".to_string()]);
        assert_eq!(custom.len(), 1);
        assert_eq!(scan("id CORP-123456", &custom)[0].rule, "custom-pattern");
    }
}
