//! The command line and the environment variables (see `specs/foundation/mcp-server`).

use crate::config::{Access, Limits};
use clap::{CommandFactory, Parser};
use std::path::PathBuf;

#[derive(Parser, Debug, Clone, Default)]
#[command(
    name = "waymark",
    version,
    about = "Long-term agent memory. Starts an MCP server over stdio.",
    after_help = "One of `--repo` and `--path` is required. Every other option is optional. Combine as many as you need, in any order.\n\
Your agent tool starts waymark, so the options usually go into the `args` list of its MCP entry.\n\n\
Examples:\n  \
waymark --repo git@github.com:example/waymark-agent-memory.git\n  \
waymark --repo git@github.com:example/waymark-agent-memory.git --access read-only --max-results 3\n  \
waymark --path ~/vaults/team-memory --folders ops,docs/* --no-delete\n\n\
Flags with a value take it after a space or `=`. Lists take comma-separated values with no spaces."
)]
pub struct Args {
    /// Git URL of the memory repository. The server clones it on first start, into `--path` or into `~/.waymark/<name>`. One of `--repo` and `--path` is required.
    #[arg(long)]
    pub repo: Option<String>,
    /// The ID that `.waymark/settings.toml` must have. Stops the server from opening a different memory repository by mistake.
    #[arg(long)]
    pub memory_repository_id: Option<String>,
    /// Folder of the memory repository. `~` is your home folder. With `--repo`, a missing or empty folder is the clone target. Without `--path`, `--repo` uses `~/.waymark/<name>`.
    #[arg(long)]
    pub path: Option<PathBuf>,
    /// Name for your changes. Default: your Git `user.name`.
    #[arg(long)]
    pub author_name: Option<String>,
    /// Email for your changes. Default: your Git `user.email`.
    #[arg(long)]
    pub author_email: Option<String>,
    /// `read-only` or `read-write`.
    #[arg(long)]
    pub access: Option<String>,
    /// Pause all writes from the start of the server.
    #[arg(long)]
    pub paused: bool,
    /// Switch off the delete tool.
    #[arg(long)]
    pub no_delete: bool,
    /// Comma-separated tool names to switch off.
    #[arg(long, value_delimiter = ',')]
    pub tools_denied: Vec<String>,
    /// Comma-separated folder globs. Only these folders are allowed.
    #[arg(long, value_delimiter = ',')]
    pub folders: Vec<String>,
    /// Most documents that `search` returns per call. Default: 5.
    #[arg(long)]
    pub max_results: Option<usize>,
    /// Most tokens that `search` returns per call. Default: 800.
    #[arg(long)]
    pub max_tokens: Option<usize>,
    /// Most documents that `show` opens in one session. Default: 20.
    #[arg(long)]
    pub max_documents_opened: Option<usize>,
    /// `local` (the default, stdio), `personal-server`, or later `team-server`. Also `WAYMARK_MODE`.
    #[arg(long, env = "WAYMARK_MODE")]
    pub mode: Option<String>,
    /// Personal server only: print the client ID and secret for apps that need them by hand, and
    /// exit. Reads `WAYMARK_SIGNING_KEY` (or the file of `WAYMARK_SIGNING_KEY_FILE`).
    #[arg(long)]
    pub print_client: bool,
    /// Personal server only: the address to listen on. Default: `0.0.0.0:$PORT` (8080).
    #[arg(long)]
    pub listen: Option<String>,
    /// Problems of the MCP entry that the lenient parser found. Not a command line argument.
    #[arg(skip)]
    pub entry_issues: Vec<EntryIssue>,
}

/// A problem with the arguments of the MCP entry (see `specs/foundation/mcp-server`, section
/// "Updates of the MCP entry").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryIssue {
    pub message: String,
    pub next: String,
    /// A blocking issue stops every tool except `status`. Otherwise it is only a notice.
    pub blocking: bool,
}

/// Arguments that no longer have an effect, with the sentence for the notice. A flag in this list
/// is ignored. Any other unknown flag blocks the tools, because it may come from a newer waymark
/// and may be a limit.
pub const DEPRECATED_FLAGS: &[(&str, &str)] = &[];

fn shell_quote(s: &str) -> String {
    if !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_./:@=,*~".contains(c))
    {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', "'\\''"))
    }
}

/// The commands that replace the entry of Claude Code with the given arguments.
fn fix_commands(kept: &[String]) -> String {
    let args: Vec<String> = kept.iter().map(|a| shell_quote(a)).collect();
    format!(
        "`claude mcp remove waymark`, then `claude mcp add --transport stdio --scope user waymark -- waymark {}`",
        args.join(" ")
    )
}

impl Args {
    /// Parse the command line without exiting on a problem. An argument that this waymark does
    /// not know, or a wrong value, does not stop the server. It becomes an `EntryIssue`, so that
    /// `initialize`, `status`, and every tool can say what to change. `--help` and `--version`
    /// work as always.
    pub fn parse_lenient() -> Args {
        Args::parse_lenient_from(std::env::args().collect(), DEPRECATED_FLAGS)
    }

    pub fn parse_lenient_from(argv: Vec<String>, deprecated: &[(&str, &str)]) -> Args {
        let cmd = Args::command();
        let mut known: Vec<String> = cmd
            .get_arguments()
            .filter_map(|a| a.get_long().map(String::from))
            .collect();
        known.push("help".into());
        known.push("version".into());
        let program = argv.first().cloned().unwrap_or_else(|| "waymark".into());
        let mut kept: Vec<String> = Vec::new();
        let mut issues: Vec<EntryIssue> = Vec::new();
        let mut unknown: Vec<String> = Vec::new();
        let mut i = 1;
        while i < argv.len() {
            let tok = &argv[i];
            let is_flag = tok.starts_with('-') && tok.len() > 1;
            let name = tok.trim_start_matches('-').split('=').next().unwrap_or("");
            let is_known = tok.starts_with("--") && known.iter().any(|k| k == name)
                || matches!(tok.as_str(), "-h" | "-V");
            if !is_flag || is_known {
                kept.push(tok.clone());
                i += 1;
                continue;
            }
            // An unknown flag. Drop it and its value, if it has one.
            let takes_value = !tok.contains('=')
                && argv
                    .get(i + 1)
                    .is_some_and(|n| !(n.starts_with('-') && n.len() > 1));
            i += if takes_value { 2 } else { 1 };
            if let Some((_, why)) = deprecated.iter().find(|(f, _)| *f == format!("--{name}")) {
                issues.push(EntryIssue {
                    message: format!("The MCP entry uses `--{name}`. {why}"),
                    next: "Remove it from the entry when you next change it.".to_string(),
                    blocking: false,
                });
            } else {
                unknown.push(tok.split('=').next().unwrap_or(tok).to_string());
            }
        }
        let mut argv_kept = vec![program];
        argv_kept.extend(kept.iter().cloned());
        let mut args = match Args::try_parse_from(&argv_kept) {
            Ok(a) => a,
            Err(e) => {
                use clap::error::ErrorKind;
                if matches!(e.kind(), ErrorKind::DisplayHelp | ErrorKind::DisplayVersion) {
                    e.exit();
                }
                let text = e.to_string();
                let first = text
                    .lines()
                    .next()
                    .unwrap_or("")
                    .trim_start_matches("error: ")
                    .to_string();
                issues.push(EntryIssue {
                    message: format!("The MCP entry has a wrong argument: {first}"),
                    next: format!(
                        "Change the entry: {}. Then restart your agent tool.",
                        fix_commands(&[])
                    ),
                    blocking: true,
                });
                // Without valid arguments the server must not guess a memory repository.
                Args::default()
            }
        };
        if !unknown.is_empty() {
            issues.push(EntryIssue {
                message: format!(
                    "The MCP entry has {} that this waymark ({}) does not know.",
                    unknown
                        .iter()
                        .map(|u| format!("`{u}`"))
                        .collect::<Vec<_>>()
                        .join(", "),
                    env!("CARGO_PKG_VERSION")
                ),
                next: format!(
                    "If the entry was made for a newer waymark, {} and restart your agent tool. Otherwise remove the argument: {}. Then restart your agent tool.",
                    crate::platform::update_step(),
                    fix_commands(&kept)
                ),
                blocking: true,
            });
        }
        args.entry_issues = issues;
        args
    }

    /// The first issue that stops the tools.
    pub fn blocking_issue(&self) -> Option<&EntryIssue> {
        self.entry_issues.iter().find(|i| i.blocking)
    }

    /// The issues that only give a notice.
    pub fn entry_notices(&self) -> Vec<String> {
        self.entry_issues
            .iter()
            .filter(|i| !i.blocking)
            .map(|i| i.message.clone())
            .collect()
    }
}

impl Args {
    /// The limits of the MCP entry: the last layer.
    pub fn entry_layer(&self) -> Result<Limits, String> {
        let access = match self.access.as_deref() {
            None => None,
            Some("read-only") => Some(Access::ReadOnly),
            Some("read-write") => Some(Access::ReadWrite),
            Some(other) => {
                return Err(format!(
                    "--access must be read-only or read-write, not \"{other}\"."
                ))
            }
        };
        let mut denied = self.tools_denied.clone();
        if self.no_delete {
            denied.push("delete_document".into());
        }
        Ok(Limits {
            access,
            paused: self.paused.then_some(true),
            tools_denied: (!denied.is_empty()).then_some(denied),
            folders: (!self.folders.is_empty()).then(|| self.folders.clone()),
            max_results: self.max_results,
            max_tokens: self.max_tokens,
            max_documents_opened: self.max_documents_opened,
            ..Limits::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lenient(args: &[&str], deprecated: &[(&str, &str)]) -> Args {
        let mut v = vec!["waymark".to_string()];
        v.extend(args.iter().map(|s| s.to_string()));
        Args::parse_lenient_from(v, deprecated)
    }

    #[test]
    fn known_arguments_make_no_issue() {
        let a = lenient(
            &["--path", "/x", "--access", "read-only", "--no-delete"],
            &[],
        );
        assert!(a.entry_issues.is_empty());
        assert_eq!(a.path.unwrap().to_str(), Some("/x"));
        assert!(a.no_delete);
    }

    #[test]
    fn an_unknown_argument_blocks_and_names_the_fix() {
        for argv in [
            vec!["--path", "/x", "--future-flag", "1"],
            vec!["--path", "/x", "--future-flag=1"],
            vec!["--path", "/x", "--future-flag"],
        ] {
            let a = lenient(&argv, &[]);
            let i = a.blocking_issue().expect("a blocking issue");
            assert!(i.message.contains("`--future-flag`"), "{}", i.message);
            assert!(i.next.contains(crate::platform::update_step()));
            assert!(i.next.contains(
                "claude mcp add --transport stdio --scope user waymark -- waymark --path /x"
            ));
            assert!(!i.next.contains("future-flag"));
            // The known part still parses.
            assert_eq!(a.path.unwrap().to_str(), Some("/x"));
        }
    }

    #[test]
    fn a_deprecated_argument_is_only_a_notice() {
        let a = lenient(
            &["--path", "/x", "--old", "v"],
            &[("--old", "It has no effect any more.")],
        );
        assert!(a.blocking_issue().is_none());
        let n = a.entry_notices();
        assert_eq!(n.len(), 1);
        assert!(n[0].contains("`--old`") && n[0].contains("no effect"));
        assert_eq!(a.path.unwrap().to_str(), Some("/x"));
    }

    #[test]
    fn a_wrong_value_blocks_and_guesses_no_memory_repository() {
        let a = lenient(&["--path", "/x", "--max-results", "many"], &[]);
        let i = a.blocking_issue().expect("a blocking issue");
        assert!(i.message.contains("wrong argument"), "{}", i.message);
        assert!(a.path.is_none() && a.repo.is_none());
    }

    #[test]
    fn entry_layer_from_arguments() {
        let a = Args::parse_from([
            "waymark",
            "--access",
            "read-only",
            "--no-delete",
            "--folders",
            "a/**,b/**",
            "--max-results",
            "3",
        ]);
        let l = a.entry_layer().unwrap();
        assert_eq!(l.access, Some(Access::ReadOnly));
        assert_eq!(l.tools_denied, Some(vec!["delete_document".to_string()]));
        assert_eq!(l.folders.unwrap().len(), 2);
        assert_eq!(l.max_results, Some(3));
    }

    #[test]
    fn a_wrong_access_value_is_an_error() {
        let a = Args::parse_from(["waymark", "--access", "root"]);
        assert!(a.entry_layer().is_err());
    }
}
