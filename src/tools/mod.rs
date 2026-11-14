//! The tools. One tool is one file. Add a tool here and in `registry`.

pub mod status;

use crate::mcp::tool::{Registry, Tool};

/// All tools of this build.
pub fn registry() -> Registry {
    let tools: Vec<Box<dyn Tool>> = vec![Box::new(status::Status)];
    Registry::new(tools)
}
