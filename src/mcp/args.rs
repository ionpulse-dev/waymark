//! The command line and the environment variables (see `specs/foundation/mcp-server`).

use crate::config::{Access, Limits};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone, Default)]
#[command(
    name = "waymark",
    version,
    about = "A knowledge base and memory for AI agents. Starts an MCP server over stdio."
)]
pub struct Args {
    /// Git URL of the knowledge base. The server clones it on first start.
    #[arg(long)]
    pub repo: Option<String>,
    /// The ID that `.mkbase/waymark.toml` must have.
    #[arg(long)]
    pub kb_id: Option<String>,
    /// Folder of the clone. Default: `~/Projects/<slug>-knowledge-base`.
    #[arg(long)]
    pub path: Option<PathBuf>,
    #[arg(long)]
    pub author_name: Option<String>,
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
    #[arg(long)]
    pub max_results: Option<usize>,
    #[arg(long)]
    pub max_tokens: Option<usize>,
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
