# Tasks: Manual edits and sync

_Prerequisite: `quality-checks` is `implemented` or `done`._

- [x] `src/lint/mod.rs`: `run` now takes an `include` predicate instead of a `Scope`, so it checks any set of documents. Tested through `sync`.
- [x] `src/manual/pending.rs`: parse `git status`, classify, skip non-documents and `.waymark/` files. Tested end to end (T1, T10, T11, T13, T15).
- [x] touching is in `src/manual/mod.rs` (`prepare`). Tested end to end (T9).
- [x] `src/manual/mod.rs`: `prepare` (lint, touch) and `commit_before_write`. Tested end to end (T1 to T6).
- [x] `src/gitsync/sync.rs`: `Repo::sync` already took a hook. Added `pulled` to `SyncResult` and `Repo::pull_only`.
- [x] `src/writes/mod.rs`: the hook runs in `execute`, inside the lock after the fetch, and `read_doc` gives `stale_base` for a document with unpublished edits (see `decisions.md`).
- [x] no hook object in `Context`: `writes::execute` calls `manual::commit_before_write`.
- [x] `sync` has the kind `read`, so the registry lists it in read-only and paused states. The tool checks the limits itself.
- [x] `src/tools/sync.rs`: schema, call, read-only mode, preview, the all-blocked error, the result. Register it in `src/tools/mod.rs`.
- [x] `tests/it_manual_edits.rs`: the test cases of `test.md`.
- [x] Check the size of `tools/list` against the budget of `mcp-server`.
- [x] Run the full suite (`make check`).
- [x] `src/manual/mod.rs`: a broken body link that an edit adds, and a deletion of a linked document, block the edit (2026-11-14).
