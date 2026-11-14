//! One read call to the remote, for `status` (see `specs/features/maintenance`).

use super::cmd;
use super::Repo;
use crate::identity::{is_login_failure, redact_text};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Probe {
    Reachable,
    NoRemote,
    LoginRefused(String),
    Unreachable(String),
}

impl Repo {
    /// `git ls-remote origin HEAD` with a time-out. It changes nothing.
    pub fn probe_remote(&self, seconds: u64) -> Probe {
        if !self.has_remote() {
            return Probe::NoRemote;
        }
        match cmd::run_timeout(&self.dir, &["ls-remote", "origin", "HEAD"], seconds) {
            Ok(Some(o)) if o.ok => Probe::Reachable,
            Ok(Some(o)) => {
                let line = redact_text(&cmd::first_line(&o.stderr));
                if is_login_failure(&o.stderr) {
                    Probe::LoginRefused(line)
                } else {
                    Probe::Unreachable(line)
                }
            }
            Ok(None) => Probe::Unreachable(format!("no answer in {seconds} seconds")),
            Err(e) => Probe::Unreachable(e.message),
        }
    }
}
