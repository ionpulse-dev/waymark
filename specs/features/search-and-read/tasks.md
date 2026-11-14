# Tasks: Search and read

_Prerequisite: `mcp-server` is `implemented` or `done`._
_Tasks are in prerequisite-first order._

- [ ] Add the `time` dependency to `Cargo.toml`. Add `mod store;` to `src/lib.rs`.
- [ ] `src/gitsync/log.rs`: add `last_commit_info(path)` (short hash, author, date) and `renamed_to(path)`. Unit test both on a temp repository.
- [ ] `src/store/mod.rs`: `Store::load`, `Doc`, `docs()`, `all_docs()`, `get()`, and `is_index()`. Unit tests: skips dot paths and the `exclude` list, keeps parse findings.
- [ ] `src/store/filter.rs`: `Scope` with limit globs and argument folders. Unit tests: a subfolder, an empty list, a limit plus an argument.
- [ ] `src/store/rank.rs`: matcher, score, sort, and snippet. Unit tests: AND of terms, weights, tie order, snippet cut.
- [ ] `src/store/backlinks.rs`: backlinks over body links and property wikilinks. Unit tests: both link forms, index files ignored.
- [ ] `src/store/stats.rs`: the four metrics and the two groupings. Unit tests.
- [ ] `src/mcp/context.rs`: add `opened` (the read budget). Unit test: same ID twice costs once.
- [ ] `src/tools/read_common.rs`: level rendering, caps, refresh and its notice. Unit tests: cap by count, cap by tokens, first entry always kept.
- [ ] `src/tools/search.rs`: input schema, call, description. Register it in `src/tools/mod.rs`.
- [ ] `src/tools/show.rs`: input schema, call, budget, version, replacement, moved ID. Register it.
- [ ] `src/tools/list.rs`: input schema, call, paging. Register it.
- [ ] `src/tools/stats.rs`: input schema, call. Register it.
- [ ] `tests/it_search_and_read.rs`: the test cases of `test.md`.
- [ ] Check the size of `tools/list` against the budget of `mcp-server`.
- [ ] Run the full suite (`make check`).
