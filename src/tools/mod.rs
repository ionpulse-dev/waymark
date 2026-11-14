//! The tools. One tool is one file. Add a tool here and in `registry`.

pub mod add;
pub mod delete;
pub mod deprecate;
pub mod dispute;
pub mod index;
pub mod lint;
pub mod list;
pub mod read_common;
pub mod search;
pub mod show;
pub mod stats;
pub mod status;
pub mod sync;
pub mod update_document;

use crate::mcp::tool::{Registry, Tool};

/// All tools of this build.
pub fn registry() -> Registry {
    let tools: Vec<Box<dyn Tool>> = vec![
        Box::new(status::Status),
        Box::new(search::Search),
        Box::new(show::Show),
        Box::new(list::List),
        Box::new(stats::Stats),
        Box::new(lint::Lint),
        Box::new(index::Index),
        Box::new(sync::Sync),
        Box::new(add::Add),
        Box::new(update_document::UpdateDocument),
        Box::new(dispute::Dispute),
        Box::new(deprecate::Deprecate),
        Box::new(delete::Delete),
    ];
    Registry::new(tools)
}
