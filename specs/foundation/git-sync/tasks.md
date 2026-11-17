# Tasks: Git sync

`requires`: `memory-repository`.

- [x] `src/gitsync/cmd.rs`: isolated `git` runs and error mapping.
- [x] `src/gitsync/message.rs`: commit message format.
- [x] `src/gitsync/lock.rs`: the file lock with timeout.
- [x] `src/gitsync/log.rs`: last commit of a path, file at a commit, commits by author.
- [x] `src/gitsync/mod.rs`: `Repo::open`, `Repo::clone_from`, author setup, no-remote mode.
- [x] `src/gitsync/write.rs`: the write transaction with base check, commit, push, and preview.
- [x] `src/gitsync/sync.rs`: pull, rebase, push, and the conflict rule.
- [x] Unit tests for each file with temporary repositories.
- [x] `tests/it_git_sync.rs`: the acceptance criteria with two clones.
- [x] Run the full suite.

### Update at the start of every call (2026-11-17)

- [x] `src/gitsync/mod.rs`: remove `last_fetch` and the freshness parts of `mark_fetched`. Keep `last_pull` and the fetch-failed flag. Add the update gate (`Mutex` plus `Condvar`).
- [x] `src/gitsync/sync.rs`: remove `FRESH_FOR` and `fresh_for()`. Replace `refresh_if_stale` with `refresh_for_read`, which uses the update gate. Test: threads that call it at the same time share the fetch (in `tests/it_git_sync.rs`, which has the two clones and the remote).
- [x] `src/tools/*.rs`: every read tool calls `refresh_for_read`. `status` stays as it was: it reports the last pull and checks the remote itself.
- [x] `tests/*.rs`: remove `WAYMARK_FETCH_MAX_AGE` from the test harnesses (`it_updates`, `it_end_to_end`, `it_lean_tools`, `it_notifications`, `it_pause_and_resume`, `it_search_and_read`).
- [x] `tests/it_git_sync.rs`: the three new acceptance criteria.
- [x] Run the full suite.
