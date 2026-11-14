# Tasks: Activity and undo

`requires`: `write-documents`, `git-sync`.

- [x] `src/gitsync/log.rs`: `Repo::all_commits` (commits with body and files), `revert_no_commit`, and `abort_revert`. The message body existed already. `commit_change` skips paths that are gone from disk and index. Tested through the tools.
- [x] `src/activity/mod.rs`: entry model, filter, move grouping, `undone`, `later_changes`, paging, and unit tests on a fixture history.
- [x] `src/tools/activity.rs`: arguments, result, and the `kind` hint (read-only).
- [x] `src/tools/undo.rs`: the rules 1 to 7, the preview, and the result.
- [x] `src/tools/mod.rs`: register `activity` and `undo`.
- [x] `tests/it_activity_and_undo.rs`: the cases of `test.md`.
- [x] Run the full suite.
