//! Author checks: `author-not-allowed` (settings) and `author-mismatch` (Git history).

use crate::memrepo::settings::People;

/// The email of `Name <email>`, in lowercase.
pub fn email_of(author: &str) -> Option<String> {
    let start = author.rfind('<')?;
    let end = author.rfind('>')?;
    (end > start).then(|| author[start + 1..end].trim().to_lowercase())
}

/// Why an author is not allowed, or `None` if the settings allow it.
pub fn not_allowed(author: &str, people: &People) -> Option<String> {
    let email = email_of(author)?;
    let domain = email.rsplit('@').next().unwrap_or("");
    if !people.email_domains.is_empty()
        && !people
            .email_domains
            .iter()
            .any(|d| d.eq_ignore_ascii_case(domain))
    {
        return Some(format!(
            "The email domain \"{domain}\" is not in people.email_domains."
        ));
    }
    if let Some(b) = people
        .email_blocklist
        .iter()
        .find(|b| !b.is_empty() && email.contains(&b.to_lowercase()))
    {
        return Some(format!(
            "The email holds \"{b}\", which is in people.email_blocklist."
        ));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn people(domains: &[&str], block: &[&str]) -> People {
        People {
            email_domains: domains.iter().map(|s| s.to_string()).collect(),
            email_blocklist: block.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn the_email_is_read_from_the_author() {
        assert_eq!(
            email_of("Jo Doe <Jo@Example.org>").as_deref(),
            Some("jo@example.org")
        );
        assert_eq!(email_of("no email"), None);
    }

    #[test]
    fn domains_and_the_blocklist_apply() {
        let p = people(&["example.org"], &["noreply"]);
        assert!(not_allowed("A <a@example.org>", &p).is_none());
        assert!(not_allowed("A <a@other.test>", &p).is_some());
        assert!(not_allowed("A <noreply@example.org>", &p).is_some());
        assert!(not_allowed("A <a@anything.test>", &people(&[], &[])).is_none());
    }
}
