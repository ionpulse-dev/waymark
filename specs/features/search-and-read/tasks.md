# Tasks: Search and read

_Prerequisite: `mcp-server` is `implemented` or `done`._
_Tasks are in prerequisite-first order._

- [x] Add `mod store;` to `src/lib.rs`. No new dependency: `chrono` is used for dates (see `decisions.md`).
- [x] `src/gitsync/log.rs`: add `renamed_to(path)`. `last_commit_info` is not needed: `Repo::log(1, None, &[path])` gives the author. Tested end to end (T16).
- [x] `src/store/mod.rs`: `Store::load`, `Doc`, `docs()`, `all_docs()`, `get()`. Unit tests: skips dot paths and the `exclude` list, keeps parse findings.
- [x] `src/store/filter.rs`: `Scope` with limit globs and argument folders. Unit tests: a subfolder, an empty list, a limit plus an argument.
- [x] `src/store/rank.rs`: matcher, score, sort, and snippet. Unit tests: AND of terms, weights, tie order, snippet cut.
- [x] `src/store/backlinks.rs`: backlinks over body links and property wikilinks. Unit tests: both link forms.
- [x] `src/store/stats.rs`: the four metrics and the two groupings. Unit tests.
- [x] `src/mcp/context.rs`: add `opened` (the read budget). Unit test: same ID twice costs once.
- [x] `src/tools/read_common.rs`: level rendering, caps, refresh and its notice. Unit tests: cap by count, cap by tokens, first entry always kept.
- [x] `src/tools/search.rs`: input schema, call, description. Register it in `src/tools/mod.rs`.
- [x] `src/tools/show.rs`: input schema, call, budget, version, replacement, moved ID. Register it.
- [x] `src/tools/list.rs`: input schema, call, paging. Register it.
- [x] `src/tools/stats.rs`: input schema, call. Register it.
- [x] `tests/it_search_and_read.rs`: the test cases of `test.md`.
- [x] Remove the timing test T21 and `tests/it_search_speed.rs` (see `decisions.md`, 2026-11-17).
- [x] Check the size of `tools/list` against the budget of `mcp-server`.
- [x] Run the full suite (`make check`).

### Update at the start of every call (2026-11-17)

`git-sync` owns the code change (see its `tasks.md`). This spec only changes its test.

- [x] `tests/it_search_and_read.rs`: rewrite T14 without `WAYMARK_FETCH_MAX_AGE`. A runs `search` once, B pushes, A runs `search` again at once and finds the document.
- [x] Run the full suite (`make check`).
