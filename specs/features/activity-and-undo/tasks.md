# Tasks: Activity and undo

`requires`: `write-documents`, `git-sync`.

- [ ] Add `changed_files` and `later_commits_touching` to `src/gitsync/log.rs`, a message body to `src/gitsync/message.rs` (if `write-documents` did not), and `revert_no_commit` and `reset_clean` to `Repo`. Unit tests.
- [ ] `src/activity/mod.rs`: entry model, filter, move grouping, `undone`, `later_changes`, paging, and unit tests on a fixture history.
- [ ] `src/tools/activity.rs`: arguments, result, and the `kind` hint (read-only).
- [ ] `src/tools/undo.rs`: the rules 1 to 7, the preview, and the result.
- [ ] `src/tools/mod.rs`: register `activity` and `undo`.
- [ ] `tests/it_activity_and_undo.rs`: the cases of `test.md`.
- [ ] Run the full suite.
