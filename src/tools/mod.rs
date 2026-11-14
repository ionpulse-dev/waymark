//! The tools. One tool is one file. Add a tool here and in `registry`.

pub mod add;
pub mod delete;
pub mod deprecate;
pub mod dispute;
pub mod status;
pub mod update_document;

use crate::mcp::tool::{Registry, Tool};

/// All tools of this build.
pub fn registry() -> Registry {
    let tools: Vec<Box<dyn Tool>> = vec![
        Box::new(status::Status),
        Box::new(add::Add),
        Box::new(update_document::UpdateDocument),
        Box::new(dispute::Dispute),
        Box::new(deprecate::Deprecate),
        Box::new(delete::Delete),
    ];
    Registry::new(tools)
}
