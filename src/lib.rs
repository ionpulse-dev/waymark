//! waymark: long-term agent memory.
//!
//! The library holds all logic. The binary only starts the server.

pub mod activity;
pub mod config;
pub mod error;
pub mod format;
pub mod gitsync;
pub mod identity;
pub mod indexgen;
pub mod lint;
pub mod maintenance;
pub mod manual;
pub mod mcp;
pub mod memrepo;
pub mod notices;
pub mod state;
pub mod store;
pub mod tools;
pub mod writes;
