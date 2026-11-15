//! "Who is this person?" (see `specs/foundation/personal-server`). The door asks an identity
//! provider, and only for this. GitHub is the first provider.

use crate::identity::check_email;
use crate::memrepo::settings::People;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailEntry {
    pub address: String,
    pub verified: bool,
    pub primary: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Person {
    /// The account name, as the allow-list knows it.
    pub login: String,
    /// The display name, if the account has one.
    pub name: Option<String>,
    pub emails: Vec<EmailEntry>,
}

pub trait IdentityProvider: Send + Sync {
    /// The address where the person signs in. `callback` is where the provider sends the person back.
    fn authorize_url(&self, state: &str, callback: &str) -> String;
    /// Turn the code from the provider into the person. Blocking. The provider token is dropped.
    fn identify(&self, code: &str, callback: &str) -> Result<Person, String>;
}

/// The name and email for the commits of this person.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chosen {
    pub name: String,
    pub email: String,
}

pub const NO_EMAIL: &str = "Add and verify an email address on your GitHub account.";

/// Accept only an address that the provider marks as verified and that passes the email check of
/// `login-and-identity`. Prefer the primary one.
pub fn choose(person: &Person, people: &People) -> Result<Chosen, String> {
    let mut ok: Vec<&EmailEntry> = person
        .emails
        .iter()
        .filter(|e| e.verified && check_email(&e.address, people).is_ok())
        .collect();
    ok.sort_by_key(|e| !e.primary);
    match ok.first() {
        Some(e) => Ok(Chosen {
            name: person
                .name
                .clone()
                .filter(|n| !n.trim().is_empty())
                .unwrap_or_else(|| person.login.clone()),
            email: e.address.clone(),
        }),
        None => Err(NO_EMAIL.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(a: &str, verified: bool, primary: bool) -> EmailEntry {
        EmailEntry {
            address: a.into(),
            verified,
            primary,
        }
    }

    fn person(emails: Vec<EmailEntry>) -> Person {
        Person {
            login: "octo".into(),
            name: Some("Octo Cat".into()),
            emails,
        }
    }

    #[test]
    fn the_primary_verified_address_wins() {
        let p = person(vec![
            e("second@example.org", true, false),
            e("main@example.org", true, true),
        ]);
        let c = choose(&p, &People::default()).unwrap();
        assert_eq!(c.email, "main@example.org");
        assert_eq!(c.name, "Octo Cat");
    }

    #[test]
    fn unverified_and_noreply_addresses_are_refused() {
        let p = person(vec![
            e("a@example.org", false, true),
            e("1+octo@users.noreply.github.com", true, false),
        ]);
        assert_eq!(choose(&p, &People::default()), Err(NO_EMAIL.to_string()));
    }

    #[test]
    fn a_verified_second_address_is_used_when_the_primary_is_blocked() {
        let p = person(vec![
            e("1+octo@users.noreply.github.com", true, true),
            e("real@example.org", true, false),
        ]);
        assert_eq!(
            choose(&p, &People::default()).unwrap().email,
            "real@example.org"
        );
    }

    #[test]
    fn the_domain_list_is_applied() {
        let people = People {
            email_domains: vec!["corp.example".into()],
            ..People::default()
        };
        let p = person(vec![e("a@other.example", true, true)]);
        assert!(choose(&p, &people).is_err());
        let p = person(vec![e("a@corp.example", true, true)]);
        assert!(choose(&p, &people).is_ok());
    }

    #[test]
    fn the_login_is_the_name_when_there_is_no_display_name() {
        let mut p = person(vec![e("a@example.org", true, true)]);
        p.name = None;
        assert_eq!(choose(&p, &People::default()).unwrap().name, "octo");
    }
}
