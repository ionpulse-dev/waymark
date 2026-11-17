# Tasks: Updates

_Prerequisite: `search-and-read` is `implemented` or `done`._

- [x] `src/state/mod.rs`: `State` with `get`, `set`, atomic write, and the folder rules. Add `mod state;` to `src/lib.rs`. Unit tests: other keys stay, a missing file is empty.
- [x] `src/gitsync/diff.rs`: `base_commit_before`, `changed_paths`, `last_change_by_others`. Unit tests on a temp repository, with a rename and a delete.
- [x] `src/store/changes.rs`: `Change`, `changes_since`, and the classification. Unit tests: every `kind`, own changes dropped, a moved document, the empty-tree base.
- [x] `src/tools/updates.rs`: input schema, parsing of `since`, the filter, order, cap, mark, message. Register it in `src/tools/mod.rs`.
- [x] `tests/it_updates.rs`: the test cases of `test.md`.
- [x] Check the size of `tools/list` against the budget of `mcp-server`.
- [x] Run the full suite (`make check`).

### Update at the start of every call (2026-11-17)

`git-sync` owns the code change (see its `tasks.md`). This spec only changes its test.

- [x] `tests/it_updates.rs`: rewrite T10 without `WAYMARK_FETCH_MAX_AGE`. B pushes, A runs `updates` at once and lists the change.
- [x] Run the full suite (`make check`).
