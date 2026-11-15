//! Resolve the commit author: argument, then environment variable, then the Git settings.

use super::AuthorProblem;
use crate::gitsync::cmd;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Source {
    Argument,
    Environment,
    GitConfig,
    /// The person who signed in to the personal server.
    SignIn,
}

impl Source {
    pub fn name(self) -> &'static str {
        match self {
            Source::Argument => "argument",
            Source::Environment => "environment",
            Source::GitConfig => "git_config",
            Source::SignIn => "sign_in",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Resolved {
    pub name: String,
    pub email: String,
    pub name_source: Source,
    pub email_source: Source,
}

impl Resolved {
    pub fn author(&self) -> cmd::Author {
        cmd::Author {
            name: self.name.clone(),
            email: self.email.clone(),
        }
    }
}

fn clean(v: Option<&str>) -> Option<String> {
    v.map(str::trim).filter(|s| !s.is_empty()).map(String::from)
}

fn pick(
    arg: Option<&str>,
    env: Option<&str>,
    git: impl Fn() -> Option<String>,
) -> Option<(String, Source)> {
    clean(arg)
        .map(|v| (v, Source::Argument))
        .or_else(|| clean(env).map(|v| (v, Source::Environment)))
        .or_else(|| git().map(|v| (v, Source::GitConfig)))
}

/// Each part is resolved on its own. `dir` is the clone, so its local Git config counts.
pub fn resolve(
    name_arg: Option<&str>,
    email_arg: Option<&str>,
    name_env: Option<&str>,
    email_env: Option<&str>,
    dir: &Path,
) -> Result<Resolved, AuthorProblem> {
    let git = |key: &'static str| {
        move || {
            cmd::run(dir, None, &["config", "--get", key])
                .ok()
                .filter(|o| o.ok)
                .and_then(|o| clean(Some(&o.stdout)))
        }
    };
    let name = pick(name_arg, name_env, git("user.name"));
    let email = pick(email_arg, email_env, git("user.email"));
    match (name, email) {
        (Some((name, ns)), Some((email, es))) => Ok(Resolved {
            name,
            email,
            name_source: ns,
            email_source: es,
        }),
        (n, e) => {
            let missing = match (n.is_none(), e.is_none()) {
                (true, true) => "name and email",
                (true, false) => "name",
                _ => "email",
            };
            Err(AuthorProblem {
                message: format!("The commit author {missing} is not set."),
                next: "Set git config user.name and user.email, or pass --author-name and --author-email, or set WAYMARK_AUTHOR_NAME and WAYMARK_AUTHOR_EMAIL.".to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_order_is_argument_environment_git() {
        let git = || Some("G".to_string());
        assert_eq!(pick(Some("A"), Some("E"), git).unwrap().1, Source::Argument);
        assert_eq!(pick(None, Some("E"), git).unwrap().1, Source::Environment);
        assert_eq!(pick(None, None, git).unwrap().1, Source::GitConfig);
    }

    #[test]
    fn a_blank_value_is_missing() {
        assert!(pick(Some("  "), Some(""), || None).is_none());
        assert_eq!(
            pick(Some("  "), Some("E"), || None).unwrap().1,
            Source::Environment
        );
    }
}
