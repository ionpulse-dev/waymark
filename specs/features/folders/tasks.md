# Tasks: Folders

- [x] `src/kb/folders.rs`: the folder model (`KnowledgeBase::folders`, `GITKEEP`).
- [x] `src/gitsync/message.rs`: the verbs `add-folder` and `remove-folder`.
- [x] `src/gitsync/write.rs`: `Tx::remove_file` removes the empty parent folders.
- [x] `src/writes/mod.rs`: `FileChange::WriteFile` and `RemoveFile`, and `check_folder_path`.
- [x] `src/error.rs`, `src/mcp/error.rs`: the optional `data` of an error.
- [x] `src/tools/folders.rs`.
- [x] `src/tools/add_empty_folder.rs`.
- [x] `src/tools/remove_folder.rs`.
- [x] `src/tools/move_documents.rs`: an empty folder can move.
- [x] Register the three tools (`src/tools/mod.rs`).
- [x] `tests/it_folders.rs`: the cases of `test.md`.
- [x] `docs/install.md`, `docs/agent-tools-research.md`, `README.md`: the new tools.
- [x] Run the full suite.
