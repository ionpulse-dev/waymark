//! The offline check that an address reaches a person.

use super::AuthorProblem;
use crate::kb::settings::People;

const BLOCKED: &[&str] = &[
    "noreply",
    "no-reply",
    "no_reply",
    "donotreply",
    "do-not-reply",
    "do_not_reply",
    "dontreply",
    "dont-reply",
    "noanswer",
    "no-answer",
    "users.noreply.github.com",
    "noreply.gitlab.com",
    "localhost",
    "localdomain",
    "nobody@",
    "null@",
    "devnull",
    "mailer-daemon",
    "postmaster@",
    "bounce",
    "bounces",
    "notifications@",
    "notification@",
    "automated",
    "bot@",
    "[bot]",
    "test@",
    "placeholder",
    "changeme",
    "unknown@",
    "none@",
];

fn problem(address: &str, why: &str) -> AuthorProblem {
    AuthorProblem {
        message: format!("The commit email {address} {why}."),
        next: "Set a real address with --author-email, WAYMARK_AUTHOR_EMAIL, or git config user.email.".to_string(),
    }
}

/// Check syntax, the blocklists, and the allowed domains. No network.
pub fn check_email(address: &str, people: &People) -> Result<(), AuthorProblem> {
    let a = address.trim().to_lowercase();
    let syntax = || problem(address, "is not a valid address");
    if a.chars().any(char::is_whitespace) || a.matches('@').count() != 1 {
        return Err(syntax());
    }
    let (local, domain) = a.split_once('@').unwrap();
    if local.is_empty() || !domain.contains('.') || domain.split('.').any(str::is_empty) {
        return Err(syntax());
    }
    let extra = people.email_blocklist.iter().map(|s| s.to_lowercase());
    for part in BLOCKED
        .iter()
        .map(|s| s.to_string())
        .chain(extra)
        .filter(|p| !p.is_empty())
    {
        if a.contains(&part) {
            return Err(problem(
                address,
                &format!("cannot receive mail (it contains \"{part}\")"),
            ));
        }
    }
    if !people.email_domains.is_empty()
        && !people
            .email_domains
            .iter()
            .any(|d| d.eq_ignore_ascii_case(domain))
    {
        return Err(AuthorProblem {
            message: format!(
                "The commit email {address} is not in an allowed domain ({}).",
                people.email_domains.join(", ")
            ),
            next: "Use an address in an allowed domain with --author-email, WAYMARK_AUTHOR_EMAIL, or git config user.email.".to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn none() -> People {
        People::default()
    }

    #[test]
    fn a_normal_address_passes_also_on_example_domains() {
        assert!(check_email("jane@example.org", &none()).is_ok());
        assert!(check_email(" Jane.Doe@corp.example.org ", &none()).is_ok());
    }

    #[test]
    fn syntax_errors_fail() {
        for a in ["jane", "a@b", "@x.org", "a@@x.org", "a b@x.org", "a@x..org"] {
            assert!(check_email(a, &none()).is_err(), "{a}");
        }
    }

    #[test]
    fn every_default_part_is_blocked() {
        for part in BLOCKED {
            let a = if part.ends_with('@') {
                format!("{part}corp.org")
            } else {
                format!("x{part}@corp.org")
            };
            let a = if part.contains('.') {
                format!("jane@{part}")
            } else {
                a
            };
            if let Err(p) = check_email(&a, &none()) {
                assert!(p.message.contains("cannot receive mail"), "{a}");
            } else {
                panic!("{a} passed");
            }
        }
        let e = check_email("jane@users.noreply.github.com", &none()).unwrap_err();
        assert!(e.message.contains("cannot receive mail"));
    }

    #[test]
    fn extra_blocklist_and_domains() {
        let p = People {
            email_blocklist: vec!["example.org".into()],
            email_domains: vec![],
        };
        assert!(check_email("jane@example.org", &p).is_err());
        let p = People {
            email_blocklist: vec![],
            email_domains: vec!["corp.example.org".into()],
        };
        assert!(check_email("jane@corp.example.org", &p).is_ok());
        assert!(check_email("jane@other.org", &p).is_err());
        assert!(check_email("jane@sub.corp.example.org", &p).is_err());
    }
}
