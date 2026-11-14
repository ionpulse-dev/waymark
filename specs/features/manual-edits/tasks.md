# Tasks: Manual edits and sync

_Prerequisite: `quality-checks` is `implemented` or `done`._

- [ ] `src/lint/mod.rs`: add `run_on(store, paths)` if `run` does not give it. Unit test.
- [ ] `src/manual/pending.rs`: parse `git status`, classify, skip non-documents and `.mkbase/` files. Unit tests with a temp repository: new, changed, deleted, image, dot file, excluded path.
- [ ] `src/manual/touch.rs`: set `updated` and `updated_by` with `format::render`. Unit tests: body unchanged, no write when equal.
- [ ] `src/manual/mod.rs`: `commit_pending` (lint, touch, approval, commit) and the quiet variant. Unit tests: pass, block, mixed, all blocked.
- [ ] `src/gitsync/sync.rs`: add the hook parameter to `Repo::sync`. Unit test with a fake hook.
- [ ] `src/gitsync/write.rs`: add `install_pending_hook`, the call after the fetch, and the check for uncommitted paths with `stale_base`. Unit tests.
- [ ] `src/mcp/context.rs`: install the hook.
- [ ] `src/mcp/tool.rs`: list `sync` in read-only and paused states.
- [ ] `src/tools/sync.rs`: schema, call, read-only mode, preview, the all-blocked error, the result. Register it in `src/tools/mod.rs`.
- [ ] `tests/it_manual_edits.rs`: the test cases of `test.md`.
- [ ] Check the size of `tools/list` against the budget of `mcp-server`.
- [ ] Run the full suite (`make check`).
