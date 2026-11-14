//! The test harness (see `specs/foundation/integration-testing`).
#![allow(dead_code)]

pub mod git;
pub mod mcp;
pub mod memrepo;
pub mod world;

#[allow(unused_imports)]
pub use mcp::{McpClient, Reply, Spawn};
#[allow(unused_imports)]
pub use world::{Person, World};
