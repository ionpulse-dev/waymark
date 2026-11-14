//! Who is the author, and does the address reach a person (see `specs/features/login-and-identity`).
//!
//! The login itself is left to `git`. This module resolves the author, checks the email offline,
//! and removes passwords from text that leaves the server.

pub mod author;
pub mod email;
pub mod redact;

pub use author::{resolve, Resolved, Source};
pub use email::check_email;
pub use redact::{redact_text, redact_url};

use crate::error::Error;

/// Why the author cannot be used. It maps to `setup_problem`.
#[derive(Debug, Clone, PartialEq)]
pub struct AuthorProblem {
    pub message: String,
    pub next: String,
}

impl From<AuthorProblem> for Error {
    fn from(p: AuthorProblem) -> Error {
        Error::setup(p.message).with_next(p.next)
    }
}

/// The fixed text for a failed Git login.
pub const LOGIN_NEXT: &str = "Git cannot sign in to this repository. Sign in with Git first, for example with `gh auth login`, or set up an SSH key.";

/// Does the text of a Git error say that the login failed?
pub fn is_login_failure(text: &str) -> bool {
    let t = text.to_lowercase();
    [
        "permission denied (publickey",
        "authentication failed",
        "could not read username",
        "could not read password",
        "terminal prompts disabled",
        "invalid username or password",
        "host key verification failed",
    ]
    .iter()
    .any(|m| t.contains(m))
}
