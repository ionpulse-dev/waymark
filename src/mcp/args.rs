//! The command line and the environment variables (see `specs/foundation/mcp-server`).

use crate::config::{Access, Limits};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone, Default)]
#[command(
    name = "waymark",
    version,
    about = "A knowledge base and memory for AI agents. Starts an MCP server over stdio.",
    after_help = "Every option is optional. Combine as many as you need, in any order.\n\
Your agent tool starts waymark, so the options usually go into the `args` list of its MCP entry.\n\n\
Examples:\n  \
waymark --repo git@github.com:example/knowledge.git\n  \
waymark --repo git@github.com:example/knowledge.git --access read-only --max-results 3\n  \
waymark --path ~/Projects/team-knowledge-base --folders ops,docs/* --no-delete\n\n\
Flags with a value take it after a space or `=`. Lists take comma-separated values with no spaces."
)]
pub struct Args {
    /// Git URL of the knowledge base. The server clones it on first start.
    #[arg(long)]
    pub repo: Option<String>,
    /// The ID that `.mkbase/waymark.toml` must have. Stops the server from opening a different knowledge base by mistake.
    #[arg(long)]
    pub kb_id: Option<String>,
    /// Folder of the clone. Default: `~/Projects/<slug>-knowledge-base`.
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
            denied.push("delete".into());
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
        assert_eq!(l.tools_denied, Some(vec!["delete".to_string()]));
        assert_eq!(l.folders.unwrap().len(), 2);
        assert_eq!(l.max_results, Some(3));
    }

    #[test]
    fn a_wrong_access_value_is_an_error() {
        let a = Args::parse_from(["waymark", "--access", "root"]);
        assert!(a.entry_layer().is_err());
    }
}
