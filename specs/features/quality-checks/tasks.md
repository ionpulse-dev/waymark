# Tasks: Quality checks

_Prerequisite: `search-and-read` is `implemented` or `done`._

- [ ] Add the `regex` dependency to `Cargo.toml`. Add `mod lint;` and `mod indexgen;` to `src/lib.rs`.
- [ ] `src/gitsync/log.rs`: add `first_authors()` and `last_authors()`. Unit tests with a rename.
- [ ] `src/lint/secrets.rs`: the built-in patterns, the placeholder skip, the extra patterns, and the `setup_problem` for a bad one. Unit tests: one positive and one negative sample for each pattern, and no matched text in the finding.
- [ ] `src/lint/personal.rs`: IBAN with mod 97 and cards with Luhn. Unit tests.
- [ ] `src/lint/duplicates.rs`: the title index, the oldest-wins rule, and `find_title`. Unit tests.
- [ ] `src/lint/authors.rs`: `author-mismatch` and `author-not-allowed`. Unit tests.
- [ ] `src/indexgen/mod.rs`: the index document, the marker line, links, sort, and the `unchanged` and `skipped` decisions. Unit tests, including idempotency.
- [ ] `src/lint/index_stale.rs`: `index-stale`. Unit test.
- [ ] `src/lint/mod.rs`: `run`, `check_candidate`, `into_error` with the `next` table. Unit tests: severity of `broken-link`, the sort order, the cap of 100.
- [ ] `tests/lint_conformance.rs`: every file of `standard/conformance/vault/` has no error, and every file of `invalid/` has the rule that its name says.
- [ ] `src/gitsync/write.rs`: call `lint::check_candidate` in step 5 of the write flow.
- [ ] `src/tools/lint.rs`: schema, call, description. Register it in `src/tools/mod.rs`.
- [ ] `src/tools/index.rs`: schema, call, preview, one commit. Register it.
- [ ] `tests/it_quality_checks.rs`: the test cases of `test.md`.
- [ ] Check the size of `tools/list` against the budget of `mcp-server`.
- [ ] Run the full suite (`make check`).
