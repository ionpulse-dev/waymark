//! The tools. One tool is one file. Add a tool here and in `registry`.

pub mod activity;
pub mod add;
pub mod add_empty_folder;
pub mod delete;
pub mod deprecate;
pub mod dispute;
pub mod folders;
pub mod history;
pub mod init;
pub mod lint;
pub mod list;
pub mod list_tags;
pub mod move_documents;
pub mod read_common;
pub mod reject;
pub mod remove_folder;
pub mod rename_tag;
pub mod restore;
pub mod review;
pub mod search;
pub mod show;
pub mod stats;
pub mod status;
pub mod sync;
pub mod undo;
pub mod update_document;
pub mod updates;
pub mod verify;

use crate::mcp::tool::{Registry, Tool};

/// All tools of this build.
pub fn registry() -> Registry {
    let tools: Vec<Box<dyn Tool>> = vec![
        Box::new(status::Status),
        Box::new(search::Search),
        Box::new(show::Show),
        Box::new(list::List),
        Box::new(folders::Folders),
        Box::new(list_tags::ListTags),
        Box::new(history::History),
        Box::new(stats::Stats),
        Box::new(updates::Updates),
        Box::new(lint::Lint),
        Box::new(sync::Sync),
        Box::new(move_documents::Move),
        Box::new(activity::Activity),
        Box::new(undo::Undo),
        Box::new(verify::Verify),
        Box::new(reject::Reject),
        Box::new(review::Review),
        Box::new(add::Add),
        Box::new(update_document::UpdateDocument),
        Box::new(dispute::Dispute),
        Box::new(deprecate::Deprecate),
        Box::new(restore::Restore),
        Box::new(delete::Delete),
        Box::new(add_empty_folder::AddEmptyFolder),
        Box::new(remove_folder::RemoveFolder),
        Box::new(rename_tag::RenameTag),
        Box::new(init::Init),
    ];
    Registry::new(tools)
}
